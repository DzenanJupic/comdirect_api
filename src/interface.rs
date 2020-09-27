use std::collections::HashMap;
use std::result::Result as StdResult;

use chrono::Local;
use rand::{self, Rng};
use reqwest::blocking::{Client, RequestBuilder, Response};
use reqwest::header::{ACCEPT, CONTENT_TYPE};
use reqwest::header::{HeaderMap, HeaderValue};
use reqwest::IntoUrl;
use serde::de::DeserializeSeed;
use serde::Serialize;
use serde_json::Deserializer;
use stock_market_utils::derivative::Derivative;

use crate::error::Error;
use crate::serde::{JsonResponseValue, JsonResponseValues};
use crate::session::{GrantType, PreSession, Session, SessionId, SessionStatus};
use crate::session::tan::{TanChallenge, TanChallengeType};
use crate::types::cost_indication::{ChangeCostIndication, CostIndication, RawCostIndication};
use crate::types::deposit::ComdirectDeposit;
use crate::types::instrument::Instrument;
use crate::types::market_place::{JsonResponseMarketplaces, MarketPlace, MarketPlaceFilterParameters};
use crate::types::order::{ComdirectOrder, OrderFilterParameters, OrderId, RawOrder};
use crate::types::order::order_change::{DeleteOrder, OrderChange, OrderChangeAction, OrderChangeValidation};
use crate::types::order::order_outline::OrderOutline;
use crate::types::position::{Position, PositionId, RawPosition};
use crate::types::quote::{Quote, QuoteOutline, QuoteTicket, RawQuote};
use crate::types::transaction::{RawTransaction, Transaction, TransactionFilterParameters};

const HEX_CHARSET: &[u8] = b"0123456789abcdef";

type Result<T> = StdResult<T, Error>;

macro_rules! url {
    ($path:literal) => (concat!("https://api.comdirect.de/api", $path));
}

