use std::collections::HashMap;
use getset::{Getters, Setters};
use serde::{Deserialize, Serialize};
use stock_market_utils::derivative::{ISIN, SYMBOL, WKN};
use stock_market_utils::order::{AuctionType, OrderDirection, OrderType, OrderTypeExtension};
use crate::order::ComdirectOrderValidityType;
use derive_builder::Builder;

new_type_ids!(
    pub struct MarketPlaceId
    pub struct MarketPalceName
    pub struct InstrumentId
);

#[derive(Clone, Debug, Deserialize, PartialEq, Getters)]
#[getset(get = "pub")]
#[serde(rename_all = "camelCase")]
pub struct MarketPlace {
    #[serde(rename = "venueId")]
    id: MarketPlaceId,
    name: MarketPalceName,
    #[serde(rename = "sides")]
    #[serde(with = "crate::serde::order_direction::vec2")]
    order_directions: Vec<OrderDirection>,
    validity_types: Vec<ComdirectOrderValidityType>,
    #[serde(with = "crate::serde::order_type::venue_map")]
    order_types: HashMap<OrderType, OrderTypeAbilities>,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Getters)]
#[getset(get = "pub")]
pub struct OrderTypeAbilities {
    #[serde(default)]
    #[serde(with = "crate::serde::order_type_extension::vec3")]
    order_type_extensions: Vec<OrderTypeExtension>,
    #[serde(default)]
    #[serde(with = "crate::serde::auction_type::vec3")]
    auction_types: Vec<AuctionType>,
}


#[derive(Deserialize)]
pub(crate) struct JsonResponseMarketplaces {
    values: (Dimensions, )
}
#[derive(Deserialize)]
pub(crate) struct Dimensions {
    venues: Vec<MarketPlace>
}

#[derive(Clone, Debug, Default, Serialize, PartialEq, Getters, Setters, Builder)]
#[builder(default)]
#[getset(set = "pub")]
#[builder(setter(strip_option))]
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

impl JsonResponseMarketplaces {
    pub(crate) fn market_places(self) -> Vec<MarketPlace> {
        self.values.0.venues
    }
}
