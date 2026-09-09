use chrono::{DateTime, Datelike, Duration, NaiveDate, TimeZone, Utc};

use crate::error::{log_technical_error, AppError};

const MONTHS: [&str; 12] = [
    "January",
    "February",
    "March",
    "April",
    "May",
    "June",
    "July",
    "August",
    "September",
    "October",
    "November",
    "December",
];

const MONTHS_SHORT: [&str; 12] = [
    "Jan", "Feb", "Mar", "Apr", "May", "Jun", "Jul", "Aug", "Sep", "Oct", "Nov", "Dec",
];

const WEEKDAYS_SHORT: [&str; 7] = ["Sun", "Mon", "Tue", "Wed", "Thu", "Fri", "Sat"];

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Period {
    Day,
    Week,
    Month,
}

impl Period {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Day => "day",
            Self::Week => "week",
            Self::Month => "month",
        }
    }
}

pub fn parse_rfc3339(value: &str) -> Result<DateTime<Utc>, AppError> {
    DateTime::parse_from_rfc3339(value)
        .map(|parsed| parsed.with_timezone(&Utc))
        .map_err(|error| {
            log_technical_error("parse timestamp", &error);
            AppError::validation("Enter a valid date and time.")
        })
}

pub fn parse_date(value: &str) -> Result<NaiveDate, AppError> {
    NaiveDate::parse_from_str(value, "%Y-%m-%d")
        .map_err(|_| AppError::validation("Enter a valid date."))
}

pub fn format_date(date: NaiveDate) -> String {
    date.format("%Y-%m-%d").to_string()
}

pub fn local_date<Tz: TimeZone>(instant: DateTime<Utc>, tz: &Tz) -> NaiveDate {
    instant.with_timezone(tz).date_naive()
}

pub fn start_of_local_day<Tz: TimeZone>(date: NaiveDate, tz: &Tz) -> Result<DateTime<Utc>, AppError> {
    let naive = date
        .and_hms_opt(0, 0, 0)
        .ok_or_else(|| AppError::validation("Enter a valid date."))?;
    Ok(resolve_local_datetime(tz, naive)?.with_timezone(&Utc))
}

pub fn week_start(date: NaiveDate) -> NaiveDate {
    let days = date.weekday().num_days_from_sunday() as i64;
    date - Duration::days(days)
}

pub fn week_column_labels(start: NaiveDate) -> Vec<String> {
    (0..7)
        .map(|offset| {
            let day = add_days(start, offset);
            format!(
                "{} {}",
                WEEKDAYS_SHORT[day.weekday().num_days_from_sunday() as usize],
                day.day()
            )
        })
        .collect()
}

pub fn month_column_labels(start: NaiveDate, end: NaiveDate) -> Vec<String> {
    let mut labels = Vec::new();
    let mut day = start;
    while day <= end {
        labels.push(day.day().to_string());
        day = add_days(day, 1);
    }
    labels
}

pub fn month_start(date: NaiveDate) -> NaiveDate {
    NaiveDate::from_ymd_opt(date.year(), date.month(), 1).unwrap_or(date)
}

pub fn add_days(date: NaiveDate, days: i64) -> NaiveDate {
    date + Duration::days(days)
}

pub fn add_months(date: NaiveDate, months: i32) -> NaiveDate {
    let mut year = date.year();
    let mut month = date.month() as i32 + months;
    while month > 12 {
        month -= 12;
        year += 1;
    }
    while month < 1 {
        month += 12;
        year -= 1;
    }
    NaiveDate::from_ymd_opt(year, month as u32, 1).unwrap_or(date)
}

pub fn period_range(date: NaiveDate, period: Period) -> (NaiveDate, NaiveDate) {
    match period {
        Period::Day => (date, date),
        Period::Week => {
            let start = week_start(date);
            (start, add_days(start, 6))
        }
        Period::Month => {
            let start = month_start(date);
            let end = add_days(add_months(start, 1), -1);
            (start, end)
        }
    }
}

pub fn period_label(date: NaiveDate, period: Period) -> String {
    let (start, _) = period_range(date, period);
    match period {
        Period::Day => format!(
            "{} {}, {}",
            MONTHS[(start.month() - 1) as usize],
            start.day(),
            start.year()
        ),
        Period::Week => format!(
            "Week of {} {}",
            MONTHS_SHORT[(start.month() - 1) as usize],
            start.day()
        ),
        Period::Month => format!(
            "{} {}",
            MONTHS[(start.month() - 1) as usize],
            start.year()
        ),
    }
}

