use std::str::FromStr;

use crate::error::ResponseError;

pub(crate) struct TanChallenge {
    pub(crate) id: String,
    // pub(crate) typ: TanChallengeType
    // pub(crate) available_types: Vec<String>
    // pub(crate) challenge: Option<String>
}

#[derive(PartialEq)]
pub(crate) enum TanChallengeType {
    PushTan,
    PhotoTan,
}

impl FromStr for TanChallengeType {
    type Err = ResponseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "P_TAN_PUSH" => Ok(TanChallengeType::PushTan),
            "P_TAN" => Ok(TanChallengeType::PhotoTan),
            _ => Err(ResponseError::UnexpectedResponseValue)
        }
    }
}