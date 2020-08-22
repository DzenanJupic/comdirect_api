use std::collections::HashMap;

use chrono::Local;
use rand::{self, Rng};
use reqwest::{
    blocking::Client,
    header::{ACCEPT, CONTENT_TYPE},
};
use reqwest::blocking::RequestBuilder;
use reqwest::header::{HeaderMap, HeaderValue};
use stock_market_utils::deposit::Deposit;
use stock_market_utils::derivative::Derivative;

use crate::deposit::ComdirectDeposit;
use crate::error::Error;
use crate::instrument::Instrument;
use crate::market_place::{MarketPlace, OrderDimensionsFilterParameters};
use crate::order::{ComdirectOrder, OrderFilterParameters, RawComdirectOrder};
use crate::position::{Position, PositionId, RawPosition};
use crate::serde::JsonResponseValues;
use crate::serde::market_place::JsonResponseMarketplaces;
use crate::session::{GrantType, PreSession, Session, SessionId, SessionStatus};
use crate::tan::{TanChallenge, TanChallengeType};
use crate::transaction::{RawTransaction, Transaction, TransactionFilterParameters};

const HEX_CHARSET: &[u8] = b"0123456789abcdef";

macro_rules! url {
    ($path:literal) => (concat!("https://api.comdirect.de/api", $path));
}

macro_rules! response_is_success {
    ($response:ident) => (response_is_success!($response, return Err(Error::ResponseError)););
    ($response:ident, $error_handle:stmt) => {
        if !$response.status().is_success() {
            #[cfg(any(test, feature = "test"))]
            println!("response: {:?}\nheaders: {:?}", $response, $response.headers());
            #[cfg(any(test, feature = "test"))]
            println!("text: {:?}", $response.text());
            $error_handle
        }
    };
}

macro_rules! session_is_active {
    ($session:expr) => {
        match $session {
            Some(ref session) => session,
            None => return Err(Error::NoActiveSession)
        }
    };
}

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

#[derive(Clone, getset::Getters)]
#[getset(get = "pub")]
pub struct Comdirect {
    client_id: ClientId,
    client_secret: ClientSecret,
    username: Username,
    password: Password,

    client: Client,
    #[getset(get)]
    session: Option<Session>,
}

impl Drop for Comdirect {
    fn drop(&mut self) {
        let _ = self.end_session();
    }
}

impl Comdirect {
    pub fn new(client_id: ClientId, client_secret: ClientSecret, username: Username, password: Password) -> Self {
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

    pub fn new_session(&mut self) -> Result<(), Error> {
        let pre_session = self.acquire_oauth_token(GrantType::Password)?;
        let mut session = self.acquire_session_status(pre_session)?;
        let tan_challenge = self.request_tan_challenge(&session, None)?;
        self.activate_tan(&session, tan_challenge)?;
        let secondary_session = self.acquire_oauth_token(GrantType::CdSecondary(session.access_token))?;

        session.access_token = secondary_session.access_token;
        session.refresh_token = secondary_session.refresh_token;
        session.expires_at = secondary_session.expires_at;

        self.session = Some(session);

        Ok(())
    }

    pub fn refresh_session(&mut self) -> Result<(), Error> {
        match self.session.is_some() {
            true => {
                let mut session = self.session.take().unwrap();
                let refresh_session = self.acquire_oauth_token(GrantType::Refresh(session.refresh_token))?;

                session.access_token = refresh_session.access_token;
                session.refresh_token = refresh_session.refresh_token;
                session.expires_at = refresh_session.expires_at;

                self.session = Some(session);

                Ok(())
            }
            false => Err(Error::NoActiveSession)
        }
    }

    pub fn end_session(&mut self) -> Result<(), Error> {
        self.revoke_oauth_token()?;
        self.session = None;

        Ok(())
    }

    fn acquire_oauth_token(&self, grant_type: GrantType) -> Result<PreSession, Error> {
        const URL: &str = "https://api.comdirect.de/oauth/token";

        let mut params: HashMap<_, &str> = HashMap::new();
        params.insert("client_id", self.client_id().as_str());
        params.insert("client_secret", self.client_secret().as_str());
        params.insert("grant_type", grant_type.as_str());

        match grant_type {
            GrantType::Password => {
                params.insert("username", self.username().as_str());
                params.insert("password", self.password().as_str());
            }
            GrantType::CdSecondary(ref access_token) => {
                params.insert("token", access_token.as_str());
            }
            GrantType::Refresh(ref refresh_token) => {
                params.insert("refresh_token", refresh_token.as_str());
            }
        }

        let response = self.client
            .post(URL)
            .header(CONTENT_TYPE, "application/x-www-form-urlencoded")
            .form(&params)
            .send()?;
        response_is_success!(response);

        Ok(response.json::<PreSession>()?)
    }

    fn revoke_oauth_token(&self) -> Result<(), Error> {
        const URL: &str = "https://api.comdirect.de/oauth/revoke";

        let session = session_is_active!(self.session);
        let request = self.client
            .delete(URL)
            .bearer_auth(&session.access_token)
            .build()?;

        let response = self.client.execute(request)?;
        response_is_success!(response);

        Ok(())
    }

    fn acquire_session_status(&self, pre_session: PreSession) -> Result<Session, Error> {
        const URL: &str = url!("/session/clients/user/v1/sessions");

        let session_id = Self::make_session_id();

        let response = self.client
            .get(URL)
            .bearer_auth(&pre_session.access_token)
            .header("x-http-request-info", self.make_request_info(&session_id))
            .send()?;
        response_is_success!(response);

        let session_status = response.json::<(SessionStatus, )>()?.0;

        Ok(pre_session.session(session_id, session_status.take_session_uuid()))
    }

    fn request_tan_challenge(&self, session: &Session, request_tan_type: Option<TanChallengeType>) -> Result<TanChallenge, Error> {
        let url = format!("{}/{}/validate", url!("/session/clients/user/v1/sessions"), session.session_uuid.as_str());
        let data = format!(
            r#"{{
                "identifier": "{}",
                "sessionTanActive":true,
                "activated2FA":true
            }}"#,
            session.session_uuid.as_str()
        );

