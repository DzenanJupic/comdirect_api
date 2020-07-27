#![feature(const_if_match)]

use std::collections::HashMap;
use std::error::Error;
use std::str::FromStr;
#[cfg(test)]
use std::thread::sleep;

use chrono::{Duration, Local};
use rand::{self, Rng};
use reqwest::{
    blocking::Client,
    header::{ACCEPT, CONTENT_TYPE},
};
use reqwest::header::{HeaderMap, HeaderValue};
use serde_json::Value;

use error::*;
use session::*;
use tan::*;

pub mod error;
pub(crate) mod tan;
pub(crate) mod session;

const HEX_CHARSET: &[u8] = b"0123456789abcdef";

macro_rules! url {
    ($path:literal) => (concat!("https://api.comdirect.de/api", $path));
}

macro_rules! response_is_success {
    ($response:ident) => {
        if !$response.status().is_success() {
            #[cfg(test)]
            println!("response: {:?}\nheaders: {:?}", $response, $response.headers());
            return Err(Box::new(ClientError::NoActiveSession));
        }
    };
}

#[derive(Clone)]
pub struct Comdirect {
    client_id: String,
    client_secret: String,
    username: String,
    password: String,

    client: Client,
    session: Option<Session>,
}

impl Comdirect {
    pub fn new(
        client_id: String,
        client_secret: String,
        username: String,
        password: String,
    ) -> Self {
        let mut default_header = HeaderMap::new();
        default_header.insert(ACCEPT, HeaderValue::from_static("application/json"));
        default_header.insert(CONTENT_TYPE, HeaderValue::from_static("application/json"));

        let client = Client::builder()
            .cookie_store(true)
            .default_headers(default_header)
            .build()
            .expect("could not build the client");

        Self {
            client_id,
            client_secret,
            username,
            password,
            client,
            session: None,
        }
    }

    pub fn new_session(&mut self) -> Result<(), Box<dyn Error>> {
        let pre_session = self.acquire_oauth_token(GrantType::Password)?;
        let mut session = self.acquire_session_status(pre_session)?;
        let tan_challenge = self.request_tan_challenge(&session)?;
        self.activate_tan(&session, tan_challenge)?;
        let secondary_session = self.acquire_oauth_token(GrantType::CdSecondary(session.access_token))?;

        session.access_token = secondary_session.access_token;
        session.refresh_token = secondary_session.refresh_token;
        session.refresh_token_expires_at = secondary_session.refresh_token_expires_at;

        self.session = Some(session);

        Ok(())
    }

    pub fn refresh_session(&mut self) -> Result<(), Box<dyn Error>> {
        match self.session.is_some() {
            true => {
                let mut session = self.session.take().unwrap();
                let refresh_session = self.acquire_oauth_token(GrantType::Refresh(session.refresh_token))?;

                session.access_token = refresh_session.access_token;
                session.refresh_token = refresh_session.refresh_token;
                session.refresh_token_expires_at = refresh_session.refresh_token_expires_at;

                self.session = Some(session);

                Ok(())
            }
            false => Err(Box::new(ClientError::NoActiveSession))
        }
    }

    pub fn end_session(&mut self) -> Result<(), Box<dyn Error>> {
        self.revoke_oauth_token()?;
        self.session = None;

        Ok(())
    }

    fn acquire_oauth_token(&self, grant_type: GrantType) -> Result<PreSession, Box<dyn Error>> {
        const URL: &str = "https://api.comdirect.de/oauth/token";

        let mut params: HashMap<_, &str> = HashMap::new();
        params.insert("client_id", &self.client_id);
        params.insert("client_secret", &self.client_secret);
        params.insert("grant_type", grant_type.to_str());

        match grant_type {
            GrantType::Password => {
                params.insert("username", &self.username);
                params.insert("password", &self.password);
            }
            GrantType::CdSecondary(ref access_token) => {
                params.insert("token", access_token);
            }
            GrantType::Refresh(ref refresh_token) => {
                params.insert("refresh_token", refresh_token);
            }
        }

        let response = self.client
                           .post(URL)
                           .header(CONTENT_TYPE, "application/x-www-form-urlencoded")
                           .form(&params)
                           .send()?;

        response_is_success!(response);

        let json: Value = response.json()?;

        // extract the access-token, refresh-token, and the refresh-token expiration time from the
        // response body
        let access_token = json
            .get("access_token")
            .ok_or(ClientError::UnexpectedResponseType)?
            .as_str()
            .ok_or(ClientError::UnexpectedResponseType)?
            .to_string();
        let refresh_token = json
            .get("refresh_token")
            .ok_or(ClientError::UnexpectedResponseType)?
            .as_str()
            .ok_or(ClientError::UnexpectedResponseType)?
            .to_string();
        let refresh_token_expires_in = Duration::seconds(
            json
                .get("expires_in")
                .ok_or(ClientError::UnexpectedResponseType)?
                .as_i64()
                .ok_or(ClientError::UnexpectedResponseType)?
        );

        Ok(PreSession {
            access_token,
            refresh_token,
            refresh_token_expires_at: Local::now() + refresh_token_expires_in,
        })
    }

    fn revoke_oauth_token(&self) -> Result<(), Box<dyn Error>> {
        const URL: &str = "https://api.comdirect.de/oauth/revoke";

        match self.session {
            Some(ref session) => {
                let request = self.client
                                  .delete(URL)
                                  .bearer_auth(&session.access_token)
                                  .build()?;

                let response = self.client.execute(request)?;
                response_is_success!(response);

                Ok(())
            }
            None => Err(Box::new(ClientError::NoActiveSession))
        }
    }

