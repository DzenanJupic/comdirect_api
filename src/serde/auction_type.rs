use serde::{Deserialize, Deserializer};
use stock_market_utils::order::AuctionType;

// noinspection RsTypeCheck
pub(crate) fn deserialize<'de, D>(deserializer: D) -> Result<AuctionType, D::Error>
    where D: Deserializer<'de> {
    RemoteAuctionType::deserialize(deserializer)
}

#[derive(Deserialize)]
#[serde(remote = "AuctionType")]
enum RemoteAuctionType {
    #[serde(rename = "OAO")]
    OpeningAuctionOnly,
    #[serde(rename = "AO")]
    AuctionOnly,
    #[serde(rename = "CAO")]
    ClosingAuctionOnly,
    #[serde(skip)]
    ContinuesTradingOnly,
    #[serde(skip)]
    All,
}

pub(crate) mod vec3 {
    use super::*;

    serde_vec!(deserialize AuctionType, "RemoteAuctionType", max=3);
}
