use chrono::{DateTime, Utc};
use pecunia::price::Price;
use serde::Deserialize;

use crate::api_types::order::ExecutionId;

#[derive(Clone, Debug, Deserialize, PartialEq)]
pub struct Execution {
    #[serde(rename = "executionId")]
    id: ExecutionId,
    /// indicates the chronological rank in which this [`Execution`] was done relative
    /// to other executions of the same [`Order`]
    rank: u64,
    /// for explanation, why this is a price, see [RawSingleOrder::quantity](RawSingleOrder::quantity)
    #[serde(with = "crate::serde::amount_value::price")]
    quantity: Price,
    #[serde(with = "crate::serde::amount_value::price")]
    price: Price,
    #[serde(with = "crate::serde::date::time_stamp_string_utc")]
    time: DateTime<Utc>,
}

