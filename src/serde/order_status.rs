use serde::{Deserialize, Deserializer, Serialize};
use stock_market_utils::order::OrderStatus;

// noinspection RsTypeCheck
pub(crate) fn deserialize<'de, D>(deserializer: D) -> Result<OrderStatus, D::Error>
    where D: Deserializer<'de> {
    RemoteOrderStatus::deserialize(deserializer)
}

#[derive(Serialize, Deserialize)]
#[serde(remote = "OrderStatus")]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
enum RemoteOrderStatus {
    Open,
    #[serde(alias = "Settled")]
    Executed,
    PartiallyExecuted,
    #[serde(alias = "CancelledSystem")]
    #[serde(alias = "CancelledTrade")]
    #[serde(alias = "CancelledUser")]
    Canceled,
    PartiallyCanceled,
    Expired,
    Pending,
    Unknown,
}

pub(crate) mod option {
    use super::*;

    serde_option!(serialize OrderStatus, "RemoteOrderStatus");
}
