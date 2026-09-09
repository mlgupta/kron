use chrono::{NaiveDate, TimeZone};
use rusqlite::Connection;

use crate::error::AppError;
use crate::models::{Report, ReportRow};
use crate::services::session_service;
use crate::timeutil::{self, parse_rfc3339, Period};

pub fn get_daily_totals<Tz: TimeZone>(
    conn: &Connection,
    date: NaiveDate,
    tz: &Tz,
) -> Result<Report, AppError> {
    build_report(conn, date, Period::Day, tz)
}

pub fn get_weekly_totals<Tz: TimeZone>(
    conn: &Connection,
    date: NaiveDate,
    tz: &Tz,
) -> Result<Report, AppError> {
    build_report(conn, date, Period::Week, tz)
}

pub fn get_monthly_totals<Tz: TimeZone>(
    conn: &Connection,
    date: NaiveDate,
    tz: &Tz,
) -> Result<Report, AppError> {
    build_report(conn, date, Period::Month, tz)
}

fn build_report<Tz: TimeZone>(
    conn: &Connection,
    date: NaiveDate,
    period: Period,
    tz: &Tz,
) -> Result<Report, AppError> {
    let (start, end) = timeutil::period_range(date, period);
    let sessions = session_service::get_sessions(conn, None, Some(start), Some(end), tz)?;
    let columns = match period {
        Period::Week => timeutil::week_column_labels(start),
        Period::Month => timeutil::month_column_labels(start, end),
        Period::Day => Vec::new(),
    };
    let day_slots = columns.len();

    let mut rows: Vec<ReportRow> = Vec::new();
    for session in sessions {
        let day_index = if day_slots == 0 {
            None
        } else {
            let started = parse_rfc3339(&session.started_at)?;
            let session_date = timeutil::local_date(started, tz);
            let offset = (session_date - start).num_days();
            if offset >= 0 && offset < day_slots as i64 {
                Some(offset as usize)
            } else {
                None
            }
        };

        if let Some(existing) = rows.iter_mut().find(|row| row.client_id == session.client_id) {
            existing.rounded_minutes += session.rounded_duration_minutes;
            if let Some(index) = day_index {
                existing.daily_minutes[index] += session.rounded_duration_minutes;
            }
        } else {
            let mut daily_minutes = vec![0; day_slots];
            if let Some(index) = day_index {
                daily_minutes[index] = session.rounded_duration_minutes;
            }
            rows.push(ReportRow {
                client_id: session.client_id,
                client_name: session.client_name.clone(),
                rounded_minutes: session.rounded_duration_minutes,
                daily_minutes,
                short_name: client_short_name(&session.client_name),
                color: client_color(session.client_id),
            });
        }
    }

    rows.sort_by(|left, right| left.client_name.to_lowercase().cmp(&right.client_name.to_lowercase()));
    let total_rounded_minutes = rows.iter().map(|row| row.rounded_minutes).sum();

    Ok(Report {
        period: period.as_str().to_string(),
        label: timeutil::period_label(date, period),
        start_date: timeutil::format_date(start),
        end_date: timeutil::format_date(end),
        previous_date: timeutil::format_date(timeutil::adjacent_date(start, period, -1)),
        next_date: timeutil::format_date(timeutil::adjacent_date(start, period, 1)),
        rows,
        total_rounded_minutes,
        columns,
    })
}

const CLIENT_COLORS: [&str; 10] = [
    "#3B5BDB",
    "#0B7285",
    "#2B8A3E",
    "#E67700",
    "#C92A2A",
    "#5F3DC4",
    "#087F5B",
    "#A61E4D",
    "#364FC7",
    "#862E9C",
];

fn client_color(client_id: i64) -> String {
    CLIENT_COLORS[(client_id.unsigned_abs() as usize) % CLIENT_COLORS.len()].to_string()
}

