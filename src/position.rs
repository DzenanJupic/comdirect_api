use chrono::Utc;
use pecunia::price::{Price, TimeBoundedPrice};
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
    /// for explanation, why this is a price, see [quantity](crate::order::RawSingleOrder::quantity)
    #[serde(with = "crate::serde::amount_value::price")]
    quantity: Price,
    #[serde(with = "crate::serde::time_bounded_price")]
    current_price: TimeBoundedPrice<Utc>,
    #[serde(with = "crate::serde::amount_value::price::option")]
    purchase_price: Option<Price>,
    #[serde(with = "crate::serde::amount_value::price")]
    current_value: Price,
    #[serde(with = "crate::serde::amount_value::price::option")]
    purchase_value: Option<Price>,
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
