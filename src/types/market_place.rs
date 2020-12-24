use std::collections::HashMap;

use derive_builder::Builder;
use getset::{Getters, Setters};
use serde::{Deserialize, Serialize};
use wall_street::derivative::{ISIN, SYMBOL, WKN};
use wall_street::order::{AuctionType, OrderDirection, OrderType, OrderTypeExtension};

use crate::types::instrument::InstrumentId;
use crate::types::order::ComdirectOrderValidityType;

new_type_ids!(
    pub struct MarketPlaceId
    pub struct MarketPlaceName
);

#[derive(Clone, Debug, Deserialize, PartialEq, Getters)]
#[getset(get = "pub")]
#[serde(rename_all = "camelCase")]
pub struct MarketPlace {
    #[serde(rename = "venueId")]
    #[serde(with = "MarketPlaceId")]
    id: MarketPlaceId,
    #[serde(with = "MarketPlaceName")]
    name: MarketPlaceName,
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
    #[serde(rename = "limitExtensions")]
    #[serde(with = "crate::serde::order_type_extension::vec3")]
    order_type_extensions: Vec<OrderTypeExtension>,
    #[serde(rename = "tradingRestrictions")]
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
pub struct MarketPlaceFilterParameters<'a> {
    // todo: serialize as wkn, isin, or symbol
    instrument_id: Option<&'a InstrumentId>,
    #[serde(rename = "WKN")]
    wkn: Option<&'a WKN>,
    #[serde(rename = "ISIN")]
    isin: Option<&'a ISIN>,
    #[serde(rename = "mneomic")]
    symbol: Option<&'a SYMBOL>,
    venue_id: Option<&'a MarketPlaceId>,
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

impl<'a> MarketPlaceFilterParameters<'a> {
    pub fn builder() -> MarketPlaceFilterParametersBuilder<'a> {
        MarketPlaceFilterParametersBuilder::default()
    }
}
