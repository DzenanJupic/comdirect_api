use derive_builder::Builder;
use getset::{Getters, Setters};
use pecunia::price::Price;
use pecunia::primitives::F64;
use pecunia::primitives::Percent;
use serde::{Serialize, Serializer};
use stock_market_utils::order::{AuctionType, OrderDirection, OrderType, OrderTypeExtension, OrderValidity};

use crate::deposit::ComdirectDeposit;
use crate::instrument::InstrumentId;
use crate::market_place::MarketPlaceId;

#[derive(Clone, Debug, Serialize, PartialEq)]
#[serde(untagged)]
pub enum OrderOutline<'d, 'i, 'm> {
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
pub struct RawSingleOrderOutline<'d, 'i, 'm> {
    #[serde(rename = "depotId")]
    #[serde(serialize_with = "serialize_deposit_as_id")]
    deposit: &'d ComdirectDeposit,
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
    #[serde(with = "crate::serde::amount_value::price::option")]
    limit: Option<Price>,
    #[builder(default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(with = "crate::serde::amount_value::price::option")]
    trigger_limit: Option<Price>,
    #[builder(default)]
    #[serde(rename = "trailingLimitDistAbs")]
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(with = "crate::serde::amount_value::price::option")]
    absolute_trailing_limit: Option<Price>,
    #[builder(default)]
    #[serde(rename = "trailingLimitDistRel")]
    #[serde(skip_serializing_if = "Option::is_none")]
    relative_trailing_limit: Option<Percent>,
    #[builder(default)]
    #[serde(rename = "bestEx")]
    best_execution: bool,

    #[serde(with = "crate::serde::amount_value::quantity")]
    quantity: F64,
}

impl<'d> OrderOutline<'d, '_, '_> {
    pub fn deposit(&self) -> &'d ComdirectDeposit {
        match self {
            OrderOutline::SingleOrder(order) => order.deposit,
            OrderOutline::CombinationOrder(order) => order.sub_orders.0.deposit
        }
    }
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

pub(crate) fn serialize_deposit_as_id<S>(deposit: &ComdirectDeposit, serializer: S) -> Result<S::Ok, S::Error>
    where S: Serializer {
    deposit.id().serialize(serializer)
}
