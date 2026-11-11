//! Listing, searching and faceting secrets.

use std::collections::BTreeMap;

use super::{mapping, SecretService};
use crate::{
    domain::secret::SecretPayload,
    dto::{FacetCount, Facets, ListSecretsQuery, SecretSort, SecretSummary},
    error::CoreError,
};

const EXPIRING_SOON_DAYS: i64 = 30;

fn same(field: Option<&str>, wanted: &str) -> bool {
    field.is_some_and(|f| f.eq_ignore_ascii_case(wanted))
}

fn matches(payload: &SecretPayload, query: &ListSecretsQuery, now: i64) -> bool {
    let days = mapping::expires_in_days(payload.expires_at, now);
    query
        .environment
        .as_deref()
        .is_none_or(|e| same(payload.environment.as_deref(), e))
        && query
            .tag
            .as_deref()
            .is_none_or(|t| payload.tags.iter().any(|x| x.eq_ignore_ascii_case(t)))
        && query.kind.is_none_or(|k| payload.kind == k)
        && (!query.favorites_only || payload.favorite)
        && query
            .expiring_within_days
            .is_none_or(|n| days.is_some_and(|d| d <= i64::from(n)))
        && query
            .search
            .as_deref()
            .is_none_or(|s| matches_search(payload, &s.to_lowercase()))
}

fn matches_search(payload: &SecretPayload, needle: &str) -> bool {
    let has = |text: &str| text.to_lowercase().contains(needle);
    has(&payload.name)
        || payload.environment.as_deref().is_some_and(has)
        || payload.tags.iter().any(|t| has(t))
}

fn count(map: &mut BTreeMap<String, u32>, value: Option<&str>) {
    if let Some(value) = value {
        *map.entry(value.to_owned()).or_insert(0) += 1;
    }
}

fn to_counts(map: BTreeMap<String, u32>) -> Vec<FacetCount> {
    map.into_iter()
        .map(|(name, count)| FacetCount { name, count })
        .collect()
}

fn sort_summaries(items: &mut [SecretSummary], sort: SecretSort) {
    match sort {
        SecretSort::Updated => items.sort_by_key(|s| std::cmp::Reverse(s.updated_at)),
        SecretSort::Name => items.sort_by_key(|s| s.name.to_lowercase()),
        SecretSort::Expiry => items.sort_by_key(|s| s.expires_at.unwrap_or(i64::MAX)),
    }
}

impl SecretService {
    /// Secrets of a vault matching the query, sorted (default: recently updated).
    ///
    /// # Errors
    /// A vault this device never unlocked while locked, or a storage error.
    pub fn list(&self, query: &ListSecretsQuery) -> Result<Vec<SecretSummary>, CoreError> {
        let (_, all) = self.browse(&query.vault_id)?;
        let now = self.ctx.now();
        let mut items: Vec<SecretSummary> = all
            .iter()
            .filter(|(_, payload)| matches(payload, query, now))
            .map(|(id, payload)| mapping::summary(id, payload, now))
            .collect();
        sort_summaries(
            &mut items,
            query.sort.clone().unwrap_or(SecretSort::Updated),
        );
        Ok(items)
    }

    /// Counts per environment and tag (for the sidebar).
    ///
    /// # Errors
    /// Locked vault or a storage error.
    pub fn facets(&self, vault_id: &str) -> Result<Facets, CoreError> {
        let (_, all) = self.browse(vault_id)?;
        let now = self.ctx.now();
        let (mut environments, mut tags) = (BTreeMap::new(), BTreeMap::new());
        let (mut favorites, mut expiring_soon) = (0, 0);
        for (_, payload) in &all {
            count(&mut environments, payload.environment.as_deref());
            for tag in &payload.tags {
                count(&mut tags, Some(tag));
            }
            favorites += u32::from(payload.favorite);
            let days = mapping::expires_in_days(payload.expires_at, now);
            expiring_soon += u32::from(days.is_some_and(|d| d <= EXPIRING_SOON_DAYS));
        }
        Ok(Facets {
            total: u32::try_from(all.len()).unwrap_or(u32::MAX),
            favorites,
            expiring_soon,
            environments: to_counts(environments),
            tags: to_counts(tags),
        })
    }
}