    fn acquire_session_status(&self, pre_session: PreSession) -> Result<Session, Box<dyn Error>> {
        const URL: &str = url!("/session/clients/user/v1/sessions");

        // generate a random 32 character long session id
        let mut rng = rand::thread_rng();
        let session_id: String = (0..32)
            .map(|_| {
                let char_id = rng.gen_range(0, HEX_CHARSET.len());
                HEX_CHARSET[char_id] as char
            })
            .collect();

        let response = self.client
                           .get(URL)
                           .bearer_auth(&pre_session.access_token)
                           .header("x-http-request-info", self.make_request_info(&session_id))
                           .send()?;

        response_is_success!(response);

        let json: Value = response.json()?;

        let session_uuid = json
            .get(0)
            .ok_or(ClientError::UnexpectedResponseType)?
            .get("identifier")
            .ok_or(ClientError::UnexpectedResponseType)?
            .as_str()
            .ok_or(ClientError::UnexpectedResponseType)?
            .to_string();

        Ok(pre_session.session(session_id, session_uuid))
    }

    fn request_tan_challenge(&self, session: &Session) -> Result<TanChallenge, Box<dyn Error>> {
        let url = format!("{}/{}/validate", url!("/session/clients/user/v1/sessions"), session.session_uuid);
        let data = format!(
            r#"{{
                "identifier": "{}",
                "sessionTanActive":true,
                "activated2FA":true
            }}"#,
            session.session_uuid
        );

        let response = self.client
                           .post(&url)
                           .bearer_auth(&session.access_token)
                           .header("x-http-request-info", self.make_request_info(&session.session_id))
                           .body(data)
                           .send()?;

        response_is_success!(response);

        let authentication_info = response
            .headers()
            .get("x-once-authentication-info")
            .ok_or(ClientError::UnexpectedResponseType)?
            .to_str()?;

        let json: Value = serde_json::from_str(authentication_info)?;

        let typ = TanChallengeType::from_str(
            json
                .get("typ")
                .ok_or(ClientError::UnexpectedResponseType)?
                .as_str()
                .ok_or(ClientError::UnexpectedResponseType)?
        )?;

        if typ != TanChallengeType::PushTan {
            /*json
                .get("availableTypes")
                .ok_or(ClientError::UnexpectedResponseType)?
                .as_array()
                .ok_or(ClientError::UnexpectedResponseType)?
                .iter()
                .find(|value| {
                    match value.as_str() {
                        Some(value) => value == tan_challenges::PUSH_TAN,
                        None => false,
                    }
                })
                .ok_or(ClientError::UnexpectedResponseType)?;

            let challenge = json
                .get("challenge")
                .ok_or(ClientError::UnexpectedResponseType)?
                .as_str()
                .ok_or(ClientError::UnexpectedResponseType)?
                .to_string();*/
            // todo: if tan challenge type is not push tan, request push tan
            // todo: support other tan challenge types
            return Err(Box::new(ClientError::UnexpectedResponseType));
        }

        let id = json
            .get("id")
            .ok_or(ClientError::UnexpectedResponseType)?
            .as_str()
            .ok_or(ClientError::UnexpectedResponseType)?
            .to_string();

        Ok(TanChallenge {
            id,
        })
    }

    fn activate_tan(&self, session: &Session, tan_challenge: TanChallenge) -> Result<(), Box<dyn Error>> {
        #[cfg(test)]
            sleep(std::time::Duration::from_secs(10));
        #[cfg(not(test))]
        println!("Please activate the session tan in your photo tan app\nThen press enter");
        #[cfg(not(test))]
            std::io::stdin()
            .read_line(&mut String::with_capacity(2))?;

        let url = format!("{}/{}", url!("/session/clients/user/v1/sessions"), session.session_uuid);
        let tan_id = format!(r#"{{"id":"{}"}}"#, tan_challenge.id);
        let data = format!(
            r#"{{
                "identifier": "{}",
                "sessionTanActive":true,
                "activated2FA":true
            }}"#,
            session.session_uuid
        );

        let response = self.client
                           .patch(&url)
                           .bearer_auth(&session.access_token)
                           .header("x-http-request-info", self.make_request_info(&session.session_id))
                           .header("x-once-authentication-info", tan_id)
            // if tan challenge would not be push tan: .header("x-once-authentication", <TAN>)
                           .body(data)
                           .send()?;

        response_is_success!(response);

        Ok(())
    }

    fn make_request_id() -> String {
        Local::now().format("%H%M%S%.3f").to_string()
    }

    fn make_request_info(&self, session_id: &str) -> String {
        format!(
            r#"{{"clientRequestId":{{"sessionId":"{}","requestId":"{}"}}}}"#,
            session_id,
            Comdirect::make_request_id()
        )
    }
}

#[cfg(test)]
mod tests {
    use crate::*;

    #[test]
    fn session() {
        //! !open Photo tan app before testing!
        //! you'll have 10 seconds to activate the push tan

        let mut comdirect = Comdirect::new(
            env!("client_id").to_string(),
            env!("client_secret").to_string(),
            env!("username").to_string(),
            env!("password").to_string(),
        );

        comdirect
            .new_session()
            .unwrap();

        sleep(std::time::Duration::from_secs(10));

        comdirect
            .refresh_session()
            .unwrap();

        sleep(std::time::Duration::from_secs(10));

        comdirect
            .end_session()
            .unwrap();
    }
}
