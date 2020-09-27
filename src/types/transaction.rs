use chrono::NaiveDate;
use pecunia::price::Price;
use serde::{Deserialize, Serialize};
use stock_market_utils::derivative::{ISIN, WKN};

use crate::types::deposit::ComdirectDeposit;
use crate::types::position::Position;

new_type_ids!(
    pub struct TransactionId
);

#[derive(Clone, Debug, PartialEq)]
pub struct Transaction<'d> {
    deposit: &'d ComdirectDeposit,
    raw: RawTransaction,
}

#[derive(Clone, Debug, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct RawTransaction {
    #[serde(rename = "transaction_id")]
    id: Option<TransactionId>,
    #[serde(rename = "bookingStatus")]
    status: BookingStatus,
    #[serde(rename = "bookingDate")]
    #[serde(with = "crate::serde::date::date_string")]
    date: NaiveDate,
    #[serde(rename = "transactionValue")]
    #[serde(with = "crate::serde::amount_value::price")]
    value: Price,
    #[serde(rename = "transactionDirection")]
    direction: TransactionDirection,
    transaction_type: TransactionType,
}

#[derive(Copy, Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "UPPERCASE")]
pub enum BookingStatus {
    Booked,
    NotBooked,
    Both,
}

#[derive(Copy, Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "UPPERCASE")]
pub enum TransactionDirection {
    In,
    Out,
}

#[derive(Copy, Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum TransactionType {
    Buy,
    Sell,
    TransferIn,
    TransferOut,
    Other,
}

#[derive(Clone, Debug, Default, serde::Serialize, getset::Setters)]
#[getset(set = "pub")]
pub struct TransactionFilterParameters<'a> {
    isin: Option<&'a ISIN>,
    wkn: Option<&'a WKN>,
    instrument_id: Option<&'a str>,
    #[serde(with = "crate::serde::date::date_string::option")]
    max_booking_date: Option<NaiveDate>,
    transaction_direction: Option<TransactionDirection>,
    transaction_type: Option<TransactionType>,
    booking_status: Option<BookingStatus>,
    min_transaction_value: Option<Price>,
    max_transaction_value: Option<Price>,
}

impl<'d> Transaction<'d> {
    pub(crate) fn from_raw(raw: RawTransaction, deposit: &'d ComdirectDeposit) -> Self {
        Self {
            deposit,
            raw,
        }
    }
}

impl<'a, 'm> TransactionFilterParameters<'a> {
    pub fn set_position_wkn(mut self, position: &'a Position) -> Self {
        self.wkn = Some(position.raw().wkn());
        self
    }
}
