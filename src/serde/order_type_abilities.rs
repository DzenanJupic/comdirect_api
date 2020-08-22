use serde::Deserialize;
use stock_market_utils::order::{AuctionType, OrderTypeExtension};

#[derive(Deserialize)]
pub(crate) struct OrderTypeAbilitiesDeserializer {
    #[serde(default)]
    #[serde(with = "crate::serde::order_type_extension::vec3")]
    pub(crate) limit_extensions: Vec<OrderTypeExtension>,
    #[serde(default)]
    #[serde(with = "crate::serde::auction_type::vec3")]
    pub(crate) trading_restrictions: Vec<AuctionType>,
}
