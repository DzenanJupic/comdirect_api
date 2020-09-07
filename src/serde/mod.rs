#[derive(serde::Deserialize)]
pub(crate) struct JsonResponseValues<V> {
    pub(crate) values: Vec<V>
}

pub(crate) mod amount_value;
pub(crate) mod auction_type;
pub(crate) mod date;
pub(crate) mod naive_date;
pub(crate) mod order_direction;
pub(crate) mod order_type;
pub(crate) mod order_type_extension;
pub(crate) mod order_status;
pub(crate) mod order_validity;
pub(crate) mod time_bounded_price;