macro_rules! session_is_active {
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
pub struct ComdirectApi {
    client_id: ClientId,
    client_secret: ClientSecret,
    username: Username,
    password: Password,

    client: Client,
    #[getset(get)]
    session: Option<Session>,
}

impl Drop for ComdirectApi {
    fn drop(&mut self) {
        let _ = self.end_session();
    }
}

impl ComdirectApi {
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

    pub fn new_session(&mut self) -> Result<()> {
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

    pub fn refresh_session(&mut self) -> Result<()> {
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

    pub fn end_session(&mut self) -> Result<()> {
        self.revoke_oauth_token()?;
        self.session = None;

        Ok(())
    }

    fn acquire_oauth_token(&self, grant_type: GrantType) -> Result<PreSession> {
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
            .send()?
            .error_for_status()?;

        Ok(response.json::<PreSession>()?)
    }

    fn revoke_oauth_token(&self) -> Result<()> {
        const URL: &str = "https://api.comdirect.de/oauth/revoke";

        let session = session_is_active!(self.session);
        self.client
            .delete(URL)
            .bearer_auth(&session.access_token)
            .send()?
            .error_for_status()?;

        Ok(())
    }

    fn acquire_session_status(&self, pre_session: PreSession) -> Result<Session> {
        const URL: &str = url!("/session/clients/user/v1/sessions");

        let session_id = Self::make_session_id();

        let response = self.client
            .get(URL)
            .bearer_auth(&pre_session.access_token)
            .header("x-http-request-info", self.make_request_info(&session_id))
            .send()?
            .error_for_status()?;

        let session_status = response.json::<(SessionStatus, )>()?.0;

        Ok(pre_session.session(session_id, session_status.take_session_uuid()))
    }

    fn request_tan_challenge(&self, session: &Session, request_tan_type: Option<TanChallengeType>) -> Result<TanChallenge> {
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
                .send()?
                .error_for_status()?;

        let tan_challenge: TanChallenge = ComdirectApi::extract_tan_challenge(response.headers())?;

        // todo: support other tan challenge types then PushTan
        match tan_challenge.typ() {
            TanChallengeType::PushTan => {}
            tan_type => match request_tan_type {
                // to prevent loops, just request another type, if the current function call was not
                // already a type request itself
                Some(requested) => return match requested == *tan_type {
                    true => Err(Error::UnsupportedTanType),
                    false => Err(Error::UnexpectedTanType)
                },
                None => {
                    return if tan_challenge.available_types().contains(&TanChallengeType::PushTan) {
                        let tan_challenge = self.request_tan_challenge(&session, Some(TanChallengeType::PushTan))?;
                        Ok(tan_challenge)
                    } else { Err(Error::UnsupportedTanType) };
                }
            }
        }

        Ok(tan_challenge)
    }

    fn activate_tan(&self, session: &Session, tan_challenge: TanChallenge) -> Result<()> {
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
        let tan_header = ComdirectApi::make_x_authentication_info_header(&tan_challenge);
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
            .header(tan_header.0, tan_header.1)
            .body(data);

        match tan_challenge.typ() {
            TanChallengeType::PushTan | TanChallengeType::Free => {}
            #[cfg(not(any(test, feature = "test")))]
            _ => request = request.header("x-once-authentication", tan),
            #[cfg(any(test, feature = "test"))]
            _ => request = request.header("x-once-authentication", "0"),
        }

        let response = request.send()?
            .error_for_status()?;

        let session_status = response.json::<SessionStatus>()?;
        if !session_status.tan_is_active() {
            return Err(Error::CouldNotCreateSession);
        }

        Ok(())
    }

    pub fn get_deposits(&self) -> Result<Vec<ComdirectDeposit>> {
        const URL: &str = url!("/brokerage/clients/user/v3/depots");

        let session = session_is_active!(self.session);

        let response = self.make_get_session_request(URL, &session)
            .send()?
            .error_for_status()?;

        let json = response.json::<JsonResponseValues<ComdirectDeposit>>()?;
        Ok(json.values)
    }

    pub fn get_positions<'d>(&self, deposit: &'d ComdirectDeposit) -> Result<Vec<Position<'d>>> {
        let session = session_is_active!(self.session);
        let url = format!("{}/{}/positions", url!("/brokerage/v3/depots"), deposit.id());

        let response = self.make_get_session_request(&url, session)
            .query(&[("without-attr", "depot")])
            .send()?
            .error_for_status()?;

        let json = response.json::<JsonResponseValues<RawPosition>>()?;

        let mut positions = Vec::with_capacity(json.values.len());
        for raw in json.values {
            positions.push(Position::from_raw(raw, deposit));
        }

        Ok(positions)
    }

    pub fn get_position<'d>(&self, deposit: &'d ComdirectDeposit, position_id: &PositionId) -> Result<Position<'d>> {
        let response = self._get_position(deposit, position_id)?;
        let raw_position = response.json::<RawPosition>()?;
        Ok(Position::from_raw(raw_position, deposit))
    }

    pub fn update_position(&self, position: &mut Position) -> Result<()> {
        let response = self._get_position(position.deposit(), position.raw().id())?;

        let body = response.bytes()?;
        let mut deserializer = Deserializer::from_slice(&body);

        position.deserialize(&mut deserializer)?;
        Ok(())
    }

    fn _get_position(&self, deposit: &ComdirectDeposit, position_id: &PositionId) -> Result<Response> {
        let session = session_is_active!(self.session);
        let url = format!("{}/{}/positions/{}", url!("/brokerage/v3/depots"), deposit.id(), position_id.as_str());

        Ok(
            self.make_get_session_request(&url, session)
                .query(&[("without-attr", "depot")])
                .send()?
        )
    }

    pub fn get_deposit_transactions<'d>(&self, deposit: &'d ComdirectDeposit) -> Result<Vec<Transaction<'d>>> {
        self._get_deposit_transactions(deposit, None)
    }

    pub fn get_deposit_transactions_filtered<'d>(&self, deposit: &'d ComdirectDeposit, filter_parameters: &TransactionFilterParameters) -> Result<Vec<Transaction<'d>>> {
        self._get_deposit_transactions(deposit, Some(filter_parameters))
    }

    #[inline(always)]
    fn _get_deposit_transactions<'d>(&self, deposit: &'d ComdirectDeposit, filter_parameters: Option<&TransactionFilterParameters>) -> Result<Vec<Transaction<'d>>> {
        let session = session_is_active!(self.session);
        let url = format!("{}/{}/transactions", url!("/brokerage/v3/depots"), deposit.id());

        let mut request = self.make_get_session_request(&url, session)
            .query(&[("without-attr", "instrument")]);
        if let Some(filters) = filter_parameters {
            request = request.query(filters);
        }

        let response = request.send()?
            .error_for_status()?;

        let json = response.json::<JsonResponseValues<RawTransaction>>()?;

        let mut transactions = Vec::new();
        for raw in json.values {
            transactions.push(Transaction::from_raw(raw, deposit))
        }

        Ok(transactions)
    }

    pub fn get_instrument(&self, derivative: &Derivative) -> Result<Instrument> {
        let session = session_is_active!(self.session);
        let url = format!("{}/{}", url!("/brokerage/v1/instruments/"), derivative.as_ref());

        let response = self.make_get_session_request(&url, session)
            .query(&[
                ("with-attr", "derivativeData"),
                ("with-attr", "fundDistribution"),
            ])
            .send()?
            .error_for_status()?;

        let json = response.json::<JsonResponseValue<Instrument>>()?;
        Ok(json.values.0)
    }

    pub fn get_marketplaces(&self) -> Result<Vec<MarketPlace>> {
        self._get_marketplaces(None)
    }

    pub fn get_marketplaces_filtered(&self, filter_parameters: &MarketPlaceFilterParameters) -> Result<Vec<MarketPlace>> {
        self._get_marketplaces(Some(filter_parameters))
    }

    #[inline(always)]
    fn _get_marketplaces(&self, filter_parameters: Option<&MarketPlaceFilterParameters>) -> Result<Vec<MarketPlace>> {
        const URL: &str = url!("/brokerage/v3/orders/dimensions");
        let session = session_is_active!(self.session);

        let mut request = self.make_get_session_request(URL, session);
        if let Some(filters) = filter_parameters {
            request = request.query(filters)
        }

        let response = request.send()?
            .error_for_status()?;

        let json = response.json::<JsonResponseMarketplaces>()?;
        Ok(json.market_places())
    }

    pub fn get_orders<'d>(&self, deposit: &'d ComdirectDeposit) -> Result<Vec<ComdirectOrder<'d>>> {
        self._get_orders(deposit, None)
    }

    pub fn get_orders_filtered<'d>(&self, deposit: &'d ComdirectDeposit, filter_parameters: &OrderFilterParameters) -> Result<Vec<ComdirectOrder<'d>>> {
        self._get_orders(deposit, Some(filter_parameters))
    }

    #[inline(always)]
    fn _get_orders<'d>(&self, deposit: &'d ComdirectDeposit, filter_parameters: Option<&OrderFilterParameters>) -> Result<Vec<ComdirectOrder<'d>>> {
        let session = session_is_active!(&self.session);
        let url = format!("{}/{}/v3/orders", url!("/brokerage/depots"), deposit.id());

        let mut request = self.make_get_session_request(&url, session);
        if let Some(filters) = filter_parameters {
            request = request.query(filters);
        }

        let response = request.send()?
            .error_for_status()?;

        let json = response.json::<JsonResponseValues<RawOrder>>()?;

        let mut orders = Vec::with_capacity(json.values.len());
        for raw in json.values {
            orders.push(ComdirectOrder::from_raw(raw, deposit));
        }

        Ok(orders)
    }

    pub fn get_order<'d>(&self, deposit: &'d ComdirectDeposit, order_id: &OrderId) -> Result<ComdirectOrder<'d>> {
        let session = session_is_active!(self.session);
        let url = format!("{}/{}", url!("/brokerage/v3/orders"), order_id.as_str());

        let response = self.make_get_session_request(&url, session)
            .send()?
            .error_for_status()?;

        let raw = response.json::<RawOrder>()?;
        Ok(ComdirectOrder::from_raw(raw, deposit))
    }

    pub fn order_cost_indication<'o, 'd, 'i, 'm>(&self, order_outline: &'o OrderOutline<'d, 'i, 'm>) -> Result<CostIndication<'o, 'd, 'i, 'm>> {
        const URL: &str = url!("/brokerage/v3/orders/costindicationexante");
        let session = session_is_active!(self.session);

        let response = self.make_post_session_request(URL, session)
            .json(order_outline)
            .send()?
            .error_for_status()?;

        let raw = response.json::<JsonResponseValue<RawCostIndication>>()?.values.0;
        let cost_indication = CostIndication::from_raw(raw, order_outline);
        Ok(cost_indication)
    }

    pub fn pre_validate_order_outline(&self, order_outline: &OrderOutline) -> Result<()> {
        const URL: &str = url!("/brokerage/v3/orders/prevalidation");
        let session = session_is_active!(self.session);

        self.make_post_session_request(URL, session)
            .json(order_outline)
            .send()?
            .error_for_status()?;

        Ok(())
    }

    pub fn place_order<'d>(&self, order_outline: &OrderOutline<'d, '_, '_>) -> Result<ComdirectOrder<'d>> {
        let tan_challenge = self.validate_order_outline(order_outline)?;
        let order = self.place_order_outline(order_outline, tan_challenge)?;
        Ok(order)
    }

    fn validate_order_outline(&self, order_outline: &OrderOutline) -> Result<TanChallenge> {
        let response = self.validate_outline(order_outline)?;

        let tan_challenge = ComdirectApi::extract_tan_challenge(response.headers())?;
        tan_is_free!(tan_challenge);

        Ok(tan_challenge)
    }

    fn validate_outline<O: Serialize>(&self, outline: &O) -> Result<Response> {
        const URL: &str = url!("/brokerage/v3/orders/validation");
        let session = session_is_active!(self.session);

        Ok(
            self.make_post_session_request(URL, session)
                .json(outline)
                .send()?
                .error_for_status()?
        )
    }

    fn place_order_outline<'d>(&self, order_outline: &OrderOutline<'d, '_, '_>, tan_challenge: TanChallenge) -> Result<ComdirectOrder<'d>> {
        const URL: &str = url!("/brokerage/v3/orders");
        let session = session_is_active!(self.session);
        let tan_header = ComdirectApi::make_x_authentication_info_header(&tan_challenge);

        let response = self.make_post_session_request(URL, session)
            .header(tan_header.0, tan_header.1)
            .json(order_outline)
            .send()?
            .error_for_status()?;

        let raw_order = response.json::<RawOrder>()?;
        let order = ComdirectOrder::from_raw(raw_order, order_outline.deposit());
        Ok(order)
    }

    pub fn pre_validate_order_change(&self, order_change: &OrderChange) -> Result<()> {
        let validation = OrderChangeValidation::Change(order_change);
        self._pre_validate_order_change(validation)
    }

    pub fn pre_validate_order_deletion(&self, order: &ComdirectOrder) -> Result<()> {
        let validation = OrderChangeValidation::Delete(order);
        self._pre_validate_order_change(validation)
    }

    #[inline(always)]
    fn _pre_validate_order_change(&self, change_validation: OrderChangeValidation) -> Result<()> {
        let session = session_is_active!(self.session);
        let url = format!(
            "{}/{}/prevalidation",
            url!("/brokerage/v3/orders"), change_validation.order_id()
        );

        Self::make_order_change_body(
            self.make_post_session_request(&url, session),
            &change_validation,
        )
            .send()?
            .error_for_status()?;
        Ok(())
    }

    pub fn order_change_cost_indication<'oc, 'o>(&self, order_change: &'oc OrderChange<'o>) -> Result<ChangeCostIndication<'oc, 'o, '_>> {
        let validation = OrderChangeValidation::Change(order_change);
        self._order_change_cost_indication(validation)
    }

    // FIXME: Currently this interface does not work / is not available
    // I already contacted the Comdirect support
    // #[inline(always)]
    // pub fn order_deletion_cost_indication<'o, 'd>(&self, order: &'o ComdirectOrder<'d>) -> Result<ChangeCostIndication<'_, 'o, 'd>, Error> {
    //     let validation = OrderChangeValidation::Delete(order);
    //     self._order_change_cost_indication(validation)
    // }

    #[inline(always)]
    fn _order_change_cost_indication<'oc, 'o, 'd>(&self, change_validation: OrderChangeValidation<'o, 'd, 'oc>) -> Result<ChangeCostIndication<'oc, 'o, 'd>> {
        use OrderChangeValidation::*;
        let session = session_is_active!(self.session);
        let url = format!("{}/{}/costindicationexante", url!("/brokerage/v3/orders"), change_validation.order_id());

        let response = Self::make_order_change_body(
            self.make_post_session_request(&url, session),
            &change_validation,
        )
            .send()?
            .error_for_status()?;

        let raw = response.json::<JsonResponseValue<RawCostIndication>>()?.values.0;
        let cost_indication = match change_validation {
            Change(order_change) => ChangeCostIndication::Change { order_change, raw },
            Delete(order) => ChangeCostIndication::Delete { order, raw }
        };

        Ok(cost_indication)
    }

    pub fn change_order(&self, order_change: OrderChange) -> Result<()> {
        let tan_challenge = self.validate_order_change(&order_change)?;
        let action = OrderChangeAction::Change(order_change);
        self._change_order(action, tan_challenge)
    }

    pub fn delete_order<'d>(&self, order: ComdirectOrder<'d>) -> StdResult<(), (Error, ComdirectOrder<'d>)> {
        macro_rules! map_err {
            ($expr:expr) => {
                match $expr {
                    Ok(o) => o,
                    Err(e) => return Err((e, order))
                }
            };
        }

        let tan_challenge = map_err!(self.validate_order_deletion(&order));
        let action = OrderChangeAction::Delete(&order);
        map_err!(self._change_order(action, tan_challenge));
        Ok(())
    }

    fn validate_order_change(&self, order_change: &OrderChange) -> Result<TanChallenge> {
        let validation = OrderChangeValidation::Change(order_change);
        self._validate_order_change(validation)
    }

    fn validate_order_deletion(&self, order: &ComdirectOrder) -> Result<TanChallenge> {
        let validation = OrderChangeValidation::Delete(order);
        self._validate_order_change(validation)
    }

    #[inline(always)]
    fn _validate_order_change(&self, change_validation: OrderChangeValidation) -> Result<TanChallenge> {
        use OrderChangeValidation::*;
        let session = session_is_active!(self.session);
        let url = format!("{}/{}/validation", url!("/brokerage/v3/orders"), change_validation.order_id());

        let mut request = self.make_post_session_request(&url, session);
        match change_validation {
            Change(order_change) => request = request.json(order_change),
            Delete(_) => request = request.json(&DeleteOrder {})
        }

        let response = request
            .send()?
            .error_for_status()?;
        let tan_challenge = ComdirectApi::extract_tan_challenge(response.headers())?;
        tan_is_free!(tan_challenge);

        Ok(tan_challenge)
    }

    #[inline(always)]
    fn _change_order(&self, change_action: OrderChangeAction, tan_challenge: TanChallenge) -> Result<()> {
        use OrderChangeAction::*;
        let session = session_is_active!(self.session);
        let url = format!("{}/{}", url!("/brokerage/v3/orders"), change_action.order_id());
        let tan_header = ComdirectApi::make_x_authentication_info_header(&tan_challenge);

        let request = match change_action {
            Change(ref order_change) => {
                self
                    .make_patch_session_request(&url, session)
                    .json(order_change)
            }
            Delete(_) => {
                self
                    .make_delete_session_request(&url, session)
                    .json(&DeleteOrder {})
            }
        };

        request
            .header(tan_header.0, tan_header.1)
            .send()?
            .error_for_status()?;

        if let Change(order_change) = change_action {
            order_change.change_order();
        }
        Ok(())
    }

    // FIXME: Bug in the REST API or wrong documentation
    // problem occurs in ComdirectApi::place_quote_outline (currently commented out)
    // api documentation: 8.1.3
    // response: {"code":"request.object.invalid","messages":[{"severity":"ERROR","key":"fehler_unsupported_ordertyp","message":"Der Ordertyp ist leider ungültig.","args":{},"origin":[]}]}
    // Comdirect community blog post: https://community.comdirect.de/t5/Website-Apps/REST-API-8-1-3-Anlage-Quote-Request-gibt-422-quot-fehler/td-p/149472
    pub fn get_quote<'d>(&self, _outline: &QuoteOutline<'d, '_, '_>) -> Result<Quote<'d>> {
        unimplemented!(
            "This part of the interface currently doesn't work caused by a bug in the REST API.\n\
            Comdirect community blog post: https://community.comdirect.de/t5/Website-Apps/REST-API-8-1-3-Anlage-Quote-Request-gibt-422-quot-fehler/td-p/149472"
        );

        /*let (quote_ticket, tan_challenge) = self.validate_quote_outline(&outline)?;
        self.validate_quote_tan(&quote_ticket, tan_challenge)?;
        let quote = self.place_quote_outline(&outline)?;
        
        Ok(quote)*/
    }

    /*fn validate_quote_outline(&self, outline: &QuoteOutline) -> Result<(QuoteTicket, TanChallenge)> {
        const URL: &str = url!("/brokerage/v3/quoteticket");
        let session = session_is_active!(self.session);
        
        let response = self.make_post_session_request(URL, session)
            .json(outline)
            .send()?
            .error_for_status()?;
        
        let tan_challenge = ComdirectApi::extract_tan_challenge(response.headers())?;
        let quote_ticket = response.json::<QuoteTicket>()?;
        
        Ok((quote_ticket, tan_challenge))
    }
    
    fn validate_quote_tan(&self, quote_ticket: &QuoteTicket, tan_challenge: TanChallenge) -> Result<()> {
        match tan_challenge.typ() {
            TanChallengeType::Free => {},
            _ => return Err(Error::UnexpectedTanType)
        }
        
        let session = session_is_active!(self.session);
        let url = format!("{}/{}", url!("/brokerage/v3/quoteticket"), quote_ticket.id());
        let tan_header = ComdirectApi::make_x_authentication_info_header(&tan_challenge);

        self.make_patch_session_request(&url, session)
            .header("x-once-authentication", "TAN_FREI")
            .header(tan_header.0, tan_header.1)
            .send()?
            .error_for_status()?;
        
        Ok(())
    }
    
    fn place_quote_outline<'d>(&self, outline: &QuoteOutline<'d, '_, '_>) -> Result<Quote<'d>> {
        const URL: &str = url!("/brokerage/v3/quotes");
        let session = session_is_active!(self.session);
        
        
        dbg!(serde_json::to_string(outline).unwrap());
        

        let response = dbg!(self.make_post_session_request(URL, session)
            .json(outline)
            .send()?)
            .error_for_status()?;
        
        dbg!(response.headers());
        let text = dbg!(response.text()?);
        
        let raw_quote = serde_json::from_str::<RawQuote>(&text)?;
        
        // let raw_quote = response.json::<RawQuote>()?;
        let quote = Quote::from_raw(raw_quote, outline.deposit());
        
        Ok(quote)
    }*/

    // TODO: REST API 8.1.4
    // currently not possible

    #[inline(always)]
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

    #[inline(always)]
    fn make_request_id() -> String {
        Local::now().format("%H%M%S%3f").to_string()
    }

    #[inline(always)]
    fn make_request_info(&self, session_id: &SessionId) -> String {
        format!(
            r#"{{"clientRequestId":{{"sessionId":"{}","requestId":"{}"}}}}"#,
            session_id.as_str(),
            ComdirectApi::make_request_id()
        )
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

    #[inline(always)]
    fn make_order_change_body(request: RequestBuilder, change_validation: &OrderChangeValidation) -> RequestBuilder {
        use OrderChangeValidation::*;
        match change_validation {
            Change(order_change) => request.json(order_change),
            Delete(_) => request.json(&DeleteOrder {})
        }
    }

    session_request_method!(make_get_session_request, get);
    session_request_method!(make_post_session_request, post);
    session_request_method!(make_patch_session_request, patch);
    session_request_method!(make_delete_session_request, delete);
}