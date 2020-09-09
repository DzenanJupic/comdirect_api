use chrono::{DateTime, Utc};
use pecunia::prelude::*;
use serde::{Deserialize, Serialize, Serializer};
use stock_market_utils::order::{AuctionType, OrderDirection, OrderStatus, OrderType, OrderTypeExtension, OrderValidity};

use crate::deposit::ComdirectDeposit;
use crate::instrument::InstrumentId;
use crate::market_place::MarketPlaceId;

new_type_ids!(
    pub struct OrderId
    pub struct ExecutionId
);

#[derive(Clone, Debug, PartialEq, getset::Getters)]
#[getset(get = "pub")]
pub struct ComdirectOrder<'d> {
    deposit: &'d ComdirectDeposit,
    raw: RawOrder,
}

#[derive(Clone, Debug, Deserialize, PartialEq)]
#[serde(untagged)]
pub enum RawOrder {
    CombinationOrder(RawCombinationOrder),
    SingleOrder(RawSingleOrder),
}

#[derive(Clone, Debug, Deserialize, PartialEq, getset::Getters)]
#[getset(get = "pub")]
#[serde(rename_all = "camelCase")]
pub struct RawCombinationOrder {
    #[serde(rename = "orderId")]
    id: OrderId,
    #[serde(with = "crate::serde::order_type")]
    order_type: OrderType,
    sub_orders: (RawSingleOrder, RawSingleOrder),
}

#[derive(Clone, Debug, Deserialize, PartialEq, getset::Getters)]
#[getset(get = "pub")]
#[serde(rename_all = "camelCase")]
pub struct RawSingleOrder {
    #[serde(rename = "orderId")]
    id: OrderId,
    instrument_id: InstrumentId,

    #[serde(with = "crate::serde::order_type")]
    order_type: OrderType,
    #[serde(default)]
    #[serde(rename = "limitExtension")]
    #[serde(with = "crate::serde::order_type_extension")]
    order_type_extension: OrderTypeExtension,
    #[serde(rename = "side")]
    #[serde(with = "crate::serde::order_direction")]
    direction: OrderDirection,
    // FIXME: #[serde(flatten)] and #[serde(default)] conflict in the current serde version 
    #[serde(skip_deserializing)]
    // github issue: https://github.com/serde-rs/serde/issues/1626
    // therefore this will fail for any order without an explicit order validity
    #[serde(flatten)]
    #[serde(default = "order_validity_default")]
    #[serde(with = "crate::serde::order_validity")]
    validity: OrderValidity,
    #[serde(default)]
    #[serde(rename = "tradingRestriction")]
    #[serde(with = "crate::serde::auction_type")]
    auction: AuctionType,
    #[serde(rename = "orderStatus")]
    #[serde(with = "crate::serde::order_status")]
    status: OrderStatus,

    #[serde(default)]
    #[serde(with = "crate::serde::amount_value::price::option")]
    limit: Option<Price>,
    #[serde(default)]
    #[serde(with = "crate::serde::amount_value::price::option")]
    trigger_limit: Option<Price>,
    #[serde(default)]
    #[serde(rename = "trailingLimitDistAbs")]
    #[serde(with = "crate::serde::amount_value::price::option")]
    absolute_trailing_limit: Option<Price>,
    #[serde(rename = "trailingLimitDistRel")]
    relative_trailing_limit: Option<Percent>,
    #[serde(rename = "creationTimestamp")]
    #[serde(with = "crate::serde::date::time_stamp_string_utc")]
    time: DateTime<Utc>,
    #[serde(default)]
    #[serde(rename = "bestEx")]
    best_execution: bool,

