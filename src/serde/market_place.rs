use std::collections::HashMap;

use serde::Deserialize;
use stock_market_utils::order::{OrderDirection, OrderType};

use crate::market_place::{MarketPalceName, MarketPlace, MarketPlaceId, OrderTypeAbilities};
use crate::order::ComdirectOrderValidity;

#[derive(Deserialize)]
pub(crate) struct JsonResponseMarketplaces {
    pub(crate) values: (VenuesMarketPlaces, )
}

#[derive(Deserialize)]
pub(crate) struct VenuesMarketPlaces {
    pub(crate) venues: Vec<MarketPlace>
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct MarketPlaceDeserializer {
    pub(crate) venue_id: MarketPlaceId,
    pub(crate) name: MarketPalceName,
    #[serde(with = "crate::serde::order_direction::vec2")]
    pub(crate) sides: Vec<OrderDirection>,
    pub(crate) validity_types: Vec<ComdirectOrderValidity>,
    #[serde(with = "crate::serde::order_type::venue_map")]
    pub(crate) order_types: HashMap<OrderType, OrderTypeAbilities>,
}
