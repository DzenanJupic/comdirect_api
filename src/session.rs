use chrono::{DateTime, Local};

#[derive(Clone)]
pub(crate) struct Session {
    pub(crate) session_id: String,
    pub(crate) session_uuid: String,
    pub(crate) access_token: String,
    pub(crate) refresh_token: String,
    pub(crate) refresh_token_expires_at: DateTime<Local>,
}

pub(crate) struct PreSession {
    pub(crate) access_token: String,
    pub(crate) refresh_token: String,
    pub(crate) refresh_token_expires_at: DateTime<Local>,
}

impl PreSession {
    pub fn session(self, session_id: String, session_uuid: String) -> Session {
        Session {
            session_id,
            session_uuid,
            access_token: self.access_token,
            refresh_token: self.refresh_token,
            refresh_token_expires_at: self.refresh_token_expires_at,
        }
    }
}

pub(crate) enum GrantType {
    Password,
    CdSecondary(String),
    Refresh(String),
}

impl GrantType {
    pub const fn to_str(&self) -> &'static str {
        match self {
            GrantType::Password => "password",
            GrantType::CdSecondary(_) => "cd_secondary",
            GrantType::Refresh(_) => "refresh_token"
        }
    }
}
