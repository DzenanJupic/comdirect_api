use chrono::{DateTime, NaiveDate, Utc};
use finance_utils::iso_codes::units::currency::Currency;
use finance_utils::iso_codes::units::NotAUnit;
use finance_utils::market_values::f64::F64;
use finance_utils::market_values::percent::Percent;
use finance_utils::market_values::price::Price;
use finance_utils::market_values::unit_value::UnitValue;
use serde::Deserialize;
use stock_market_utils::order::{AuctionType, OrderDirection, OrderStatus, OrderType, OrderTypeExtension};

use crate::instrument::InstrumentId;
use crate::order::{ComdirectOrderValidity, Execution, OrderId};

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct RawOrderDeserializer {
    pub(crate) order_id: OrderId,
    #[serde(with = "crate::serde::date_time::mifid2")]
    pub(crate) creation_timestamp: DateTime<Utc>,
    #[serde(default)]
    pub(crate) leg_number: u64,
    #[serde(default)]
    pub(crate) best_ex: bool,
    #[serde(with = "crate::serde::order_type")]
    pub(crate) order_type: OrderType,
    #[serde(with = "crate::serde::order_status")]
    pub(crate) order_status: OrderStatus,
    #[serde(default)]
    pub(crate) sub_orders: Vec<RawOrderDeserializer>,
    #[serde(with = "crate::serde::order_direction")]
    pub(crate) side: OrderDirection,
    pub(crate) instrument_id: InstrumentId,
    pub(crate) quantity: UnitValue<NotAUnit, F64>,
    #[serde(default)]
    #[serde(with = "crate::serde::order_type_extension")]
    pub(crate) limit_extension: OrderTypeExtension,
    #[serde(default)]
    #[serde(with = "crate::serde::auction_type")]
    pub(crate) trading_restriction: AuctionType,
    pub(crate) limit: Option<UnitValue<Currency, Price>>,
    pub(crate) trigger_limit: Option<UnitValue<Currency, Price>>,
    pub(crate) trailing_limit_dist_rel: Option<Percent>,
    #[serde(default)]
    pub(crate) validity_type: ComdirectOrderValidity,
    #[serde(default)]
    #[serde(with = "crate::serde::naive_date::option")]
    pub(crate) validity: Option<NaiveDate>,
    pub(crate) open_quantity: Option<UnitValue<NotAUnit, F64>>,
    pub(crate) canceled_quantity: Option<UnitValue<NotAUnit, F64>>,
    pub(crate) executed_quantity: Option<UnitValue<NotAUnit, F64>>,
    pub(crate) expected_value: Option<UnitValue<Currency, Price>>,
    pub(crate) executions: Vec<Execution>,
}
