use chrono::Local;
use pecunia::iso_codes::units::currency::Currency;
use pecunia::iso_codes::units::NotAUnit;
use pecunia::market_values::f64::F64;
use pecunia::market_values::price::Price;
use pecunia::market_values::unit_value::{TimeBoundedUnitValue, UnitValue};
use serde::Deserialize;
use stock_market_utils::derivative::WKN;

use crate::deposit::ComdirectDeposit;

new_type_ids!(
    pub struct PositionId
);

#[derive(Clone, Debug, PartialEq, getset::Getters, getset::Setters)]
#[getset(get = "pub")]
pub struct Position<'d> {
    deposit: &'d ComdirectDeposit,
    #[getset(set = "pub(crate)")]
    raw: RawPosition,
}

#[derive(Clone, Debug, Deserialize, PartialEq, getset::Getters)]
#[getset(get = "pub")]
#[serde(rename_all = "camelCase")]
pub struct RawPosition {
    #[serde(rename = "positionId")]
    id: PositionId,
    wkn: WKN,
    quantity: UnitValue<NotAUnit, F64>,
    #[serde(with = "crate::serde::time_bounded_unit_value")]
    current_price: TimeBoundedUnitValue<Currency, Price, Local>,
    purchase_price: Option<UnitValue<Currency, Price>>,
    current_value: UnitValue<Currency, Price>,
    purchase_value: Option<UnitValue<Currency, Price>>,
}

impl<'d> Position<'d> {
    pub(crate) fn from_raw(raw: RawPosition, deposit: &'d ComdirectDeposit) -> Self {
        Self {
            deposit,
            raw,
        }
    }
    pub(crate) fn into_raw(self) -> RawPosition {
        self.raw
    }
}
