//! Tiny date helpers (UTC) so the CLI needs no date library.

use anyhow::{bail, Result};

const SECONDS_PER_DAY: i64 = 86_400;

/// Parses `YYYY-MM-DD` into Unix seconds at 00:00 UTC.
pub fn parse_date(text: &str) -> Result<i64> {
    let parts: Vec<&str> = text.trim().split('-').collect();
    let [year, month, day] = parts.as_slice() else {
        bail!("expected a date like 2026-12-31");
    };
    let (year, month, day): (i64, u32, u32) = (year.parse()?, month.parse()?, day.parse()?);
    if !(1..=12).contains(&month) || !(1..=31).contains(&day) {
        bail!("expected a date like 2026-12-31");
    }
    Ok(days_from_civil(year, month, day) * SECONDS_PER_DAY)
}

/// Formats Unix seconds as `YYYY-MM-DD HH:MM` (UTC).
pub fn format_time(unix: i64) -> String {
    let days = unix.div_euclid(SECONDS_PER_DAY);
    let seconds = unix.rem_euclid(SECONDS_PER_DAY);
    let (year, month, day) = civil_from_days(days);
    format!(
        "{year:04}-{month:02}-{day:02} {:02}:{:02}",
        seconds / 3600,
        (seconds % 3600) / 60
    )
}

/// Formats only the date part.
pub fn format_date(unix: i64) -> String {
    format_time(unix).chars().take(10).collect()
}

// Howard Hinnant's civil-date algorithms.
fn days_from_civil(year: i64, month: u32, day: u32) -> i64 {
    let y = if month <= 2 { year - 1 } else { year };
    let era = y.div_euclid(400);
    let yoe = y - era * 400;
    let mp = i64::from((month + 9) % 12);
    let doy = (153 * mp + 2) / 5 + i64::from(day) - 1;
    let doe = yoe * 365 + yoe / 4 - yoe / 100 + doy;
    era * 146_097 + doe - 719_468
}

fn civil_from_days(days: i64) -> (i64, u32, u32) {
    let z = days + 719_468;
    let era = z.div_euclid(146_097);
    let doe = z - era * 146_097;
    let yoe = (doe - doe / 1460 + doe / 36_524 - doe / 146_096) / 365;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let day = u32::try_from(doy - (153 * mp + 2) / 5 + 1).unwrap_or(1);
    let month = u32::try_from(if mp < 10 { mp + 3 } else { mp - 9 }).unwrap_or(1);
    let year = yoe + era * 400 + i64::from(month <= 2);
    (year, month, day)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn round_trips_dates() {
        let unix = parse_date("2026-12-04").unwrap();
        assert_eq!(format_date(unix), "2026-12-04");
        assert_eq!(parse_date("1970-01-01").unwrap(), 0);
        assert_eq!(format_time(86_400 + 3_660), "1970-01-02 01:01");
    }

    #[test]
    fn rejects_garbage() {
        assert!(parse_date("tomorrow").is_err());
        assert!(parse_date("2026-13-01").is_err());
    }
}
