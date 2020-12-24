use chrono::{DateTime, Utc};
use pecunia::prelude::*;
use serde::{Deserialize, Serialize};
use wall_street::order::{AuctionType, OrderDirection, OrderStatus, OrderType, OrderTypeExtension, OrderValidity};

use execution::Execution;

use crate::types::deposit::ComdirectDeposit;
use crate::types::instrument::InstrumentId;
use crate::types::market_place::MarketPlaceId;

pub mod execution;
pub mod order_change;
pub mod order_outline;

new_type_ids!(
    pub struct OrderId
    pub struct ExecutionId
);

#[derive(Debug, PartialEq, getset::Getters)]
#[getset(get = "pub")]
pub struct Order<'d> {
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
    creation: DateTime<Utc>,
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

impl<'d> Order<'d> {
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

    #[inline(always)]
    pub fn status0(&self) -> OrderStatus {
        use RawOrder::*;
        match self.raw {
            SingleOrder(ref order) => order.status,
            CombinationOrder(ref order) => order.sub_orders.0.status
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

fn order_validity_default() -> OrderValidity {
    OrderValidity::OneDay
}
