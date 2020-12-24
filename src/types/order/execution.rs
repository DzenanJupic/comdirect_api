use chrono::{DateTime, Utc};
use pecunia::price::Price;
use serde::Deserialize;

use crate::types::order::ExecutionId;

#[derive(Clone, Debug, Deserialize, PartialEq)]
pub struct Execution {
    #[serde(with = "ExecutionId")]
    #[serde(rename = "executionId")]
    id: ExecutionId,
    /// indicates the chronological rank in which this [`Execution`] was done relative
    /// to other executions of the same [`Order`]
    /// first execution = 1
    #[serde(rename = "executionNumber")]
    rank: u64,
    /// for explanation, why this is a price, see [RawSingleOrder::quantity](RawSingleOrder::quantity)
    #[serde(rename = "executedQuantity")]
    #[serde(with = "crate::serde::amount_value::price")]
    quantity: Price,
    #[serde(rename = "executionPrice")]
    #[serde(with = "crate::serde::amount_value::price")]
    price: Price,
    #[serde(default)]
    #[serde(rename = "executionTimestamp")]
    #[serde(with = "crate::serde::date::time_stamp_string_utc::option")]
    time: Option<DateTime<Utc>>,
}
