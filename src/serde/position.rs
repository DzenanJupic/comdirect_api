use chrono::Local;
use finance_utils::iso_codes::units::currency::Currency;
use finance_utils::iso_codes::units::NotAUnit;
use finance_utils::market_values::f64::F64;
use finance_utils::market_values::price::Price;
use finance_utils::market_values::unit_value::{TimeBoundedUnitValue, UnitValue};
use serde::Deserialize;
use stock_market_utils::derivative::WKN;

use crate::position::PositionId;

#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RawPositionDeserializer {
    pub(crate) position_id: PositionId,
    pub(crate) wkn: WKN,
    pub(crate) quantity: UnitValue<NotAUnit, F64>,
    #[serde(with = "crate::serde::time_bounded_unit_value")]
    pub(crate) current_price: TimeBoundedUnitValue<Currency, Price, Local>,
    pub(crate) purchase_price: Option<UnitValue<Currency, Price>>,
    pub(crate) current_value: UnitValue<Currency, Price>,
    pub(crate) purchase_value: Option<UnitValue<Currency, Price>>,
}
