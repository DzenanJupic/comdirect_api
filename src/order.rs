use chrono::{DateTime, Utc};
use pecunia::iso_codes::units::currency::Currency;
use pecunia::market_values::f64::F64;
use pecunia::market_values::percent::Percent;
use pecunia::market_values::price::Price;
use pecunia::market_values::unit_value::UnitValue;
use serde::{Deserialize, Serialize};
use stock_market_utils::order::{AuctionType, OrderDirection, OrderStatus, OrderType, OrderTypeExtension, OrderValidity};

use crate::deposit::ComdirectDeposit;
use crate::instrument::InstrumentId;
use crate::market_place::MarketPlaceId;
use pecunia::iso_codes::units::NotAUnit;

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
    #[serde(flatten)]
    #[serde(with = "crate::serde::order_validity")]
    validity: OrderValidity,
    #[serde(default)]
    #[serde(rename = "tradingRestriction")]
    #[serde(with = "crate::serde::auction_type")]
    auction: AuctionType,
    #[serde(rename = "orderStatus")]
    #[serde(with = "crate::serde::order_status")]
    status: OrderStatus,

    limit: Option<UnitValue<Currency, Price>>,
    trigger_limit: Option<UnitValue<Currency, Price>>,
    #[serde(rename = "trailingLimitDistAbs")]
    absolute_trailing_limit: Option<UnitValue<Currency, Price>>,
    #[serde(rename = "trailingLimitDistRel")]
    relative_trailing_limit: Option<Percent>,
    #[serde(rename = "creationTimestamp")]
    #[serde(with = "crate::serde::date_time::mifid2")]
    time: DateTime<Utc>,
    #[serde(rename = "bestEx")]
    best_execution: bool,

    quantity: UnitValue<NotAUnit, F64>,
    #[serde(rename = "openQuantity")]
    open: Option<UnitValue<NotAUnit, F64>>,
    #[serde(rename = "cancelledQuantity")]
    canceled: Option<UnitValue<NotAUnit, F64>>,
    #[serde(rename = "executedQuantity")]
    executed: Option<UnitValue<NotAUnit, F64>>,

    executions: Vec<Execution>,
}

#[derive(Clone, Debug, Deserialize, PartialEq)]
pub struct Execution {
    #[serde(rename = "executionId")]
    id: ExecutionId,
    /// indicates the chronological rank in which this [`Execution`] was done relative
    /// to other executions of the same [`Order`]
    rank: u64,
    quantity: F64,
    price: UnitValue<Currency, Price>,
    #[serde(with = "crate::serde::date_time::mifid2")]
    time: DateTime<Utc>,
}

#[derive(Copy, Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub enum ComdirectOrderValidityType {
    #[serde(rename = "GFD")]
    GoodForDay,
    #[serde(rename = "GTD")]
    GoodTillDate
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
