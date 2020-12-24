use chrono::{DateTime, Utc};
use getset::Getters;
use pecunia::price::Price;
use pecunia::primitives::F64;
use serde::{Serialize, Serializer};
use wall_street::order::OrderDirection;

use crate::types::deposit::ComdirectDeposit;
use crate::types::instrument::InstrumentId;
use crate::types::market_place::MarketPlaceId;
use crate::types::quote::{Quote, QuoteId};

#[derive(Debug, Serialize, Getters)]
#[getset(get = "pub")]
#[serde(rename_all = "camelCase")]
#[serde(tag = "orderType", rename = "QUOTE")]
pub struct QuoteOrderOutline<'d> {
    #[serde(rename = "depotId")]
    #[serde(serialize_with = "crate::serde::serialize_deposit_as_id")]
    deposit: &'d ComdirectDeposit,
    instrument_id: InstrumentId,
    #[serde(rename = "venueId")]
    market_place_id: MarketPlaceId,

    quote_id: QuoteId,
    #[serde(rename = "quoteTicketId")]
    ticket_id: QuoteId,

    #[serde(rename = "side")]
    #[serde(with = "crate::serde::order_direction")]
    direction: OrderDirection,
    #[serde(with = "crate::serde::amount_value::quantity")]
    quantity: F64,
    #[serde(rename = "creationTimestamp")]
    #[serde(serialize_with = "ser_creation")]
    creation: DateTime<Utc>,
    #[serde(with = "crate::serde::amount_value::price")]
    limit: Price,
}

impl<'d> From<Quote<'d>> for QuoteOrderOutline<'d> {
    fn from(quote: Quote<'d>) -> Self {
        Self {
            deposit: quote.deposit,
            instrument_id: quote.raw.instrument_id,
            market_place_id: quote.raw.market_place_id,
            quote_id: quote.raw.id,
            ticket_id: quote.ticket.id,
            direction: quote.raw.direction,
            quantity: quote.raw.quantity,
            creation: quote.raw.creation,
            limit: quote.raw.limit,
        }
    }
}

fn ser_creation<S>(date: &DateTime<Utc>, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer {
    let mut s = date
        .format("%FT%T,%6f")
        .to_string();
    s.push_str("+00");
    s.serialize(serializer)
}
