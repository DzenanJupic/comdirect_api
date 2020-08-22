use std::collections::HashMap;

use serde::Deserialize;
use stock_market_utils::derivative::{ISIN, SYMBOL, WKN};
use stock_market_utils::order::{AuctionType, OrderDirection, OrderType, OrderTypeExtension};

use crate::order::ComdirectOrderValidity;
use crate::serde::market_place::MarketPlaceDeserializer;
use crate::serde::order_type_abilities::OrderTypeAbilitiesDeserializer;

new_type_ids!(
    pub struct MarketPlaceId
    pub struct MarketPalceName
    pub struct InstrumentId
);

#[derive(Clone, Debug, Deserialize, PartialEq)]
#[serde(from = "crate::serde::market_place::MarketPlaceDeserializer")]
pub struct MarketPlace {
    id: MarketPlaceId,
    name: MarketPalceName,
    order_directions: Vec<OrderDirection>,
    validity_types: Vec<ComdirectOrderValidity>,
    order_types: HashMap<OrderType, OrderTypeAbilities>,
}

#[derive(Clone, Debug, Deserialize, PartialEq)]
#[serde(from = "crate::serde::order_type_abilities::OrderTypeAbilitiesDeserializer")]
pub struct OrderTypeAbilities {
    order_type_extensions: Vec<OrderTypeExtension>,
    auction_types: Vec<AuctionType>,
}

#[derive(Clone, Debug, Default, serde::Serialize, PartialEq, getset::Setters)]
#[getset(set = "pub")]
#[serde(rename_all = "camelCase")]
pub struct OrderDimensionsFilterParameters {
    instrument_id: Option<InstrumentId>,
    #[serde(rename = "WKN")]
    wkn: Option<WKN>,
    #[serde(rename = "ISIN")]
    isin: Option<ISIN>,
    #[serde(rename = "mneomic")]
    symbol: Option<SYMBOL>,
    venue_id: Option<MarketPlaceId>,
    #[serde(rename = "side")]
    #[serde(with = "crate::serde::order_direction::option")]
    order_direction: Option<OrderDirection>,
    #[serde(with = "crate::serde::order_type::option")]
    order_type: Option<OrderType>,
}

impl From<MarketPlaceDeserializer> for MarketPlace {
    fn from(d: MarketPlaceDeserializer) -> Self {
        Self {
            id: d.venue_id,
            name: d.name,
            order_directions: d.sides,
            validity_types: d.validity_types,
            order_types: d.order_types,
        }
    }
}

impl From<OrderTypeAbilitiesDeserializer> for OrderTypeAbilities {
    fn from(d: OrderTypeAbilitiesDeserializer) -> Self {
        Self {
            order_type_extensions: d.limit_extensions,
            auction_types: d.trading_restrictions,
        }
    }
}
