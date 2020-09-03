use derive_more::{Display, Error as DeriveError};
use reqwest::StatusCode;

#[derive(Copy, Clone, Debug, Display, DeriveError)]
pub enum Error {
    NoActiveSession,
    InvalidTan,
    UnsupportedTanType,
    CouldNotCreateSession,
    CouldNotEndSession,

    UnexpectedResponseHeaders,
    UnexpectedJsonValues,
    ResponseClientError,
    ResponseServerError,
    NotFound,
    UnprocessedRequest,

    NotSupported,
    IOError,

    Other,
}

impl From<()> for Error {
    fn from(_: ()) -> Self { Self::Other }
}

impl From<reqwest::Error> for Error {
    fn from(error: reqwest::Error) -> Self {
        #[cfg(any(test, feature = "test"))]
        eprintln!("reqwest Error: `{}`", error);
        match error.status() {
            Some(StatusCode::NOT_FOUND) => Self::NotFound,
            Some(StatusCode::UNPROCESSABLE_ENTITY) => Self::UnprocessedRequest,
            Some(s) if s.is_client_error() => Self::ResponseClientError,
            Some(s) if s.is_server_error() => Self::ResponseServerError,
            _ => Self::Other
        }
    }
}

impl From<serde_json::Error> for Error {
    fn from(_: serde_json::Error) -> Self {
        Self::UnexpectedJsonValues
    }
}

impl From<std::io::Error> for Error {
    fn from(_: std::io::Error) -> Self {
        Self::IOError
    }
}

