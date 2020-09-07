use pecunia::primitive_value::PrimitiveValue;
use pecunia::primitives::F64;
use pecunia::units::NotAUnit;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
struct AmountValue {
    #[serde(serialize_with = "PrimitiveValue::serialize_f64_str")]
    #[serde(deserialize_with = "PrimitiveValue::deserialize_str")]
    value: F64,
    unit: NotAUnit,
}

pub(crate) mod quantity {
    use pecunia::primitives::F64;
    use pecunia::units::NotAUnit;
    use serde::{Serialize, Serializer};

    use super::AmountValue;

    #[allow(unused)]
    pub(crate) fn serialize<S>(&value: &F64, serializer: S) -> Result<S::Ok, S::Error>
        where S: Serializer {
        AmountValue { value, unit: NotAUnit }.serialize(serializer)
    }
}

pub(crate) mod price {
    use pecunia::price::Price;
    use pecunia::primitives::RawPrice;
    use pecunia::serde_with;
    use pecunia::units::currency::Currency;
    use serde::{Deserialize, Serialize};

    #[derive(Serialize, Deserialize)]
    #[serde(remote = "Price")]
    struct PriceDef {
        #[serde(rename = "value")]
        #[serde(getter = "Price::raw_price")]
        raw_price: RawPrice,
        #[serde(rename = "unit")]
        #[serde(getter = "Price::currency")]
        currency: Currency,
    }

    impl From<PriceDef> for Price {
        fn from(pd: PriceDef) -> Self {
            Price::from_raw(pd.raw_price, pd.currency)
        }
    }

    serde_with!(Deserializer for Price as pub(crate) PriceDeserializer with "PriceDef");

    pub(crate) mod option {
        use pecunia::serde_option;

        use super::*;

        serde_option!(deserialize Price as pub(crate) PriceOptionDeserializer with PriceDeserializer);
    }
}

