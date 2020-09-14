use chrono::Utc;
use pecunia::price::{Price, TimeBoundedPrice};
use serde::{Deserialize, Deserializer};
use serde::de::DeserializeSeed;

use crate::position::Position;

impl<'de> DeserializeSeed<'de> for &mut Position<'_> {
    type Value = ();

    fn deserialize<D>(self, deserializer: D) -> Result<Self::Value, <D as Deserializer<'de>>::Error> where
        D: Deserializer<'de> {
        let update_position = UpdatePosition::deserialize(deserializer)?;

        self.set_quantity(update_position.quantity);
        self.set_current_price(update_position.current_price);
        self.set_current_value(update_position.current_value);

        Ok(())
    }
}

#[derive(serde::Deserialize)]
#[serde(rename_all = "camelCase")]
struct UpdatePosition {
    #[serde(with = "crate::serde::amount_value::price")]
    quantity: Price,
    #[serde(with = "crate::serde::time_bounded_price")]
    current_price: TimeBoundedPrice<Utc>,
    #[serde(with = "crate::serde::amount_value::price")]
    current_value: Price,
}
