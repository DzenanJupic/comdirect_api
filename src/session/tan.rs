use serde::Deserialize;

new_type_ids!(
    pub struct TanChallengeId
);

#[derive(Debug, Deserialize, getset::Getters)]
#[getset(get = "pub(crate)")]
pub(crate) struct TanChallenge {
    id: TanChallengeId,
    typ: TanChallengeType,
    #[serde(rename = "availableTypes")]
    available_types: Vec<TanChallengeType>,
    challenge: Option<String>,
}

#[derive(Clone, Copy, Debug, Deserialize, PartialEq)]
pub(crate) enum TanChallengeType {
    #[serde(rename = "P_TAN_PUSH")]
    PushTan,
    #[serde(rename = "P_TAN")]
    PhotoTan,
    #[serde(rename = "P_TAN_APP")]
    PhotoTanApp,
    #[serde(rename = "M_TAN")]
    MobileTan,
    #[serde(rename = "TAN_FREI")]
    Free,
}

impl TanChallengeType {
    pub const fn to_authentication_info(&self) -> &'static str {
        use TanChallengeType::*;
        match self {
            PushTan => r#"{"typ":"P_TAN_PUSH"}"#,
            PhotoTan => r#"{"typ":"P_TAN"}"#,
            PhotoTanApp => r#"{"typ":"P_TAN_APP"}"#,
            MobileTan => r#"{"typ":"M_TAN"}"#,
            Free => r#"{"typ":"TAN_FREI"}"#,
        }
    }
}
