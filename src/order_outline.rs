use derive_builder::Builder;
use pecunia::iso_codes::units::currency::Currency;
use pecunia::iso_codes::units::NotAUnit;
use pecunia::market_values::f64::F64;
use pecunia::market_values::percent::Percent;
use pecunia::market_values::price::Price;
use pecunia::market_values::unit_value::UnitValue;
use getset::{Getters, Setters};
use serde::Serialize;
use stock_market_utils::order::{AuctionType, OrderDirection, OrderType, OrderTypeExtension, OrderValidity};

use crate::deposit::DepositId;
use crate::instrument::InstrumentId;
use crate::market_place::MarketPlaceId;

#[derive(Clone, Debug, Serialize, PartialEq)]
#[serde(untagged)]
pub enum ComdirectOrderOutline<'d, 'i, 'm> {
    CombinationOrder(RawCombinationOrderOutline<'d, 'i, 'm>),
    SingleOrder(RawSingleOrderOutline<'d, 'i, 'm>),
}

#[derive(Clone, Debug, Serialize, PartialEq, Getters, Setters, Builder)]
#[getset(get = "pub")]
#[getset(set = "pub")]
#[serde(rename_all = "camelCase")]
pub struct RawCombinationOrderOutline<'d, 'i, 'm> {
    #[serde(with = "crate::serde::order_type")]
    order_type: OrderType,
    sub_orders: (RawSingleOrderOutline<'d, 'i, 'm>, RawSingleOrderOutline<'d, 'i, 'm>),
}

#[derive(Clone, Debug, Serialize, PartialEq, Getters, Setters, Builder)]
#[getset(get = "pub")]
#[getset(set = "pub")]
#[builder(setter(strip_option))]
#[serde(rename_all = "camelCase")]
// todo: try without is_none
pub struct RawSingleOrderOutline<'d, 'i, 'm> {
    #[serde(rename = "depotId")]
    deposit_id: &'d DepositId,
    #[serde(skip_serializing_if = "Option::is_none")]
    instrument_id: Option<&'i InstrumentId>,
    #[serde(rename = "venueId")]
    market_place_id: &'m MarketPlaceId,

    #[builder(default)]
    #[serde(with = "crate::serde::order_type")]
    order_type: OrderType,
    #[builder(default)]
    #[serde(rename = "limitExtension")]
    #[serde(with = "crate::serde::order_type_extension")]
    #[serde(skip_serializing_if = "OrderTypeExtension::is_none")]
    order_type_extension: OrderTypeExtension,
    #[builder(default)]
    #[serde(rename = "side")]
    #[serde(with = "crate::serde::order_direction")]
    direction: OrderDirection,
    #[serde(flatten)]
    #[builder(default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(with = "crate::serde::order_validity::option")]
    validity: Option<OrderValidity>,
    #[builder(default)]
    #[serde(rename = "tradingRestriction")]
    #[serde(with = "crate::serde::auction_type")]
    #[serde(skip_serializing_if = "AuctionType::is_all")]
    auction: AuctionType,

    #[builder(default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    limit: Option<UnitValue<Currency, Price>>,
    #[builder(default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    trigger_limit: Option<UnitValue<Currency, Price>>,
    #[builder(default)]
    #[serde(rename = "trailingLimitDistAbs")]
    #[serde(skip_serializing_if = "Option::is_none")]
    absolute_trailing_limit: Option<UnitValue<Currency, Price>>,
    #[builder(default)]
    #[serde(rename = "trailingLimitDistRel")]
    #[serde(skip_serializing_if = "Option::is_none")]
    relative_trailing_limit: Option<Percent>,
    #[builder(default)]
    #[serde(rename = "bestEx")]
    best_execution: bool,

    quantity: UnitValue<NotAUnit, F64>,
}

impl RawCombinationOrderOutline<'_, '_, '_> {
    pub fn builder<'d, 'i, 'm>() -> RawCombinationOrderOutlineBuilder<'d, 'i, 'm> {
        RawCombinationOrderOutlineBuilder::default()
    }
}

impl RawSingleOrderOutline<'_, '_, '_> {
    pub fn builder<'d, 'i, 'm>() -> RawSingleOrderOutlineBuilder<'d, 'i, 'm> {
        RawSingleOrderOutlineBuilder::default()
    }
}
