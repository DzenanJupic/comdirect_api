use chrono::{DateTime, Utc};
use finance_utils::iso_codes::units::currency::Currency;
use finance_utils::market_values::f64::F64;
use finance_utils::market_values::MarketValue;
use finance_utils::market_values::percent::Percent;
use finance_utils::market_values::price::Price;
use finance_utils::market_values::unit_value::UnitValue;
use serde::Deserialize;
use serde::export::TryFrom;
use stock_market_utils::order::{AuctionType, OrderDirection, OrderStatus, OrderType, OrderTypeExtension, OrderValidity};

use crate::deposit::ComdirectDeposit;
use crate::instrument::InstrumentId;
use crate::market_place::MarketPlaceId;
use crate::serde::execution::ExecutionDeserializer;
use crate::serde::order::RawOrderDeserializer;

new_type_ids!(
    pub struct OrderId
    pub struct ExecutionId
);

// todo: implement a more efficient way of deserializing a Vec<ComdirectOrder>
// currently this will:
//  1. allocate a Vec
//  2. deserialize RawComdirectOrder[s] into it
//  3. allocate a new Vec
//  4. convert the raw orders to orders into it
//
// this can be made more efficient, since response impls Read and it's possible to deserialize
// from a byte reader. This would skip allocating a second vector and iterating over each raw order
//
// the whole process can be made generic, so it's also usable for Position or Transaction

#[derive(Clone, Debug, PartialEq)]
pub struct ComdirectOrder<'d> {
    deposit: &'d ComdirectDeposit,
    raw: RawComdirectOrder,
}

#[derive(Clone, Debug, Deserialize, PartialEq)]
#[serde(try_from = "crate::serde::order::RawOrderDeserializer")]
pub struct RawComdirectOrder {
    id: OrderId,
    /// if this order is a sub-order, this field indicates the rank of this order.
    /// I.e. in an one-triggers-the-other order, this could indicated, that this order
    /// comes second
    /// todo: 0 or 1 based?
    child_rank: u64,
    instrument_id: InstrumentId,

    order_type: OrderType,
    order_type_extensions: OrderTypeExtension,
    direction: OrderDirection,
    validity: OrderValidity,
    auction: AuctionType,
    status: OrderStatus,

    // todo: maybe make an enum out of the limits?
    limit: Option<UnitValue<Currency, Price>>,
    trigger_limit: Option<UnitValue<Currency, Price>>,
    trailing_limit: Option<Percent>,
    time: DateTime<Utc>,
    best_execution: bool,

    sub_orders: Vec<RawComdirectOrder>,

    quantity: F64,
    open: F64,
    canceled: F64,
    executed: F64,

    expected_value: Option<UnitValue<Currency, Price>>,
    executions: Vec<Execution>,
}


#[derive(Clone, Debug, Deserialize, PartialEq)]
#[serde(from = "crate::serde::execution::ExecutionDeserializer")]
pub struct Execution {
    id: ExecutionId,
    /// indicates the chronological rank in which this [`Execution`] was done relative
    /// to other executions of the same [`Order`]
    rank: u64,
    quantity: F64,
    price: UnitValue<Currency, Price>,
    time: DateTime<Utc>,
}

#[derive(Copy, Clone, Debug, Deserialize, PartialEq, Eq)]
pub enum ComdirectOrderValidity {
    #[serde(rename = "GFD")]
    OneDay,
    #[serde(rename = "GTD")]
    TillDate,
}

#[derive(Clone, Debug, Default, serde::Serialize, PartialEq, getset::Setters)]
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
    pub(crate) fn from_raw(raw: RawComdirectOrder, deposit: &'d ComdirectDeposit) -> Self {
        Self {
            deposit,
            raw,
        }
    }
}

impl TryFrom<RawOrderDeserializer> for RawComdirectOrder {
    type Error = &'static str;

    fn try_from(d: RawOrderDeserializer) -> Result<Self, Self::Error> {
        let mut sub_orders = Vec::with_capacity(d.sub_orders.len());
        for sub_order in d.sub_orders {
            sub_orders.push(Self::try_from(sub_order)?);
        }

        let validity = {
            use crate::order::ComdirectOrderValidity::*;
            match d.validity_type {
                OneDay => OrderValidity::OneDay,
                TillDate => match d.validity {
                    Some(date) => OrderValidity::TillDate(date),
                    None => return Err(
                        "comdirect api returned an invalid order object \
                         (order validity is TillDate, but no date was specified)"
                    )
                }
            }
        };

        Ok(Self {
            id: d.order_id,
            child_rank: d.leg_number,
            instrument_id: d.instrument_id,
            order_type: d.order_type,
            order_type_extensions: d.limit_extension,
            direction: d.side,
            validity,
            auction: d.trading_restriction,
            status: d.order_status,
            limit: d.limit,
            trigger_limit: d.trigger_limit,
            trailing_limit: d.trailing_limit_dist_rel,
            time: d.creation_timestamp,
            best_execution: d.best_ex,
            sub_orders,
            quantity: d.quantity.copy_value(),
            open: option_to_value(d.open_quantity),
            canceled: option_to_value(d.canceled_quantity),
            executed: option_to_value(d.executed_quantity),
            expected_value: d.expected_value,
            executions: d.executions,
        })
    }
}

impl From<ExecutionDeserializer> for Execution {
    fn from(d: ExecutionDeserializer) -> Self {
        Execution {
            id: d.execution_id,
            rank: d.execution_number,
            quantity: *d.quantity.value(),
            price: d.execution_price,
            time: d.execution_timestamp,
        }
    }
}

impl Default for ComdirectOrderValidity {
    fn default() -> Self { Self::OneDay }
}

fn option_to_value<'de, U, V>(option: Option<UnitValue<U, V>>) -> V
    where V: MarketValue<'de> {
    match option {
        Some(v) => v.copy_value(),
        None => V::from(0.0)
    }
}