pub fn elapsed_seconds_since(started_at: DateTime<Utc>) -> i64 {
    (Utc::now() - started_at).num_seconds().max(0)
}

pub fn format_elapsed_hm(total_seconds: i64) -> String {
    let seconds = total_seconds.max(0);
    let hours = seconds / 3600;
    let minutes = (seconds % 3600) / 60;
    format!("{hours:02}:{minutes:02}")
}

pub fn format_elapsed_hms(total_seconds: i64) -> String {
    let seconds = total_seconds.max(0);
    let hours = seconds / 3600;
    let minutes = (seconds % 3600) / 60;
    let secs = seconds % 60;
    format!("{hours:02}:{minutes:02}:{secs:02}")
}

pub fn format_hours_minutes(total_minutes: i64) -> String {
    let minutes = total_minutes.max(0);
    let hours = minutes / 60;
    let remainder = minutes % 60;
    format!("{hours}h {remainder:02}m")
}

/// Paired with the template clock glyph in the menu bar, so the title carries no icon of its own.
pub fn tray_title(client_name: Option<&str>, elapsed_seconds: i64) -> String {
    match client_name {
        Some(name) => format!("{name}  {}", format_elapsed_hm(elapsed_seconds)),
        None => "Kron".to_string(),
    }
}

pub fn adjacent_date(date: NaiveDate, period: Period, direction: i32) -> NaiveDate {
    match period {
        Period::Day => add_days(date, direction as i64),
        Period::Week => add_days(date, 7 * direction as i64),
        Period::Month => add_months(month_start(date), direction),
    }
}

fn resolve_local_datetime<Tz: TimeZone>(
    tz: &Tz,
    naive: chrono::NaiveDateTime,
) -> Result<DateTime<Tz>, AppError> {
    match tz.from_local_datetime(&naive) {
        chrono::LocalResult::Single(datetime) => Ok(datetime),
        chrono::LocalResult::Ambiguous(earliest, _) => Ok(earliest),
        chrono::LocalResult::None => {
            let shifted = naive + Duration::hours(1);
            match tz.from_local_datetime(&shifted) {
                chrono::LocalResult::Single(datetime) => Ok(datetime),
                chrono::LocalResult::Ambiguous(earliest, _) => Ok(earliest),
                chrono::LocalResult::None => {
                    log_technical_error("resolve local datetime", &naive);
                    Err(AppError::validation("Enter a valid date and time."))
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::{FixedOffset, Weekday};

    #[test]
    fn weeks_start_on_sunday() {
        let thursday = NaiveDate::from_ymd_opt(2026, 9, 3).unwrap();
        assert_eq!(week_start(thursday), NaiveDate::from_ymd_opt(2026, 8, 30).unwrap());
        assert_eq!(thursday.weekday(), Weekday::Thu);
        assert_eq!(
            week_column_labels(week_start(thursday)),
            vec!["Sun 30", "Mon 31", "Tue 1", "Wed 2", "Thu 3", "Fri 4", "Sat 5"]
        );
    }

    #[test]
    fn start_of_day_uses_local_offset() {
        let tz = FixedOffset::west_opt(4 * 3600).unwrap();
        let date = NaiveDate::from_ymd_opt(2026, 9, 3).unwrap();
        let start = start_of_local_day(date, &tz).unwrap();
        assert_eq!(local_date(start, &tz), date);
    }

    #[test]
    fn formats_elapsed_hours_and_minutes_without_seconds() {
        assert_eq!(format_elapsed_hm(0), "00:00");
        assert_eq!(format_elapsed_hm(137), "00:02");
        assert_eq!(format_elapsed_hm(2 * 3600 + 17 * 60 + 34), "02:17");
    }

    #[test]
    fn formats_elapsed_hours_minutes_seconds() {
        assert_eq!(format_elapsed_hms(0), "00:00:00");
        assert_eq!(format_elapsed_hms(2 * 3600 + 17 * 60 + 34), "02:17:34");
    }

    #[test]
    fn formats_rounded_hours_minutes_label() {
        assert_eq!(format_hours_minutes(0), "0h 00m");
        assert_eq!(format_hours_minutes(405), "6h 45m");
    }

    #[test]
    fn formats_tray_title_for_idle_and_running() {
        assert_eq!(tray_title(None, 0), "Kron");
        assert_eq!(tray_title(Some("Client A"), 2 * 3600 + 17 * 60), "Client A  02:17");
    }
}
