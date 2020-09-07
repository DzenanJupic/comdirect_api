use chrono::{DateTime, Utc};
use pecunia::price::Price;
use pecunia::primitives::Percent;
use serde::{Deserialize, Serialize};
use stock_market_utils::order::{AuctionType, OrderDirection, OrderStatus, OrderType, OrderTypeExtension};

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
    // github issue: https://github.com/serde-rs/serde/issues/1626
    // therefore this will fail for any order without an explicit order validity
    // #[serde(flatten)]
    // #[serde(default = "order_validity_default")]
    // #[serde(with = "crate::serde::order_validity")]
    // validity: OrderValidity,
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

// FIXME: function is ok, but currently unused (see RawSingleOrder.validity) 
// fn order_validity_default() -> OrderValidity {
//     OrderValidity::OneDay
// }

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
    pub(crate) fn from_raw(raw: RawOrder, deposit: &'d ComdirectDeposit) -> Self {
        Self {
            deposit,
            raw,
        }
    }
    pub fn into_raw(self) -> RawOrder {
        self.raw
    }
}

impl RawOrder {
    pub fn id(&self) -> &OrderId {
        match self {
            RawOrder::CombinationOrder(raw) => raw.id(),
            RawOrder::SingleOrder(raw) => raw.id()
        }
    }
}
