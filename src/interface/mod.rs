use std::collections::HashMap;
use std::result::Result as StdResult;

use chrono::Local;
use rand::{self, Rng};
use reqwest::blocking::{Client, RequestBuilder, Response};
use reqwest::header::{ACCEPT, CONTENT_TYPE};
use reqwest::header::{HeaderMap, HeaderValue};
use reqwest::IntoUrl;
use serde::Serialize;
use wall_street::derivative::Derivative;

use crate::error::Error;
use crate::serde::{JsonResponseValue, JsonResponseValues};
use crate::session::{GrantType, PreSession, Session, SessionId, SessionStatus};
use crate::session::tan::{TanChallenge, TanChallengeType};
use crate::types::cost_indication::{ChangeCostIndication, CostIndication, RawCostIndication};
use crate::types::deposit::ComdirectDeposit;
use crate::types::instrument::Instrument;
use crate::types::market_place::{JsonResponseMarketplaces, MarketPlace, MarketPlaceFilterParameters};
use crate::types::order::{Order, OrderFilterParameters, OrderId, RawOrder};
use crate::types::order::order_change::{DeleteOrder, OrderChange, OrderChangeAction, OrderChangeValidation};
use crate::types::order::order_outline::OrderOutline;
use crate::types::position::{Position, PositionId, RawPosition};
use crate::types::quote::{Quote, QuoteOutline, QuoteTicket, RawQuote};
use crate::types::quote::order_outline::QuoteOrderOutline;
use crate::types::transaction::{RawTransaction, Transaction, TransactionFilterParameters};

const HEX_CHARSET: &[u8] = b"0123456789abcdef";

type Result<T> = StdResult<T, Error>;

macro_rules! url {
    ($path:literal) => (concat!("https://api.comdirect.de/api", $path));
}

macro_rules! session_is_active {
    (*$session:expr) => {
        match $session {
            Some(session) => session,
            None => return Err(Error::NoActiveSession)
        }
    };
    ($session:expr) => {
        match $session {
            Some(ref session) => session,
            None => return Err(Error::NoActiveSession)
        }
    };
}

macro_rules! tan_is_free {
    ($tan:expr) => {
        if *$tan.typ() != TanChallengeType::Free {
            return Err(Error::UnexpectedTanType);
        }
    };
}

macro_rules! session_request_method {
    ($method_name:ident, $method:ident) => {
        #[inline(always)]
        fn $method_name<U: IntoUrl>(&self, url: U, session: &Session) -> RequestBuilder {
            self.client
                .$method(url)
                .bearer_auth(&session.access_token)
                .header("x-http-request-info", self.make_request_info(&session.session_id))
        }
    };
}

pub mod session;
pub mod deposit;
pub mod instrument;
pub mod order;
pub mod quote;

new_type_ids!(
    #[derive(derive_more::From)]
    pub struct ClientId
    #[derive(derive_more::From)]
    pub struct ClientSecret
    #[derive(derive_more::From)]
    pub struct Username
    #[derive(derive_more::From)]
    pub struct Password
);

pub struct ApiClient {
    client_id: ClientId,
    client_secret: ClientSecret,
    username: Username,
    password: Password,

    client: Client,
    session: Option<Session>,
}

impl Drop for ApiClient {
    fn drop(&mut self) {
        let _ = self.end_session();
    }
}

impl ApiClient {
    pub fn new(client_id: ClientId, client_secret: ClientSecret, username: Username, password: Password) -> Self {
        Self {
            client_id,
            client_secret,
            username,
            password,
            client: Self::default_client(),
            session: None,
        }
    }

    #[inline(always)]
    fn default_client() -> Client {
        Client::builder()
            .cookie_store(true)
            .default_headers(Self::default_header_map())
            .build()
            .expect("could not build the client")
    }

    #[inline(always)]
    fn default_header_map() -> HeaderMap<HeaderValue> {
        let mut default_header = HeaderMap::new();
        default_header.insert(ACCEPT, HeaderValue::from_static("application/json"));
        default_header.insert(CONTENT_TYPE, HeaderValue::from_static("application/json"));
        default_header
    }

    #[inline(always)]
    fn make_request_info(&self, session_id: &SessionId) -> String {
        format!(
            r#"{{"clientRequestId":{{"sessionId":"{}","requestId":"{}"}}}}"#,
            session_id.as_str(),
            ApiClient::make_request_id()
        )
    }

    #[inline(always)]
    fn make_request_id() -> String {
        Local::now().format("%H%M%S%3f").to_string()
    }

    #[inline(always)]
    fn extract_tan_challenge(header_map: &HeaderMap) -> Result<TanChallenge> {
        let authentication_info = header_map
            .get("x-once-authentication-info")
            .ok_or(Error::UnexpectedResponseHeaders)?
            .to_str()
            .map_err(|_| Error::UnexpectedResponseHeaders)?;
        Ok(serde_json::from_str(authentication_info)?)
    }

    #[inline(always)]
    fn make_x_authentication_info_header(tan_challenge: &TanChallenge) -> (&'static str, String) {
        ("x-once-authentication-info", format!(r#"{{"id":"{}"}}"#, tan_challenge.id().as_str()))
    }

    session_request_method!(make_get_session_request, get);
    session_request_method!(make_post_session_request, post);
    session_request_method!(make_patch_session_request, patch);
    session_request_method!(make_delete_session_request, delete);
}
