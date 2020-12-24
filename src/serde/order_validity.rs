use chrono::NaiveDate;
use pecunia::{serde_option, serde_with};
use serde::{Deserialize, Serialize};
use wall_street::order::OrderValidity;

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
    #[serde(with = "crate::serde::date::date_string")]
    TillDate(NaiveDate),
}

serde_with!(Serializer for OrderValidity as pub(crate) OrderValiditySerializer with "OrderValidityDef");
serde_with!(Deserializer for OrderValidity as pub(crate) OrderValidityDeserializer with "OrderValidityDef");

pub(crate) mod option {
    use super::*;

    serde_option!(serialize OrderValidity as pub(crate) OrderValidityOptionSerializer with OrderValiditySerializer);
    serde_option!(deserialize OrderValidity as pub(crate) OrderValidityOptionDeserializer with OrderValidityDeserializer);
}
