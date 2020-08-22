use serde::{Deserialize, Deserializer, Serialize};
use stock_market_utils::order::OrderType;

// noinspection RsTypeCheck
pub(crate) fn deserialize<'de, D>(deserializer: D) -> Result<OrderType, D::Error>
    where D: Deserializer<'de> {
    RemoteOrderType::deserialize(deserializer)
}

#[derive(Serialize, Deserialize)]
#[serde(remote = "OrderType")]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
enum RemoteOrderType {
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

pub(crate) mod option {
    use super::*;

    serde_option!(serialize OrderType, "RemoteOrderType");
}

pub(crate) mod venue_map {
    use std::collections::HashMap;

    use serde::de::{MapAccess, Visitor};
    use serde::Deserializer;
    use serde::export::fmt;

    use crate::market_place::OrderTypeAbilities;

    use super::*;

    pub(crate) fn deserialize<'de, D>(deserializer: D) -> Result<HashMap<OrderType, OrderTypeAbilities>, D::Error> where D: Deserializer<'de> {
        deserializer.deserialize_map(MapVisitor)
    }

    #[derive(Deserialize)]
    struct OrderTypeDeserializer(
        #[serde(with = "RemoteOrderType")]
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