    /// even though the quantity most of the time is just an amount with no currency (NotACurrency), there are cases,
    /// where the quantity is represented by a valid price. This happens when for savings plan (These orders are 
    /// expected to buy a variable amount of stocks for a fixed price) order or partially executed orders.
    #[serde(with = "crate::serde::amount_value::price")]
    quantity: Price,
    /// for explanation, why this is a price, see [quantity](RawSingleOrder::quantity)
    #[serde(default)]
    #[serde(rename = "openQuantity")]
    #[serde(with = "crate::serde::amount_value::price::option")]
    open: Option<Price>,
    /// for explanation, why this is a price, see [quantity](RawSingleOrder::quantity)
    #[serde(default)]
    #[serde(rename = "cancelledQuantity")]
    #[serde(with = "crate::serde::amount_value::price::option")]
    canceled: Option<Price>,
    /// for explanation, why this is a price, see [quantity](RawSingleOrder::quantity)
    #[serde(default)]
    #[serde(rename = "executedQuantity")]
    #[serde(with = "crate::serde::amount_value::price::option")]
    executed: Option<Price>,
    executions: Vec<Execution>,
}

pub(crate) enum OrderChangeValidation<'o, 'd, 'oc> {
    Change(&'oc OrderChange<'o>),
    Delete(&'o ComdirectOrder<'d>),
}

pub(crate) enum OrderChangeAction<'o, 'd> {
    Change(OrderChange<'o>),
    Delete(ComdirectOrder<'d>),
}

#[derive(Debug, Serialize, PartialEq, getset::Getters)]
#[getset(get = "pub with_prefix")]
#[serde(rename_all = "camelCase")]
pub struct OrderChange<'o> {
    #[serde(rename = "orderId")]
    #[serde(serialize_with = "serialize_order_as_id")]
    raw_single_order: &'o mut RawSingleOrder,
    #[getset(get = "pub with_prefix")]
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(with = "crate::serde::amount_value::price::option")]
    limit: Option<Price>,
    #[getset(get = "pub with_prefix")]
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(with = "crate::serde::amount_value::price::option")]
    trigger_limit: Option<Price>,
    #[getset(get = "pub with_prefix")]
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(with = "crate::serde::amount_value::price::option")]
    absolute_trailing_limit: Option<Price>,
    #[getset(get = "pub with_prefix")]
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(serialize_with = "PrimitiveValue::serialize_option_f64_str")]
    relative_trailing_limit: Option<Percent>,
    #[serde(flatten)]
    #[getset(get = "pub with_prefix")]
    #[serde(with = "crate::serde::order_validity::option")]
    #[serde(skip_serializing_if = "Option::is_none")]
    validity: Option<OrderValidity>,
}

#[derive(Clone, Debug, Deserialize, PartialEq)]
pub struct Execution {
    #[serde(rename = "executionId")]
    id: ExecutionId,
    /// indicates the chronological rank in which this [`Execution`] was done relative
    /// to other executions of the same [`Order`]
    rank: u64,
    /// for explanation, why this is a price, see [RawSingleOrder::quantity](RawSingleOrder::quantity)
    #[serde(with = "crate::serde::amount_value::price")]
    quantity: Price,
    #[serde(with = "crate::serde::amount_value::price")]
    price: Price,
    #[serde(with = "crate::serde::date::time_stamp_string_utc")]
    time: DateTime<Utc>,
}

#[derive(Copy, Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub enum ComdirectOrderValidityType {
    #[serde(rename = "GFD")]
    GoodForDay,
    #[serde(rename = "GTD")]
    GoodTillDate,
}

#[derive(Clone, Debug, Default, Serialize, PartialEq, getset::Setters)]
#[getset(set = "pub")]
#[serde(rename_all = "camelCase")]
pub struct OrderFilterParameters {
    #[serde(with = "crate::serde::order_status::option")]
    order_status: Option<OrderStatus>,
    venue_id: Option<MarketPlaceId>,
    #[serde(default)]
    #[serde(rename = "side")]
    #[serde(with = "crate::serde::order_direction::option")]
    order_direction: Option<OrderDirection>,
    #[serde(default)]
    #[serde(with = "crate::serde::order_type::option")]
    order_type: Option<OrderType>,
}

impl OrderId {
    pub fn new_unchecked(value: String) -> Self {
        Self(value)
    }
}

impl<'d> ComdirectOrder<'d> {
    #[inline(always)]
    pub(crate) fn from_raw(raw: RawOrder, deposit: &'d ComdirectDeposit) -> Self {
        Self {
            deposit,
            raw,
        }
    }
    #[inline(always)]
    pub fn into_raw(self) -> RawOrder {
        self.raw
    }

