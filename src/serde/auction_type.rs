use serde::{Deserialize, Serialize};
use stock_market_utils::order::AuctionType;
use pecunia::{serde_with, serde_vec};

#[derive(Serialize, Deserialize)]
#[serde(remote = "AuctionType")]
enum AuctionTypeDef {
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

serde_with!(Serializer for AuctionType as pub(crate) AuctionTypeSerializer with "AuctionTypeDef");
serde_with!(Deserializer for AuctionType as pub(crate) AuctionTypeDeserializer with "AuctionTypeDef");

pub(crate) mod vec3 {
    use super::*;
    serde_vec!(deserialize AuctionType as pub(crate) AuctionTypeVecDeserializer with AuctionTypeDeserializer, max=3);
}
