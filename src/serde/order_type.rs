use pecunia::{serde_option, serde_with};
use serde::{Deserialize, Serialize};
use stock_market_utils::order::OrderType;

#[derive(Serialize, Deserialize)]
#[serde(remote = "OrderType")]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
enum OrderTypeDef {
    Market,
    Limit,
    StopMarket,
    StopLimit,
    TrailingStopMarket,
    TrailingStopLimit,
    #[serde(rename = "ONE_CANCELS_OTHER")]
    OneCancelsTheOther,
    #[serde(rename = "NEXT_ORDER")]
    OneTriggersTheOther,
    Quote,
}

serde_with!(Serializer for OrderType as pub(crate) OrderTypeSerializer with "OrderTypeDef");
serde_with!(Deserializer for OrderType as pub(crate) OrderTypeDeserializer with "OrderTypeDef");

pub(crate) mod option {
    use super::*;

    serde_option!(serialize OrderType as pub(crate) OrderTypeOptionSerializer with OrderTypeSerializer);
}

pub(crate) mod venue_map {
    use std::collections::HashMap;

    use serde::de::{MapAccess, Visitor};
    use serde::Deserializer;
    use serde::export::fmt;

    use crate::api_types::market_place::OrderTypeAbilities;

    use super::*;

    pub(crate) fn deserialize<'de, D>(deserializer: D) -> Result<HashMap<OrderType, OrderTypeAbilities>, D::Error> where D: Deserializer<'de> {
        deserializer.deserialize_map(MapVisitor)
    }

    #[derive(Deserialize)]
    struct OrderTypeDeserializer(
        #[serde(with = "OrderTypeDef")]
        OrderType
    );

    struct MapVisitor;

    impl<'de> Visitor<'de> for MapVisitor {
        type Value = HashMap<OrderType, OrderTypeAbilities>;

        fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
            formatter.write_str("a HashMap<OrderType, OrderTypeAbilities>")
        }

        fn visit_map<V: MapAccess<'de>>(self, mut visitor: V) -> Result<Self::Value, V::Error> {
            let mut values = match visitor.size_hint() {
                Some(size) => HashMap::with_capacity(size),
                None => HashMap::new()
            };

            while let Some((order_type_deserializer, abilities)) = visitor.next_entry::<OrderTypeDeserializer, OrderTypeAbilities>()? {
                values.insert(
                    order_type_deserializer.0,
                    abilities,
                );
            }

            Ok(values)
        }
    }
}