fn client_short_name(name: &str) -> String {
    let words: Vec<&str> = name.split_whitespace().filter(|word| !word.is_empty()).collect();
    if words.len() >= 2 {
        words
            .iter()
            .filter_map(|word| word.chars().find(|ch| ch.is_alphanumeric()))
            .take(3)
            .map(|ch| ch.to_uppercase().to_string())
            .collect()
    } else {
        name.chars()
            .filter(|ch| ch.is_alphanumeric())
            .take(3)
            .map(|ch| ch.to_uppercase().to_string())
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db;
    use crate::services::client_service;
    use crate::services::session_service::insert_session;
    use crate::timeutil::parse_rfc3339;
    use chrono::{FixedOffset, Utc};

    fn tz() -> FixedOffset {
        FixedOffset::west_opt(4 * 3600).unwrap()
    }

    fn at(stamp: &str) -> chrono::DateTime<Utc> {
        parse_rfc3339(stamp).unwrap()
    }

    fn date(year: i32, month: u32, day: u32) -> NaiveDate {
        NaiveDate::from_ymd_opt(year, month, day).unwrap()
    }

    #[test]
    fn daily_totals_use_rounded_minutes() {
        let conn = db::open_in_memory().unwrap();
        let acme = client_service::create_client(&conn, "Acme", None).unwrap();
        let globex = client_service::create_client(&conn, "Globex", None).unwrap();
        // 16 actual minutes -> 30 rounded; 14 actual -> 15 rounded
        insert_session(&conn, acme.id, at("2026-09-03T09:00:00-04:00"), at("2026-09-03T09:16:00-04:00"), None).unwrap();
        insert_session(&conn, globex.id, at("2026-09-03T10:00:00-04:00"), at("2026-09-03T10:14:00-04:00"), None).unwrap();
        insert_session(&conn, acme.id, at("2026-09-04T09:00:00-04:00"), at("2026-09-04T10:00:00-04:00"), None).unwrap();

        let report = get_daily_totals(&conn, date(2026, 9, 3), &tz()).unwrap();
        assert_eq!(report.label, "September 3, 2026");
        assert_eq!(report.total_rounded_minutes, 45);
        assert_eq!(report.rows[0].client_name, "Acme");
        assert_eq!(report.rows[0].rounded_minutes, 30);
        assert_eq!(report.rows[1].rounded_minutes, 15);
        assert_ne!(
            report.total_rounded_minutes,
            (16 + 14) as i64,
            "reports must not sum actual minutes"
        );
    }

    #[test]
    fn weekly_totals_are_sunday_through_saturday_by_day() {
        let conn = db::open_in_memory().unwrap();
        let acme = client_service::create_client(&conn, "Acme", None).unwrap();
        let globex = client_service::create_client(&conn, "Globex", None).unwrap();
        insert_session(&conn, acme.id, at("2026-08-31T09:00:00-04:00"), at("2026-08-31T10:00:00-04:00"), None).unwrap();
        insert_session(&conn, acme.id, at("2026-09-03T09:00:00-04:00"), at("2026-09-03T09:20:00-04:00"), None).unwrap();
        insert_session(&conn, acme.id, at("2026-08-30T09:00:00-04:00"), at("2026-08-30T10:00:00-04:00"), None).unwrap();
        insert_session(&conn, globex.id, at("2026-09-05T09:00:00-04:00"), at("2026-09-05T09:45:00-04:00"), None).unwrap();
        insert_session(&conn, acme.id, at("2026-09-06T09:00:00-04:00"), at("2026-09-06T10:00:00-04:00"), None).unwrap();

        let report = get_weekly_totals(&conn, date(2026, 9, 3), &tz()).unwrap();
        assert_eq!(report.label, "Week of Aug 30");
        assert_eq!(report.start_date, "2026-08-30");
        assert_eq!(report.end_date, "2026-09-05");
        assert_eq!(
            report.columns,
            vec!["Sun 30", "Mon 31", "Tue 1", "Wed 2", "Thu 3", "Fri 4", "Sat 5"]
        );
        assert_eq!(report.rows.len(), 2);
        assert_eq!(report.rows[0].client_name, "Acme");
        assert_eq!(report.rows[0].daily_minutes, vec![60, 60, 0, 0, 30, 0, 0]);
        assert_eq!(report.rows[0].rounded_minutes, 150);
        assert_eq!(report.rows[1].client_name, "Globex");
        assert_eq!(report.rows[1].daily_minutes, vec![0, 0, 0, 0, 0, 0, 45]);
        assert_eq!(report.total_rounded_minutes, 195);
    }

    #[test]
    fn monthly_totals_stay_inside_month() {
        let conn = db::open_in_memory().unwrap();
        let acme = client_service::create_client(&conn, "Acme", None).unwrap();
        insert_session(&conn, acme.id, at("2026-08-31T09:00:00-04:00"), at("2026-08-31T09:20:00-04:00"), None).unwrap();
        insert_session(&conn, acme.id, at("2026-09-01T09:00:00-04:00"), at("2026-09-01T10:00:00-04:00"), None).unwrap();

        let report = get_monthly_totals(&conn, date(2026, 9, 15), &tz()).unwrap();
        assert_eq!(report.label, "September 2026");
        assert_eq!(report.total_rounded_minutes, 60);
        assert_eq!(report.columns.len(), 30);
        assert_eq!(report.rows[0].short_name, "ACM");
        assert_eq!(report.rows[0].daily_minutes[0], 60);
        assert!(report.rows[0].daily_minutes.iter().skip(1).all(|minutes| *minutes == 0));
        assert!(!report.rows[0].color.is_empty());
    }

    #[test]
    fn client_short_names_use_initials_or_prefix() {
        assert_eq!(client_short_name("Acme"), "ACM");
        assert_eq!(client_short_name("Northwind Traders"), "NT");
        assert_eq!(client_short_name("A B C Foods"), "ABC");
    }

    #[test]
    fn dst_spring_forward_session_keeps_local_start_date() {
        let conn = db::open_in_memory().unwrap();
        let acme = client_service::create_client(&conn, "Acme", None).unwrap();
        insert_session(
            &conn,
            acme.id,
            at("2026-03-08T01:30:00-05:00"),
            at("2026-03-08T03:30:00-04:00"),
            None,
        )
        .unwrap();

        let eastern = FixedOffset::west_opt(5 * 3600).unwrap();
        let report = get_daily_totals(&conn, date(2026, 3, 8), &eastern).unwrap();
        assert_eq!(report.total_rounded_minutes, 60);
        let previous = get_daily_totals(&conn, date(2026, 3, 7), &eastern).unwrap();
        assert_eq!(previous.total_rounded_minutes, 0);
    }
}
