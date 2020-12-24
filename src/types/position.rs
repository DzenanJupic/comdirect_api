use chrono::Utc;
use pecunia::price::{Price, TimeBoundedPrice};
use pecunia::primitives::F64;
use reqwest::blocking::Response;
use serde::de::DeserializeSeed;
use serde::Deserialize;
use serde_json::Deserializer;
use wall_street::derivative::WKN;

use crate::error::Error;
use crate::types::deposit::ComdirectDeposit;

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
    #[serde(with = "crate::serde::amount_value::quantity")]
    quantity: F64,
    #[serde(with = "crate::serde::time_bounded_price")]
    current_price: TimeBoundedPrice<Utc>,
    #[serde(with = "crate::serde::amount_value::price::option")]
    purchase_price: Option<Price>,
    #[serde(with = "crate::serde::amount_value::price")]
    current_value: Price,
    #[serde(with = "crate::serde::amount_value::price::option")]
    purchase_value: Option<Price>,
}

macro_rules! set_field {
    ($method:ident($field:ident: $field_ty:ty)) => {
        pub(crate) fn $method(&mut self, $field: $field_ty) {
            self.raw.$field = $field;
        }
    };
}

impl<'d> Position<'d> {
    pub(crate) fn from_raw(raw: RawPosition, deposit: &'d ComdirectDeposit) -> Self {
        Self {
            deposit,
            raw,
        }
    }

    pub(crate) fn update_from_response(&mut self, response: Response) -> Result<(), Error> {
        let body = response.bytes()?;
        let mut deserializer = Deserializer::from_slice(&body);
        Ok(self.deserialize(&mut deserializer)?)
    }

    set_field!(set_quantity(quantity: F64));
    set_field!(set_current_price(current_price: TimeBoundedPrice<Utc>));
    set_field!(set_current_value(current_value: Price));
}
