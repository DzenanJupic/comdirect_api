use derive_more::{Display, Error as DeriveError};

#[derive(Copy, Clone, Debug, Display, DeriveError)]
pub enum Error {
    NoActiveSession,
    InvalidTan,
    CouldNotCreateSession,
    CouldNotEndSession,
    UnexpectedResponseHeaders,
    UnsupportedTanType,
    UnexpectedJsonValues,
    ResponseError,
    IOError,
    Other,
}

impl From<()> for Error {
    fn from(_: ()) -> Self { Self::Other }
}

impl From<reqwest::Error> for Error {
    fn from(error: reqwest::Error) -> Self {
        if error.is_status() {
            Self::ResponseError
        } else {
            #[cfg(any(test, feature = "test"))]
            println!("reqwest Error: `{}`", error);
            Self::Other
        }
    }
}

impl From<reqwest::header::ToStrError> for Error {
    fn from(_: reqwest::header::ToStrError) -> Self {
        Self::UnexpectedResponseHeaders
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

