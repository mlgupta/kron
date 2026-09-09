use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Client {
    pub id: i64,
    pub name: String,
    pub description: Option<String>,
    pub active: bool,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ActiveTimer {
    pub client_id: i64,
    pub client_name: String,
    pub started_at: String,
    pub note: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct TimeSession {
    pub id: i64,
    pub client_id: i64,
    pub client_name: String,
    pub started_at: String,
    pub stopped_at: String,
    pub actual_duration_seconds: i64,
    pub rounded_duration_minutes: i64,
    pub note: Option<String>,
    pub created_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ReportRow {
    pub client_id: i64,
    pub client_name: String,
    pub rounded_minutes: i64,
    #[serde(default)]
    pub daily_minutes: Vec<i64>,
    #[serde(default)]
    pub short_name: String,
    #[serde(default)]
    pub color: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Report {
    pub period: String,
    pub label: String,
    pub start_date: String,
    pub end_date: String,
    pub previous_date: String,
    pub next_date: String,
    pub rows: Vec<ReportRow>,
    pub total_rounded_minutes: i64,
    #[serde(default)]
    pub columns: Vec<String>,
}
