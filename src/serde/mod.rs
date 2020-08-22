macro_rules! serde_option {
    ($original_type:ident$(<$($generics:tt),*>)?, $remote_type:literal) => {
        pub(crate) mod option {
            use super::*;
            serde_option!(serialize $original_type$(<$($generics),*>)?, $remote_type);
            serde_option!(deserialize $original_type$(<$($generics),*>)?, $remote_type);
        }
    };
    (serialize $original_type:ident$(<$($generics:tt),*>)?, $remote_type:literal) => {
        use serde::{Serialize, Serializer};
        #[derive(Serialize)]
        struct OriginalTypeSerializer<'a, $($($generics),*)?>(
            #[serde(with = $remote_type)]
            &'a $original_type$(<$($generics),*>)?
        );
        pub(crate) fn serialize<$($($generics,)*)? S>(original_type_option: &Option<$original_type$(<$($generics),*>)?>, serializer: S) -> Result<S::Ok, S::Error>
            where S: Serializer {
            match original_type_option {
                Some(ref original_type) => {
                    OriginalTypeSerializer(original_type).serialize(serializer)
                }
                None => Option::<()>::serialize(&None, serializer)
            }
        }
    };
    (deserialize $original_type:ident$(<$($generics:tt),*>)?, $remote_type:literal) => {
        use serde::{Deserialize, Deserializer};
        #[derive(Deserialize)]
        struct OriginalTypeDeserializer<$($($generics),*)?>(
            #[serde(with = $remote_type)]
            $original_type$(<$($generics),*>)?
        );
        pub(crate) fn deserialize<'de, $($($generics,)*)? D>(deserializer: D) -> Result<Option<$original_type$(<$($generics),*>)?>, D::Error>
            where D: Deserializer<'de> {
            Ok(
                <Option<OriginalTypeDeserializer>>::deserialize(deserializer)?
                    .map(|original_type_deserializer| original_type_deserializer.0)
            )
        }
    };
}

macro_rules! serde_vec {
    ($original_type:ident$(<$($generics:tt),*>)?, $remote_type:literal) => {
        pub(crate) mod vec {
            use super::*;
            serde_vec!(serialize $original_type$(<$($generics),*>)?, $remote_type);
            serde_vec!(deserialize $original_type$(<$($generics),*>)?, $remote_type);
        }
    };
    (serialize $original_type:ident$(<$($generics:tt),*>)?, $remote_type:literal) => {
        use serde::{Serialize, Serializer};
        #[derive(Serialize)]
        struct OriginalTypeSerializer<'a, $(<$($generics),*>)?>(
            #[serde(with = $remote_type)]
            &'a $original_type$(<$($generics),*>)?
        );
        pub(crate) fn serialize<$(<$($generics,)*>)? S>(vec: &Vec<$original_type<$(<$($generics),*>)?>, serializer: S) -> Result<S::Ok, S::Error> where S: Serializer {
            let mut seq = serializer.serialize_seq(Some(vec.len()))?;
            for original_type in vec {
                seq.serialize_element(&OriginalTypeSerializer(original_type))?;
            }
            seq.end()
        }
    };
    (deserialize $original_type:ident$(<$($generics:tt),*>)?, $remote_type:literal $(, max=$max:literal)?) => {
        use serde::de::{SeqAccess, Visitor, Deserializer};
        use serde::export::fmt;
        #[derive(Deserialize)]
        struct OriginalTypeDeserializer<$($($generics),*)?>(
            #[serde(with = $remote_type)]
            $original_type<$($($generics),*)?>
        );
        struct VecVisitor;
        impl<'de> Visitor<'de> for VecVisitor {
            type Value = Vec<$original_type<$($($generics),*)?>>;

            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                formatter.write_str(concat!("a Vec<", stringify!($original_type<$($($generics),*>)?),">"))
            }

            fn visit_seq<V: SeqAccess<'de>>(self, mut visitor: V) -> Result<Self::Value, V::Error> {
                let mut values = match visitor.size_hint() {
                    Some(size) => Vec::with_capacity(size),
                    None => {
                        let first = match visitor.next_element::<OriginalTypeDeserializer>()? {
                            Some(type_deserializer) => type_deserializer.0,
                            None => return Ok(Vec::new())
                        };

                        let mut vec = Vec::with_capacity(0 $(+ $max)?);
                        vec.push(first);

                        vec
                    }
                };


                while let Some(type_deserializer) = visitor.next_element::<OriginalTypeDeserializer>()? {
                    values.push(type_deserializer.0)
                }

                Ok(values)
            }
        }
        pub(crate) fn deserialize<'de, $(<$($generics,)*>)? D>(deserializer: D) -> Result<Vec<$original_type<$($($generics),*)?>>, D::Error>
            where D: Deserializer<'de> {
            deserializer.deserialize_seq(VecVisitor)
        }
    };
}

#[derive(serde::Deserialize)]
pub(crate) struct JsonResponseValues<V> {
    pub(crate) values: Vec<V>
}

pub(crate) mod auction_type;
pub(crate) mod date_time;
pub(crate) mod deposit;
pub(crate) mod execution;
pub(crate) mod instrument;
pub(crate) mod market_place;
pub(crate) mod naive_date;
pub(crate) mod order;
pub(crate) mod order_type_abilities;
pub(crate) mod order_direction;
pub(crate) mod order_type;
pub(crate) mod order_type_extension;
pub(crate) mod order_status;
pub(crate) mod position;
pub(crate) mod time_bounded_unit_value;
pub(crate) mod transaction;
