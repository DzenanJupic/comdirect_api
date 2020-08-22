use chrono::{DateTime, Utc};
use finance_utils::iso_codes::units::currency::Currency;
use finance_utils::iso_codes::units::NotAUnit;
use finance_utils::market_values::f64::F64;
use finance_utils::market_values::price::Price;
use finance_utils::market_values::unit_value::UnitValue;
use serde::Deserialize;

use crate::order::ExecutionId;

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct ExecutionDeserializer {
    pub(crate) execution_id: ExecutionId,
    pub(crate) execution_number: u64,
    pub(crate) quantity: UnitValue<NotAUnit, F64>,
    pub(crate) execution_price: UnitValue<Currency, Price>,
    #[serde(with = "crate::serde::date_time::mifid2")]
    pub(crate) execution_timestamp: DateTime<Utc>,
}
