#[derive(Debug)]
pub enum Error {
    // ProcessError(String),
    Sqlx(sqlx::Error),

    Serde(serde_json::Error),

    Policy(crate::infra::policy::Error),

    UnknownEvent(String),
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Error::Sqlx(error) => write!(f, "{}", error),
            Error::Serde(error) => write!(f, "{}", error),
            Error::Policy(error) => write!(f, "{}", error),
            Error::UnknownEvent(s) => write!(f, "{}", s),
        }
    }
}

impl From<sqlx::Error> for Error {
    fn from(value: sqlx::Error) -> Self {
        tracing::error!("{}", value);
        Self::Sqlx(value)
    }
}

impl From<serde_json::Error> for Error {
    fn from(value: serde_json::Error) -> Self {
        tracing::error!("{}", value);
        Self::Serde(value)
    }
}

impl From<crate::infra::policy::Error> for Error {
    fn from(value: crate::infra::policy::Error) -> Self {
        tracing::warn!("{}", value);
        Self::Policy(value)
    }
}

impl From<String> for Error {
    fn from(value: String) -> Self {
        tracing::warn!("{}", value);
        Self::UnknownEvent(value)
    }
}
