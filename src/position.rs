use chrono::Local;
use finance_utils::iso_codes::units::currency::Currency;
use finance_utils::iso_codes::units::NotAUnit;
use finance_utils::market_values::f64::F64;
use finance_utils::market_values::price::Price;
use finance_utils::market_values::unit_value::{TimeBoundedUnitValue, UnitValue};
use serde::Deserialize;
use stock_market_utils::derivative::WKN;

use crate::deposit::ComdirectDeposit;
use crate::serde::position::RawPositionDeserializer;

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
#[serde(from = "crate::serde::position::RawPositionDeserializer")]
pub struct RawPosition {
    id: PositionId,
    wkn: WKN,
    quantity: UnitValue<NotAUnit, F64>,
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

impl From<RawPositionDeserializer> for RawPosition {
    fn from(d: RawPositionDeserializer) -> Self {
        Self {
            id: d.position_id,
            wkn: d.wkn,
            quantity: d.quantity,
            current_price: d.current_price,
            purchase_price: d.purchase_price,
            current_value: d.current_value,
            purchase_value: d.purchase_value,
        }
    }
}
