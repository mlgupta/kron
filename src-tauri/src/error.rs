use serde::Serialize;

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct AppError {
    pub code: String,
    pub message: String,
}

impl AppError {
    pub fn new(code: impl Into<String>, message: impl Into<String>) -> Self {
        Self {
            code: code.into(),
            message: message.into(),
        }
    }

    pub fn validation(message: impl Into<String>) -> Self {
        Self::new("VALIDATION", message)
    }

    pub fn not_found(message: impl Into<String>) -> Self {
        Self::new("NOT_FOUND", message)
    }

    pub fn duplicate_name() -> Self {
        Self::new(
            "DUPLICATE_NAME",
            "A client with this name already exists.",
        )
    }

    pub fn client_has_sessions() -> Self {
        Self::new(
            "CLIENT_HAS_SESSIONS",
            "This client has recorded time sessions. Confirm deletion to remove the client and its history, or deactivate the client instead.",
        )
    }

    pub fn timer_already_running() -> Self {
        Self::new(
            "TIMER_ALREADY_RUNNING",
            "A timer is already running. Stop it or confirm switching clients first.",
        )
    }

    pub fn timer_not_running() -> Self {
        Self::new(
            "TIMER_NOT_RUNNING",
            "No timer is running.",
        )
    }

    pub fn client_inactive() -> Self {
        Self::new(
            "CLIENT_INACTIVE",
            "That client is inactive. Activate it before starting a timer.",
        )
    }

    pub fn invalid_date_range() -> Self {
        Self::new(
            "INVALID_DATE_RANGE",
            "The start date must be on or before the end date.",
        )
    }

    pub fn invalid_session() -> Self {
        Self::new(
            "INVALID_SESSION",
            "The session end time must be after the start time.",
        )
    }

    pub fn database() -> Self {
        Self::new(
            "DATABASE",
            "Something went wrong while saving your data. Please try again.",
        )
    }

    pub fn to_invoke_error(&self) -> String {
        serde_json::to_string(self).unwrap_or_else(|_| self.message.clone())
    }
}

impl std::fmt::Display for AppError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.message)
    }
}

impl std::error::Error for AppError {}

impl From<rusqlite::Error> for AppError {
    fn from(error: rusqlite::Error) -> Self {
        log_technical_error("sqlite", &error);
        match error {
            rusqlite::Error::SqliteFailure(info, _)
                if info.code == rusqlite::ErrorCode::ConstraintViolation =>
            {
                AppError::duplicate_name()
            }
            _ => AppError::database(),
        }
    }
}

pub fn log_technical_error(context: &str, error: &impl std::fmt::Display) {
    eprintln!("[Kron] {context}: {error}");
}
