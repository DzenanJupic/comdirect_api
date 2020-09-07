use chrono::{DateTime, Utc};
use pecunia::price::{Price, TimeBoundedPrice};
use serde::{Deserialize, Deserializer, Serialize};

#[derive(Serialize, Deserialize)]
#[serde(remote = "TimeBoundedPrice::<Utc>")]
struct TimeBoundedPriceDef {
    #[serde(getter = "TimeBoundedPrice::price")]
    #[serde(serialize_with = "Price::serialize_unit_value_map")]
    #[serde(deserialize_with = "Price::deserialize_unit_value_map")]
    price: Price,
    #[serde(getter = "TimeBoundedPrice::date_time")]
    #[serde(rename = "priceDateTime")]
    #[serde(with = "crate::serde::date::date_time_string_utc")]
    date_time: DateTime<Utc>,
}

impl From<TimeBoundedPriceDef> for TimeBoundedPrice<Utc> {
    fn from(td: TimeBoundedPriceDef) -> Self {
        From::<(Price, DateTime<Utc>)>::from((td.price, td.date_time))
    }
}

#[derive(Deserialize)]
#[serde(transparent)]
pub(crate) struct TimeBoundedPriceDeserializer {
    #[serde(with = "TimeBoundedPriceDef")]
    pub(crate) remote: TimeBoundedPrice<Utc>
}

pub(crate) fn deserialize<'de, D>(deserializer: D) -> Result<TimeBoundedPrice<Utc>, D::Error>
    where
        D: Deserializer<'de> {
    Deserialize::deserialize(deserializer).map(|ok: TimeBoundedPriceDeserializer| ok.remote)
}

