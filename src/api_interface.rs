use std::collections::HashMap;

use chrono::Local;
use rand::{self, Rng};
use reqwest::{
    blocking::Client,
    header::{ACCEPT, CONTENT_TYPE},
};
use reqwest::blocking::RequestBuilder;
use reqwest::header::{HeaderMap, HeaderValue};
use stock_market_utils::derivative::Derivative;

use crate::deposit::ComdirectDeposit;
use crate::error::Error;
use crate::instrument::Instrument;
use crate::market_place::{JsonResponseMarketplaces, MarketPlace, MarketPlaceFilterParameters};
use crate::order::{ComdirectOrder, OrderFilterParameters, OrderId, RawOrder};
use crate::order_outline::ComdirectOrderOutline;
use crate::position::{Position, PositionId, RawPosition};
use crate::serde::JsonResponseValues;
use crate::session::{GrantType, PreSession, Session, SessionId, SessionStatus};
use crate::tan::{TanChallenge, TanChallengeType};
use crate::transaction::{RawTransaction, Transaction, TransactionFilterParameters};

const HEX_CHARSET: &[u8] = b"0123456789abcdef";

macro_rules! url {
    ($path:literal) => (concat!("https://api.comdirect.de/api", $path));
}

macro_rules! response_is_success {
    ($response:ident) => (response_is_success!($response, return Err(Error::Other)););
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

macro_rules! session_request_method {
    ($method_name:ident, $method:ident) => {
        #[inline(always)]
        fn $method_name(&self, url: &str, session: &Session) -> RequestBuilder {
            self.client
                .$method(url)
                .bearer_auth(&session.access_token)
                .header("x-http-request-info", self.make_request_info(&session.session_id))
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
            .to_str()
            .map_err(|_| Error::UnexpectedResponseHeaders)?;

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
        // use DeserializeSeeded 
        let new_position = self.get_position(position.deposit(), position.raw().id())?;
        position.set_raw(new_position.into_raw());

        Ok(())
    }

    #[inline(always)]
    pub fn get_deposit_transactions<'d>(&self, deposit: &'d ComdirectDeposit) -> Result<Vec<Transaction<'d>>, Error> {
        self._get_deposit_transactions(deposit, None)
    }

    #[inline(always)]
    pub fn get_deposit_transactions_filtered<'d>(&self, deposit: &'d ComdirectDeposit, filter_parameters: &TransactionFilterParameters) -> Result<Vec<Transaction<'d>>, Error> {
        self._get_deposit_transactions(deposit, Some(filter_parameters))
    }

    fn _get_deposit_transactions<'d>(&self, deposit: &'d ComdirectDeposit, filter_parameters: Option<&TransactionFilterParameters>) -> Result<Vec<Transaction<'d>>, Error> {
        let session = session_is_active!(self.session);
        let url = format!("{}/{}/transactions", url!("/brokerage/v3/depots"), deposit.id());

        let mut request = self.make_get_session_request(&url, session)
            .query(&[("without-attr", "instrument")]);
        if let Some(filters) = filter_parameters {
            request = request.query(filters);
        }

        let response = request.send()?;
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

    #[inline(always)]
    pub fn get_marketplaces(&self) -> Result<Vec<MarketPlace>, Error> {
        self._get_marketplaces(None)
    }

    #[inline(always)]
    pub fn get_marketplaces_filtered(&self, filter_parameters: &MarketPlaceFilterParameters) -> Result<Vec<MarketPlace>, Error> {
        self._get_marketplaces(Some(filter_parameters))
    }

    fn _get_marketplaces(&self, filter_parameters: Option<&MarketPlaceFilterParameters>) -> Result<Vec<MarketPlace>, Error> {
        const URL: &str = url!("/brokerage/v3/orders/dimensions");
        let session = session_is_active!(self.session);

        let mut request = self.make_get_session_request(URL, session);
        if let Some(filters) = filter_parameters {
            request = request.query(filters)
        }

        let response = request.send()?;
        response_is_success!(response);

        let json = response.json::<JsonResponseMarketplaces>()?;
        Ok(json.market_places())
    }

    #[inline(always)]
    pub fn get_orders<'d>(&self, deposit: &'d ComdirectDeposit) -> Result<Vec<ComdirectOrder<'d>>, Error> {
        self._get_orders(deposit, None)
    }

    #[inline(always)]
    pub fn get_orders_filtered<'d>(&self, deposit: &'d ComdirectDeposit, filter_parameters: &OrderFilterParameters) -> Result<Vec<ComdirectOrder<'d>>, Error> {
        self._get_orders(deposit, Some(filter_parameters))
    }

    fn _get_orders<'d>(&self, deposit: &'d ComdirectDeposit, filter_parameters: Option<&OrderFilterParameters>) -> Result<Vec<ComdirectOrder<'d>>, Error> {
        let session = session_is_active!(&self.session);
        let url = format!("{}/{}/v3/orders", url!("/brokerage/depots"), deposit.id());

        let mut request = self.make_get_session_request(&url, session);
        if let Some(filters) = filter_parameters {
            request = request.query(filters);
        }

        let response = request.send()?;
        response_is_success!(response);

        let json = response.json::<JsonResponseValues<RawOrder>>()?;

        let mut orders = Vec::with_capacity(json.values.len());
        for raw in json.values {
            orders.push(ComdirectOrder::from_raw(raw, deposit));
        }

        Ok(orders)
    }

    pub fn get_order<'d>(&self, deposit: &'d ComdirectDeposit, order_id: &OrderId) -> Result<ComdirectOrder<'d>, Error> {
        let session = session_is_active!(self.session);
        let url = format!("{}/{}", url!("/brokerage/v3/orders"), order_id.as_str());

        let response = self.make_get_session_request(&url, session)
            .send()?;
        response_is_success!(response);

        let raw = response.json::<RawOrder>()?;
        Ok(ComdirectOrder::from_raw(raw, deposit))
    }

    pub fn pre_validate_order(&self, order_outline: &ComdirectOrderOutline) -> Result<(), Error> {
        const URL: &str = url!("/brokerage/v3/orders/prevalidation");
        let session = session_is_active!(self.session);

        let response = self.make_post_session_request(URL, session)
            .json(order_outline)
            .send()?;
        response_is_success!(response);

        Ok(())
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

    fn make_request_id() -> String {
        Local::now().format("%H%M%S%.3f").to_string()
    }

    fn make_request_info(&self, session_id: &SessionId) -> String {
        format!(
            r#"{{"clientRequestId":{{"sessionId":"{}","requestId":"{}"}}}}"#,
            session_id.as_str(),
            Comdirect::make_request_id()
        )
    }

    session_request_method!(make_get_session_request, get);
    session_request_method!(make_post_session_request, post);
}