        let mut request = self.make_post_session_request(&url, &session);

        if let Some(tan_type) = request_tan_type {
            request = request.header("x-once-authentication-info", tan_type.to_authentication_info());
        }

        let response =
            request
                .body(data)
                .send()?;
        response_is_success!(response);

        let authentication_info = response
            .headers()
            .get("x-once-authentication-info")
            .ok_or(Error::UnexpectedResponseHeaders)?
            .to_str()?;

        let tan_challenge: TanChallenge = serde_json::from_str(authentication_info)?;

        match tan_challenge.typ() {
            TanChallengeType::PushTan => {}
            _ => {
                // todo: support other tan challenge types then PushTan
                // to prevent loops, just request another type, if the current function call was not
                // already a type request
                if request_tan_type.is_none() {
                    if tan_challenge.available_types().contains(&TanChallengeType::PushTan) {
                        let tan_challenge = self.request_tan_challenge(&session, Some(TanChallengeType::PushTan))?;
                        return Ok(tan_challenge);
                    }
                }
                return Err(Error::UnsupportedTanType);
            }
        }

        Ok(tan_challenge)
    }

    //noinspection RsMatchCheck
    fn activate_tan(&self, session: &Session, tan_challenge: TanChallenge) -> Result<(), Error> {
        #[cfg(any(test, feature = "test"))]
            std::thread::sleep(std::time::Duration::from_secs(10));

        #[cfg(not(any(test, feature = "test")))]
            let tan =
            {
                let mut tan = match tan_challenge.typ() {
                    TanChallengeType::PushTan | TanChallengeType::Free => String::with_capacity(2),
                    TanChallengeType::MobileTan => {
                        match tan_challenge.challenge() {
                            Some(phone_number) => println!("phone number: {}", phone_number),
                            None => println!("No phone number was provided")
                        }
                        String::new()
                    }
                    _ => String::new()
                };
                println!("Please activate the tan in your photo tan app\nThen press enter");
                std::io::stdin()
                    .read_line(&mut tan)?;
                tan
            };

        let url = format!("{}/{}", url!("/session/clients/user/v1/sessions"), session.session_uuid.as_str());
        let tan_id = format!(r#"{{"id":"{}"}}"#, tan_challenge.id().as_str());
        let data = format!(
            r#"{{
                "identifier": "{}",
                "sessionTanActive":true,
                "activated2FA":true
            }}"#,
            session.session_uuid.as_str()
        );

        let mut request = self.client
            .patch(&url)
            .bearer_auth(&session.access_token)
            .header("x-http-request-info", self.make_request_info(&session.session_id))
            .header("x-once-authentication-info", tan_id)
            .body(data);

        match tan_challenge.typ() {
            TanChallengeType::PushTan | TanChallengeType::Free => {}
            #[cfg(not(any(test, feature = "test")))]
            _ => request = request.header("x-once-authentication", tan),
            #[cfg(any(test, feature = "test"))]
            _ => request = request.header("x-once-authentication", "0"),
        }

        let response = request.send()?;
        response_is_success!(response);

        let session_status = response.json::<SessionStatus>()?;
        if !session_status.tan_is_active() {
            return Err(Error::CouldNotCreateSession);
        }

        Ok(())
    }

    pub fn get_deposits(&self) -> Result<Vec<ComdirectDeposit>, Error> {
        const URL: &str = url!("/brokerage/clients/user/v3/depots");

        let session = session_is_active!(self.session);

        let response = self.make_get_session_request(URL, &session)
            .send()?;
        response_is_success!(response);

        let json = response.json::<JsonResponseValues<ComdirectDeposit>>()?;
        Ok(json.values)
    }

    pub fn get_positions<'d>(&self, deposit: &'d ComdirectDeposit) -> Result<Vec<Position<'d>>, Error> {
        let session = session_is_active!(self.session);
        let url = format!("{}/{}/positions", url!("/brokerage/v3/depots"), deposit.id());

        let response = self.make_get_session_request(&url, session)
            .query(&[("without-attr", "depot")])
            .send()?;
        response_is_success!(response);

        let json = response.json::<JsonResponseValues<RawPosition>>()?;

        let mut positions = Vec::with_capacity(json.values.len());
        for raw in json.values {
            positions.push(Position::from_raw(raw, deposit));
        }

        Ok(positions)
    }

    pub fn get_position<'d>(&self, deposit: &'d ComdirectDeposit, position_id: &PositionId) -> Result<Position<'d>, Error> {
        let session = session_is_active!(self.session);
        let url = format!("{}/{}/positions/{}", url!("/brokerage/v3/depots"), deposit.id(), position_id.as_str());

        let response = self.make_get_session_request(&url, session)
            .query(&[("without-attr", "depot")])
            .send()?;

        let raw_position = response.json::<RawPosition>()?;
        Ok(Position::from_raw(raw_position, deposit))
    }

    pub fn update_position(&self, position: &mut Position) -> Result<(), Error> {
        // todo: do not deserialize the whole position (id, ...) twice but deserialize just the changed information
        let new_position = self.get_position(position.deposit(), position.raw().id())?;
        position.set_raw(new_position.into_raw());

        Ok(())
    }

    pub fn get_deposit_transactions<'d>(&self, deposit: &'d ComdirectDeposit, filter_parameters: &TransactionFilterParameters) -> Result<Vec<Transaction<'d>>, Error> {
        let session = session_is_active!(self.session);
        let url = format!("{}/{}/transactions", url!("/brokerage/v3/depots"), deposit.id());

        let response = self.make_get_session_request(&url, session)
            .query(&[("without-attr", "instrument")])
            .query(filter_parameters)
            .send()?;
        response_is_success!(response);

        let json = response.json::<JsonResponseValues<RawTransaction>>()?;

        let mut transactions = Vec::new();
        for raw in json.values {
            transactions.push(Transaction::from_raw(raw, deposit))
        }

        Ok(transactions)
    }

    pub fn get_instrument(&self, derivative: &Derivative) -> Result<Vec<Instrument>, Error> {
        let session = session_is_active!(self.session);
        let url = format!("{}/{}", url!("/brokerage/v1/instruments/"), derivative.as_ref());

        let response = self.make_get_session_request(&url, session)
            .query(&[
                ("with-attr", "derivativeData"),
                ("with-attr", "fundDistribution"),
            ])
            .send()?;
        response_is_success!(response);

        let json = response.json::<JsonResponseValues<Instrument>>()?;
        Ok(json.values)
    }

    pub fn get_marketplaces(&self, filter_parameters: &OrderDimensionsFilterParameters) -> Result<Vec<MarketPlace>, Error> {
        const URL: &str = url!("/brokerage/v3/orders/dimensions");
        let session = session_is_active!(self.session);

        let response = self.make_get_session_request(URL, session)
            .query(filter_parameters)
            .send()?;
        response_is_success!(response);

        let json = response.json::<JsonResponseMarketplaces>()?;
        Ok(json.values.0.venues)
    }

    pub fn get_orders<'d>(&self, deposit: &'d ComdirectDeposit, filter_parameters: &OrderFilterParameters) -> Result<Vec<ComdirectOrder<'d>>, Error> {
        let session = session_is_active!(&self.session);
        let url = format!("{}/{}/v3/orders", url!("/brokerage/depots"), deposit.id());

        let response = self.make_get_session_request(&url, session)
            .query(filter_parameters)
            .send()?;
        response_is_success!(response);

        let json = response.json::<JsonResponseValues<RawComdirectOrder>>()?;

        let mut orders = Vec::with_capacity(json.values.len());
        for raw in json.values {
            orders.push(ComdirectOrder::from_raw(raw, deposit));
        }

        Ok(orders)
    }

    fn make_request_id() -> String {
        Local::now().format("%H%M%S%.3f").to_string()
    }

    fn make_session_id() -> SessionId {
        let mut rng = rand::thread_rng();
        let session_id: String = (0..32)
            .map(|_| {
                let char_id = rng.gen_range(0, HEX_CHARSET.len());
                HEX_CHARSET[char_id] as char
            })
            .collect();
        SessionId(session_id)
    }

    fn make_request_info(&self, session_id: &SessionId) -> String {
        format!(
            r#"{{"clientRequestId":{{"sessionId":"{}","requestId":"{}"}}}}"#,
            session_id.as_str(),
            Comdirect::make_request_id()
        )
    }

    fn make_get_session_request(&self, url: &str, session: &Session) -> RequestBuilder {
        self.client
            .get(url)
            .bearer_auth(&session.access_token)
            .header("x-http-request-info", self.make_request_info(&session.session_id))
    }

    fn make_post_session_request(&self, url: &str, session: &Session) -> RequestBuilder {
        self.client
            .post(url)
            .bearer_auth(&session.access_token)
            .header("x-http-request-info", self.make_request_info(&session.session_id))
    }
}
