use std::error::Error;
use std::fmt;

#[derive(Debug)]
pub enum ClientError {
    NoActiveSession,
    UnexpectedResponseType,
    CouldNotCreateSession,
    CouldNotEndSession,
}

impl fmt::Display for ClientError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "{:?}", self)
    }
}

impl Error for ClientError {}

#[derive(Debug)]
pub enum ResponseError {
    UnexpectedResponseValue
}

impl fmt::Display for ResponseError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "{:?}", self)
    }
}

impl Error for ResponseError {}