    #[inline(always)]
    pub fn id(&self) -> &OrderId {
        match &self.raw {
            RawOrder::SingleOrder(raw) => &raw.id,
            RawOrder::CombinationOrder(raw) => &raw.id
        }
    }
}

impl RawOrder {
    #[inline(always)]
    pub fn id(&self) -> &OrderId {
        match self {
            RawOrder::CombinationOrder(raw) => raw.id(),
            RawOrder::SingleOrder(raw) => raw.id()
        }
    }
}

macro_rules! update_limit {
    (fn $method_name:ident, $field:ident) => (update_limit!(fn $method_name, $field: Price););
    (fn $method_name:ident, $field:ident: $field_ty:ty) => {
        pub(crate) fn $method_name(&mut self, new_limit: $field_ty) -> Option<$field_ty> {
            std::mem::replace(&mut self.$field, Some(new_limit))
        }
    };
}

// todo
#[allow(unused)]
impl RawSingleOrder {
    update_limit!(fn update_limit, limit);
    update_limit!(fn update_trigger_limit, trigger_limit);
    update_limit!(fn update_absolute_trailing_limit, absolute_trailing_limit);
    update_limit!(fn update_relative_trailing_limit, relative_trailing_limit: Percent);

    pub(crate) fn update_validity(&mut self, new_validity: OrderValidity) -> OrderValidity {
        std::mem::replace(&mut self.validity, new_validity)
    }
}

impl OrderChangeValidation<'_, '_, '_> {
    #[inline(always)]
    pub(crate) fn order_id(&self) -> &OrderId {
        use OrderChangeValidation::*;
        match self {
            Change(change) => change.order_id(),
            Delete(order) => order.id()
        }
    }
}

impl<'o> OrderChange<'o> {
    pub fn from_order0(order: &'o mut ComdirectOrder<'_>) -> Self {
        let raw_single_order = match order.raw {
            RawOrder::SingleOrder(ref mut raw) => raw,
            RawOrder::CombinationOrder(ref mut raw) => &mut raw.sub_orders.0
        };
        Self::from_raw_single_order(raw_single_order)
    }

    pub fn from_order1(order: &'o mut ComdirectOrder<'_>) -> Self {
        let raw_single_order = match order.raw {
            RawOrder::SingleOrder(ref mut raw) => raw,
            RawOrder::CombinationOrder(ref mut raw) => &mut raw.sub_orders.1
        };
        Self::from_raw_single_order(raw_single_order)
    }

    fn from_raw_single_order(raw_single_order: &'o mut RawSingleOrder) -> Self {
        Self {
            raw_single_order,
            limit: None,
            trigger_limit: None,
            absolute_trailing_limit: None,
            relative_trailing_limit: None,
            validity: None,
        }
    }

    pub(crate) fn change_order(self) {
        self.limit
            .map(|limit| self.raw_single_order.limit = Some(limit));
        self.trigger_limit
            .map(|limit| self.raw_single_order.trigger_limit = Some(limit));
        self.absolute_trailing_limit
            .map(|limit| self.raw_single_order.absolute_trailing_limit = Some(limit));
        self.relative_trailing_limit
            .map(|limit| self.raw_single_order.relative_trailing_limit = Some(limit));
        self.validity
            .map(|validity| self.raw_single_order.validity = validity);
    }

    #[inline(always)]
    pub fn order_id(&self) -> &OrderId {
        &self.raw_single_order.id
    }

    option_builder_fn!(
        pub fn limit(Price)
        pub fn trigger_limit(Price)
        pub fn absolute_trailing_limit(Price)
        pub fn relative_trailing_limit(Percent)
        pub fn validity(OrderValidity)
    );
}

fn order_validity_default() -> OrderValidity {
    OrderValidity::OneDay
}

pub(crate) fn serialize_order_as_id<S>(order: &RawSingleOrder, serializer: S) -> Result<S::Ok, S::Error>
    where S: Serializer {
    order.id.serialize(serializer)
}
