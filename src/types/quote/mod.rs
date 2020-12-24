use chrono::{DateTime, Utc};
use getset::{Getters, Setters};
use pecunia::price::Price;
use pecunia::primitives::F64;
use serde::{Deserialize, Serialize};
use wall_street::order::OrderDirection;

use crate::types::deposit::ComdirectDeposit;
use crate::types::instrument::InstrumentId;
use crate::types::market_place::MarketPlaceId;

pub mod order_outline;

new_type_ids!(
    pub struct QuoteId
);

#[derive(Debug, Getters)]
pub struct Quote<'d> {
    deposit: &'d ComdirectDeposit,
    ticket: QuoteTicket,
    raw: RawQuote,
}

#[derive(Clone, Debug, Deserialize, Getters)]
#[getset(get = "pub")]
#[serde(rename_all = "camelCase")]
pub struct RawQuote {
    #[serde(with = "QuoteId")]
    #[serde(rename = "quoteId")]
    id: QuoteId,
    #[serde(with = "InstrumentId")]
    instrument_id: InstrumentId,
    #[serde(with = "MarketPlaceId")]
    #[serde(rename = "venueId")]
    market_place_id: MarketPlaceId,
    #[serde(rename = "side")]
    #[serde(with = "crate::serde::order_direction")]
    direction: OrderDirection,
    #[serde(with = "crate::serde::amount_value::quantity")]
    quantity: F64,
    #[serde(rename = "creationDateTimeStamp")]
    #[serde(with = "crate::serde::date::time_stamp_string_utc")]
    creation: DateTime<Utc>,
    validity: u64,
    #[serde(with = "crate::serde::amount_value::price")]
    limit: Price,
    #[serde(with = "crate::serde::amount_value::price")]
    expected_value: Price,
}

#[derive(Clone, Debug, Serialize, Getters, Setters, derive_builder::Builder)]
#[getset(get = "pub")]
#[getset(set = "pub")]
#[serde(rename_all = "camelCase")]
#[serde(tag = "orderType", rename = "QUOTE")]
pub struct QuoteOutline<'d, 'i, 'm> {
    #[serde(rename = "depotId")]
    #[serde(serialize_with = "crate::serde::serialize_deposit_as_id")]
    deposit: &'d ComdirectDeposit,
    instrument_id: &'i InstrumentId,
    #[serde(rename = "venueId")]
    market_place_id: &'m MarketPlaceId,
    #[serde(rename = "side")]
    #[serde(with = "crate::serde::order_direction")]
    direction: OrderDirection,
    #[serde(with = "crate::serde::amount_value::quantity")]
    quantity: F64,
}

#[derive(Clone, Debug, Serialize, Deserialize, Getters)]
#[getset(get = "pub")]
pub struct QuoteTicket {
    #[serde(with = "QuoteId")]
    #[serde(rename = "quoteTicketId")]
    id: QuoteId
}

impl<'d> Quote<'d> {
    pub(crate) fn from_raw(raw: RawQuote, ticket: QuoteTicket, deposit: &'d ComdirectDeposit) -> Self {
        Self { deposit, ticket, raw }
    }
}

impl<'d, 'i, 'm> QuoteOutline<'d, 'i, 'm> {
    pub fn builder() -> QuoteOutlineBuilder<'d, 'i, 'm> {
        QuoteOutlineBuilder::default()
    }
}
