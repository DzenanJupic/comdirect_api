use chrono::{DateTime, Local};
use serde::Deserialize;

#[doc(hidden)]
pub(crate) mod tan;

new_type_ids!(
    pub(crate) struct SessionId
    pub(crate) struct SessionUuid
    pub(crate) struct AccessToken
    pub(crate) struct RefreshToken
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

impl Session {
    pub(crate) fn from_pre_session(pre_session: PreSession, session_id: SessionId, session_uuid: SessionUuid)
        -> Self {
        Self {
            session_id,
            session_uuid,
            access_token: pre_session.access_token,
            refresh_token: pre_session.refresh_token,
            expires_at: pre_session.expires_at,
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

pub(crate) enum GrantType<'t> {
    Password,
    CdSecondary(&'t AccessToken),
    Refresh(&'t RefreshToken),
}

impl Session {
    #[inline(always)]
    pub(crate) fn update(&mut self, pre_session: PreSession) {
        self.access_token = pre_session.access_token;
        self.refresh_token = pre_session.refresh_token;
        self.expires_at = pre_session.expires_at;
    }
}

impl SessionStatus {
    pub(crate) fn take_session_uuid(self) -> SessionUuid {
        self.identifier
    }
}

impl GrantType<'_> {
    pub const fn as_str(&self) -> &'static str {
        match self {
            GrantType::Password => "password",
            GrantType::CdSecondary(_) => "cd_secondary",
            GrantType::Refresh(_) => "refresh_token"
        }
    }
}
