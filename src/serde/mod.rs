#[derive(serde::Deserialize)]
pub(crate) struct JsonResponseValues<V> {
    pub(crate) values: Vec<V>
}

/* todo: to deserialize vectors of orders/positions/transactions more efficiently, use seeded deserialization
         the current problem is, that these values don't come as pure json lists, but as JsonResponseValues
         to solve this problem some more coding needs to be done
pub(crate) struct ComdirectOrderVec<'v, 'd> {
    deposit: &'d ComdirectDeposit,
    vec: &'v mut Vec<ComdirectOrder<'d>>,
}

impl<'v, 'd> ComdirectOrderVec<'v, 'd> {
    pub(crate) fn new(deposit: &'d ComdirectDeposit, vec: &'v mut Vec<ComdirectOrder<'d>>) -> Self {
        Self { deposit, vec }
    }
}


impl<'de, 'd> DeserializeSeed<'de> for ComdirectOrderVec<'_, 'd> {
    type Value = ();

    fn deserialize<D>(self, deserializer: D) -> Result<Self::Value, <D as Deserializer<'de>>::Error> where
        D: Deserializer<'de> {
        let raw = RawOrder::deserialize(deserializer)?;
        Ok(self.vec.push(ComdirectOrder::from_raw(raw, self.deposit)))
    }
}*/

pub(crate) mod amount_value;
pub(crate) mod auction_type;
pub(crate) mod date;
pub(crate) mod order_direction;
pub(crate) mod order_type;
pub(crate) mod order_type_extension;
pub(crate) mod order_status;
pub(crate) mod order_validity;
pub(crate) mod time_bounded_price;
