use derive_more::{Display, Error as DeriveError};
use reqwest::StatusCode;

#[derive(Copy, Clone, Debug, Display, PartialEq, DeriveError)]
pub enum Error {
    NoActiveSession,
    InvalidTan,
    /// indicates that the 
    UnsupportedTanType,
    /// will be used if either a requested tan type was not delivered, or when an active
    /// session tan exists, but the response header indicates that the tan type is not Free
    UnexpectedTanType,
    CouldNotCreateSession,
    CouldNotEndSession,

    UnexpectedResponseHeaders,
    UnexpectedJsonValues,
    ResponseClientError,
    ResponseServerError,
    NotFound,
    UnprocessableRequest,

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
        dbg!(&error);
        match error.status() {
            Some(StatusCode::NOT_FOUND) => Self::NotFound,
            Some(StatusCode::UNPROCESSABLE_ENTITY) => Self::UnprocessableRequest,
            Some(s) if s.is_client_error() => Self::ResponseClientError,
            Some(s) if s.is_server_error() => Self::ResponseServerError,
            _ => Self::Other
        }
    }
}

impl From<serde_json::Error> for Error {
    fn from(e: serde_json::Error) -> Self {
        println!("serde error: {}", e);
        Self::UnexpectedJsonValues
    }
}

impl From<std::io::Error> for Error {
    fn from(_: std::io::Error) -> Self {
        Self::IOError
    }
}

