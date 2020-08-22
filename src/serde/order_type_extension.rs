use serde::{Deserialize, Deserializer};
use stock_market_utils::order::OrderTypeExtension;

// noinspection RsTypeCheck
pub(crate) fn deserialize<'de, D>(deserializer: D) -> Result<OrderTypeExtension, D::Error>
    where D: Deserializer<'de> {
    RemoteOrderTypeExtension::deserialize(deserializer)
}

#[derive(Deserialize)]
#[serde(remote = "OrderTypeExtension")]
enum RemoteOrderTypeExtension {
    #[serde(rename = "AON")]
    AllOrNone,
    #[serde(rename = "IOC")]
    ImmediateOrCancel,
    #[serde(rename = "FOK")]
    FillOrKill,
    #[serde(skip)]
    None,
}

pub(crate) mod vec3 {
    use super::*;

    serde_vec!(deserialize OrderTypeExtension, "RemoteOrderTypeExtension", max=3);
}
