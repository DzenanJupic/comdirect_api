use serde::{Deserialize,  Serialize};
use stock_market_utils::order::OrderDirection;
use pecunia::{serde_with, serde_option, serde_vec};

#[derive(Serialize, Deserialize)]
#[serde(remote = "OrderDirection")]
#[serde(rename_all = "UPPERCASE")]
enum OrderDirectionDef {
    Buy,
    Sell,
}

serde_with!(Serializer for OrderDirection as pub(crate) OrderDirectionSerializer with "OrderDirectionDef");
serde_with!(Deserializer for OrderDirection as pub(crate) OrderDirectionDeserializer with "OrderDirectionDef");

pub(crate) mod option {
    use super::*;
    serde_option!(serialize OrderDirection as pub(crate) OrderDirectionOptionSerializer with OrderDirectionSerializer);
    serde_option!(deserialize OrderDirection as pub(crate) OrderDirectionOptionDeserializer with OrderDirectionDeserializer);
}

pub(crate) mod vec2 {
    use super::*;
    serde_vec!(serialize OrderDirection as pub(crate) OrderDirectionVecSerializer with OrderDirectionSerializer);
    serde_vec!(deserialize OrderDirection as pub(crate) OrderDirectionVecDeserializer with OrderDirectionDeserializer, max=2);
}
