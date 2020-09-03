use pecunia::{serde_with};
use serde::{Deserialize, Serialize};
use stock_market_utils::order::OrderStatus;

#[derive(Serialize, Deserialize)]
#[serde(remote = "OrderStatus")]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
enum OrderStatusDef {
    Open,
    #[serde(alias = "SETTLED")]
    Executed,
    PartiallyExecuted,
    #[serde(alias = "CANCELLED_SYSTEM")]
    #[serde(alias = "CANCELLED_TRADE")]
    #[serde(alias = "CANCELLED_USER")]
    Canceled,
    PartiallyCanceled,
    Expired,
    #[serde(alias = "WAITING")]
    Pending,
    Unknown,
}

serde_with!(Serializer for OrderStatus as pub(crate) OrderStatusSerializer with "OrderStatusDef");
serde_with!(Deserializer for OrderStatus as pub(crate) OrderStatusDeserializer with "OrderStatusDef");

pub(crate) mod option {
    use super::*;
    pecunia::serde_option!(serialize OrderStatus as pub(crate) OrderStatusOptionSerialier with OrderStatusSerializer);
    pecunia::serde_option!(deserialize OrderStatus as pub(crate) OrderStatusOptionDeserialier with OrderStatusDeserializer);
}
