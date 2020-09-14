use chrono::{DateTime, Local};
use serde::Deserialize;

#[doc(hidden)]
pub(crate) mod tan;

new_type_ids!(
    pub struct SessionId
    pub struct SessionUuid
    pub struct AccessToken
    pub struct RefreshToken
);

#[derive(Clone, Debug, PartialEq)]
pub(crate) struct Session {
    pub(crate) session_id: SessionId,
    pub(crate) session_uuid: SessionUuid,
    pub(crate) access_token: AccessToken,
    pub(crate) refresh_token: RefreshToken,
    pub(crate) expires_at: DateTime<Local>,
}

#[derive(Debug, Deserialize)]
pub(crate) struct PreSession {
    pub(crate) access_token: AccessToken,
    pub(crate) refresh_token: RefreshToken,
    #[serde(rename = "expires_in")]
    #[serde(with = "crate::serde::date::seconds")]
    pub(crate) expires_at: DateTime<Local>,
}

impl PreSession {
    pub fn session(self, session_id: SessionId, session_uuid: SessionUuid) -> Session {
        Session {
            session_id,
            session_uuid,
            access_token: self.access_token,
            refresh_token: self.refresh_token,
            expires_at: self.expires_at,
        }
    }
}

#[derive(Debug, Deserialize, getset::CopyGetters)]
pub(crate) struct SessionStatus {
    identifier: SessionUuid,
    #[serde(rename = "sessionTanActive")]
    #[getset(get_copy = "pub(crate)")]
    tan_is_active: bool,
}

pub(crate) enum GrantType {
    Password,
    CdSecondary(AccessToken),
    Refresh(RefreshToken),
}

impl SessionStatus {
    pub(crate) fn take_session_uuid(self) -> SessionUuid {
        self.identifier
    }
}

impl GrantType {
    pub const fn as_str(&self) -> &'static str {
        match self {
            GrantType::Password => "password",
            GrantType::CdSecondary(_) => "cd_secondary",
            GrantType::Refresh(_) => "refresh_token"
        }
    }
}
