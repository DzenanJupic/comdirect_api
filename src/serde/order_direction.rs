use serde::{Deserialize, Deserializer, Serialize};
use stock_market_utils::order::OrderDirection;

// noinspection RsTypeCheck
pub(crate) fn deserialize<'de, D>(deserializer: D) -> Result<OrderDirection, D::Error>
    where D: Deserializer<'de> {
    RemoteOrderDirection::deserialize(deserializer)
}

#[derive(Serialize, Deserialize)]
#[serde(remote = "OrderDirection")]
#[serde(rename_all = "UPPERCASE")]
enum RemoteOrderDirection {
    Buy,
    Sell,
}

pub(crate) mod option {
    use super::*;

    serde_option!(serialize OrderDirection, "RemoteOrderDirection");
}

pub(crate) mod vec2 {
    use super::*;

    serde_vec!(deserialize OrderDirection, "RemoteOrderDirection", max=2);
}
