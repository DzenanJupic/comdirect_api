use stock_market_utils::order::OrderValidity;
use serde::{Serialize, Deserialize};
use pecunia::{serde_with, serde_option};
use chrono::NaiveDate;

#[derive(Serialize, Deserialize)]
#[serde(remote = "OrderValidity")]
#[serde(tag = "validityType", content = "validity")]
pub enum OrderValidityDef {
    #[serde(rename = "GFD")]
    OneDay,
    #[serde(skip)]
    OneWeek,
    #[serde(skip)]
    OneMonth,
    #[serde(skip)]
    OneYear,
    #[serde(skip)]
    Infinite,
    #[serde(rename = "GTD")]
    #[serde(with = "crate::serde::naive_date")]
    TillDate(NaiveDate),
}

serde_with!(Serializer for OrderValidity as pub(crate) OrderValiditySerializer with "OrderValidityDef");
serde_with!(Deserializer for OrderValidity as pub(crate) OrderValidityDeserializer with "OrderValidityDef");

pub(crate) mod option {
    use super::*;
    serde_option!(serialize OrderValidity as pub(crate) OrderValidityOptionSerializer with OrderValiditySerializer);
}
