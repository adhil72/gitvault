//! Sign in with GitHub.

use std::sync::Arc;

use super::{context::Context, keys};
use crate::{
    domain::account::GithubAccount,
    dto::{AccountDto, DeviceCodeDto, GhAccountDto, LoginPollDto},
    error::CoreError,
    ports::DevicePoll,
};

pub struct AuthService {
    ctx: Arc<Context>,
}

impl AuthService {
    #[must_use]
    pub const fn new(ctx: Arc<Context>) -> Self {
        Self { ctx }
    }

    fn client_id(&self) -> Result<String, CoreError> {
        self.ctx
            .settings()
            .github_client_id
            .or_else(|| self.ctx.config.github_client_id.clone())
            .filter(|id| !id.trim().is_empty())
            .ok_or_else(|| {
                CoreError::Validation(
                    "no GitHub OAuth client id configured: set GITVAULT_GITHUB_CLIENT_ID, \
                     or sign in with a personal access token (`gitvault auth login --token`)"
                        .into(),
                )
            })
    }

    /// Starts the GitHub device flow.
    ///
    /// # Errors
    /// Validation error without a client id; network errors from GitHub.
    pub fn login_start(&self) -> Result<DeviceCodeDto, CoreError> {
        let code = self.ctx.host.device_code(&self.client_id()?)?;
        Ok(DeviceCodeDto {
            device_code: code.device_code,
            user_code: code.user_code,
            verification_uri: code.verification_uri,
            expires_in: code.expires_in,
            interval: code.interval,
        })
    }

    /// Polls once for the result of the device flow.
    ///
    /// # Errors
    /// Network errors from GitHub.
    pub fn login_poll(&self, device_code: &str) -> Result<LoginPollDto, CoreError> {
        match self
            .ctx
            .host
            .poll_device_token(&self.client_id()?, device_code)?
        {
            DevicePoll::Pending => Ok(LoginPollDto::Pending),
            DevicePoll::SlowDown => Ok(LoginPollDto::SlowDown),
            DevicePoll::Expired => Ok(LoginPollDto::Expired),
            DevicePoll::Denied => Ok(LoginPollDto::Denied),
            DevicePoll::Authorized { token } => Ok(LoginPollDto::Authorized {
                account: self.login_with_token(&token)?,
            }),
        }
    }

    /// Signs in with the token of the locally signed-in GitHub CLI (no OAuth app needed).
    ///
    /// # Errors
    /// Fails when `gh` is missing or signed out, or GitHub rejects its token.
    pub fn login_with_gh_cli(&self) -> Result<AccountDto, CoreError> {
        self.login_with_token(&self.ctx.gh_cli.token()?)
    }

    /// Accounts of the local GitHub CLI; `current` marks the one GitVault uses.
    #[must_use]
    pub fn gh_accounts(&self) -> Vec<GhAccountDto> {
        let signed_in = self.ctx.account().map(|a| a.login);
        self.ctx
            .gh_cli
            .accounts()
            .into_iter()
            .map(|a| GhAccountDto {
                current: signed_in.as_deref() == Some(a.login.as_str()),
                login: a.login,
            })
            .collect()
    }

    /// Signs in with one specific account of the GitHub CLI (the CLI's own active account is untouched).
    ///
    /// # Errors
    /// Fails when `gh` has no such account or GitHub rejects its token.
    pub fn login_with_gh_account(&self, login: &str) -> Result<AccountDto, CoreError> {
        self.login_with_token(&self.ctx.gh_cli.token_for(login)?)
    }

    /// Signs in with an existing token (personal access token or `gh auth token`).
    ///
    /// # Errors
    /// [`CoreError::NotAuthenticated`] when GitHub rejects the token.
    pub fn login_with_token(&self, token: &str) -> Result<AccountDto, CoreError> {
        let token = token.trim();
        let account = self.ctx.host.current_user(token)?;
        let json =
            serde_json::to_string(&account).map_err(|e| CoreError::Internal(e.to_string()))?;
        self.ctx.kv.set(keys::GITHUB_TOKEN, token)?;
        self.ctx.kv.set(keys::GITHUB_ACCOUNT, &json)?;
        Ok(to_dto(&account))
    }

    /// Forgets the GitHub token. Local vaults are untouched.
    ///
    /// # Errors
    /// Fails on a storage error.
    pub fn logout(&self) -> Result<(), CoreError> {
        self.ctx.kv.delete(keys::GITHUB_TOKEN)?;
        self.ctx.kv.delete(keys::GITHUB_ACCOUNT)
    }

    /// The signed-in account, if any (no network call).
    #[must_use]
    pub fn status(&self) -> Option<AccountDto> {
        self.ctx.token().ok()?;
        self.ctx.account().map(|a| to_dto(&a))
    }
}

fn to_dto(account: &GithubAccount) -> AccountDto {
    AccountDto {
        login: account.login.clone(),
        id: account.id,
        name: account.name.clone(),
        avatar_url: account.avatar_url.clone(),
    }
}
