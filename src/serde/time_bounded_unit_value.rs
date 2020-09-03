/*use chrono::{TimeZone, DateTime};
use pecunia::market_values::unit_value::{TimeBoundedUnitValue, UnitValue};
use serde::{Deserialize, Deserializer};

// noinspection RsTypeCheck
pub(crate) fn deserialize<'de, D, U, V, Tz: TimeZone>(deserializer: D) -> Result<TimeBoundedUnitValue<U, V, Tz>, D::Error>
    where
        D: Deserializer<'de>,
        DateTime<Tz>: From<DateTime<chrono::FixedOffset>>,
        UnitValue<U, V>: Deserialize<'de>, {
    RemoteTimeBoundedUnitValue::deserialize(deserializer)
}

#[derive(Deserialize)]
#[serde(remote = "TimeBoundedUnitValue")]
#[serde(bound(deserialize = "\
DateTime<Tz>: From<DateTime<chrono::FixedOffset>>,\
UnitValue<U, V>: Deserialize<'de>,\
"))]
pub(crate) struct RemoteTimeBoundedUnitValue<U, V, Tz: TimeZone> {
    #[serde(rename = "price")]
    #[serde(getter = "TimeBoundedUnitValue::unit_value")]
    unit_value: UnitValue<U, V>,
    #[serde(rename = "priceDateTime")]
    #[serde(with = "crate::serde::date_time")]
    #[serde(getter = "TimeBoundedUnitValue::time")]
    time: DateTime<Tz>,
}

impl<U, V, Tz: TimeZone> From<RemoteTimeBoundedUnitValue<U, V, Tz>> for TimeBoundedUnitValue<U, V, Tz> {
    fn from(r: RemoteTimeBoundedUnitValue<U, V, Tz>) -> Self {
        let (unit, value) = r.unit_value.into();
        TimeBoundedUnitValue::new(value, unit, r.time)
    }
}*/
// todo: The above code currently does not work, due to a bug in serde (https://github.com/serde-rs/serde/issues/1844)
// therefore I currently use the tweaked, expanded code
// this is an extremely ugly solution, but it works.

use chrono::{DateTime, TimeZone};
use pecunia::market_values::unit_value::{TimeBoundedUnitValue, UnitValue};
use serde::{Deserialize, Deserializer};

pub(crate) fn deserialize<'de, D, U, V, Tz: TimeZone>(
    deserializer: D,
) -> Result<TimeBoundedUnitValue<U, V, Tz>, D::Error>
    where
        D: Deserializer<'de>,
        DateTime<Tz>: From<DateTime<chrono::FixedOffset>>,
        UnitValue<U, V>: Deserialize<'de>,
{
    RemoteTimeBoundedUnitValue::deserialize(deserializer)
}

pub(crate) struct RemoteTimeBoundedUnitValue<U, V, Tz: TimeZone> {
    unit_value: UnitValue<U, V>,
    time: DateTime<Tz>,
}

#[doc(hidden)]
#[allow(non_upper_case_globals, unused_attributes, unused_qualifications)]
const _: () = {
    #[allow(rust_2018_idioms, clippy::useless_attribute)]
    extern crate serde as _serde;
    // noinspection RsUnresolvedReference
    // noinspection DuplicatedCode
    impl<'de, U, V, Tz: TimeZone> RemoteTimeBoundedUnitValue<U, V, Tz>
        where
            DateTime<Tz>: From<DateTime<chrono::FixedOffset>>,
            UnitValue<U, V>: Deserialize<'de>,
    {
        pub(crate) fn deserialize<__D>(
            __deserializer: __D,
        ) -> _serde::export::Result<TimeBoundedUnitValue<U, V, Tz>, __D::Error>
            where
                __D: _serde::Deserializer<'de>,
        {
            match _serde::export::None::<RemoteTimeBoundedUnitValue<U, V, Tz>> {
                _serde::export::Some(RemoteTimeBoundedUnitValue {
                                         unit_value: ref __v0,
                                         time: ref __v1,
                                     }) => {}
                _ => {}
            }
            #[allow(non_camel_case_types)]
            enum __Field {
                __field0,
                __field1,
                __ignore,
            }
            struct __FieldVisitor;
            impl<'de> _serde::de::Visitor<'de> for __FieldVisitor {
                type Value = __Field;
                fn expecting(
                    &self,
                    __formatter: &mut _serde::export::Formatter,
                ) -> _serde::export::fmt::Result {
                    _serde::export::Formatter::write_str(__formatter, "field identifier")
                }
                fn visit_u64<__E>(
                    self,
                    __value: u64,
                ) -> _serde::export::Result<Self::Value, __E>
                    where
                        __E: _serde::de::Error,
                {
                    match __value {
                        0u64 => _serde::export::Ok(__Field::__field0),
                        1u64 => _serde::export::Ok(__Field::__field1),
                        _ => _serde::export::Err(_serde::de::Error::invalid_value(
                            _serde::de::Unexpected::Unsigned(__value),
                            &"field index 0 <= i < 2",
                        )),
                    }
                }
                fn visit_str<__E>(
                    self,
                    __value: &str,
                ) -> _serde::export::Result<Self::Value, __E>
                    where
                        __E: _serde::de::Error,
                {
                    match __value {
                        "price" => _serde::export::Ok(__Field::__field0),
                        "priceDateTime" => _serde::export::Ok(__Field::__field1),
                        _ => _serde::export::Ok(__Field::__ignore),
                    }
                }
                fn visit_bytes<__E>(
                    self,
                    __value: &[u8],
                ) -> _serde::export::Result<Self::Value, __E>
                    where
                        __E: _serde::de::Error,
                {
                    match __value {
                        b"price" => _serde::export::Ok(__Field::__field0),
                        b"priceDateTime" => _serde::export::Ok(__Field::__field1),
                        _ => _serde::export::Ok(__Field::__ignore),
                    }
                }
            }
            impl<'de> _serde::Deserialize<'de> for __Field {
                #[inline]
                fn deserialize<__D>(
                    __deserializer: __D,
                ) -> _serde::export::Result<Self, __D::Error>
                    where
                        __D: _serde::Deserializer<'de>,
                {
                    _serde::Deserializer::deserialize_identifier(__deserializer, __FieldVisitor)
                }
            }
            struct __Visitor<'de, U, V, Tz: TimeZone>
                where
                    DateTime<Tz>: From<DateTime<chrono::FixedOffset>>,
                    UnitValue<U, V>: Deserialize<'de>,
            {
                marker: _serde::export::PhantomData<TimeBoundedUnitValue<U, V, Tz>>,
                lifetime: _serde::export::PhantomData<&'de ()>,
            }
            impl<'de, U, V, Tz: TimeZone> _serde::de::Visitor<'de> for __Visitor<'de, U, V, Tz>
                where
                    DateTime<Tz>: From<DateTime<chrono::FixedOffset>>,
                    UnitValue<U, V>: Deserialize<'de>,
            {
                type Value = TimeBoundedUnitValue<U, V, Tz>;
                fn expecting(
                    &self,
                    __formatter: &mut _serde::export::Formatter,
                ) -> _serde::export::fmt::Result {
                    _serde::export::Formatter::write_str(
                        __formatter,
                        "struct TimeBoundedUnitValue",
                    )
                }
                #[inline]
                fn visit_seq<__A>(
                    self,
                    mut __seq: __A,
                ) -> _serde::export::Result<Self::Value, __A::Error>
                    where
                        __A: _serde::de::SeqAccess<'de>,
                {
                    let __field0 = match match _serde::de::SeqAccess::next_element::<
                        UnitValue<U, V>,
                    >(&mut __seq)
                    {
                        _serde::export::Ok(__val) => __val,
                        _serde::export::Err(__err) => {
                            return _serde::export::Err(__err);
                        }
                    } {
                        _serde::export::Some(__value) => __value,
                        _serde::export::None => {
                            return _serde::export::Err(_serde::de::Error::invalid_length(
                                0usize,
                                &"struct TimeBoundedUnitValue with 2 elements",
                            ));
                        }
                    };
                    let __field1 = match {
                        struct __DeserializeWith<'de, U, V, Tz: TimeZone>
                            where
                                DateTime<Tz>: From<DateTime<chrono::FixedOffset>>,
                                UnitValue<U, V>: Deserialize<'de>,
                        {
                            value: DateTime<Tz>,
                            phantom:
                            _serde::export::PhantomData<TimeBoundedUnitValue<U, V, Tz>>,
                            lifetime: _serde::export::PhantomData<&'de ()>,
                        }
                        impl<'de, U, V, Tz: TimeZone> _serde::Deserialize<'de> for __DeserializeWith<'de, U, V, Tz>
                            where
                                DateTime<Tz>: From<DateTime<chrono::FixedOffset>>,
                                UnitValue<U, V>: Deserialize<'de>,
                        {
                            fn deserialize<__D>(
                                __deserializer: __D,
                            ) -> _serde::export::Result<Self, __D::Error>
                                where
                                    __D: _serde::Deserializer<'de>,
                            {
                                _serde::export::Ok(__DeserializeWith {
                                    value: match crate::serde::date_time::deserialize(
                                        __deserializer,
                                    ) {
                                        _serde::export::Ok(__val) => __val,
                                        _serde::export::Err(__err) => {
                                            return _serde::export::Err(__err);
                                        }
                                    },
                                    phantom: _serde::export::PhantomData,
                                    lifetime: _serde::export::PhantomData,
                                })
                            }
                        }
                        _serde::export::Option::map(
                            match _serde::de::SeqAccess::next_element::<
                                __DeserializeWith<'de, U, V, Tz>,
                            >(&mut __seq)
                            {
                                _serde::export::Ok(__val) => __val,
                                _serde::export::Err(__err) => {
                                    return _serde::export::Err(__err);
                                }
                            },
                            |__wrap| __wrap.value,
                        )
                    } {
                        _serde::export::Some(__value) => __value,
                        _serde::export::None => {
                            return _serde::export::Err(_serde::de::Error::invalid_length(
                                1usize,
                                &"struct TimeBoundedUnitValue with 2 elements",
                            ));
                        }
                    };
                    _serde::export::Ok(_serde::export::Into::<TimeBoundedUnitValue<U, V, Tz>>::into(
                        RemoteTimeBoundedUnitValue {
                            unit_value: __field0,
                            time: __field1,
                        },
                    ))
                }
                #[inline]
                fn visit_map<__A>(
                    self,
                    mut __map: __A,
                ) -> _serde::export::Result<Self::Value, __A::Error>
                    where
                        __A: _serde::de::MapAccess<'de>,
                {
                    let mut __field0: _serde::export::Option<UnitValue<U, V>> =
                        _serde::export::None;
                    let mut __field1: _serde::export::Option<DateTime<Tz>> =
                        _serde::export::None;
                    while let _serde::export::Some(__key) =
                    match _serde::de::MapAccess::next_key::<__Field>(&mut __map) {
                        _serde::export::Ok(__val) => __val,
                        _serde::export::Err(__err) => {
                            return _serde::export::Err(__err);
                        }
                    }
                    {
                        match __key {
                            __Field::__field0 => {
                                if _serde::export::Option::is_some(&__field0) {
                                    return _serde::export::Err(
                                        <__A::Error as _serde::de::Error>::duplicate_field(
                                            "price",
                                        ),
                                    );
                                }
                                __field0 = _serde::export::Some(
                                    match _serde::de::MapAccess::next_value::<UnitValue<U, V>>(
                                        &mut __map,
                                    ) {
                                        _serde::export::Ok(__val) => __val,
                                        _serde::export::Err(__err) => {
                                            return _serde::export::Err(__err);
                                        }
                                    },
                                );
                            }
                            __Field::__field1 => {
                                if _serde::export::Option::is_some(&__field1) {
                                    return _serde::export::Err(
                                        <__A::Error as _serde::de::Error>::duplicate_field(
                                            "priceDateTime",
                                        ),
                                    );
                                }
                                __field1 = _serde::export::Some({
                                    struct __DeserializeWith<'de, U, V, Tz: TimeZone>
                                        where
                                            DateTime<Tz>: From<DateTime<chrono::FixedOffset>>,
                                            UnitValue<U, V>: Deserialize<'de>,
                                    {
                                        value: DateTime<Tz>,
                                        phantom: _serde::export::PhantomData<
                                            TimeBoundedUnitValue<U, V, Tz>,
                                        >,
                                        lifetime: _serde::export::PhantomData<&'de ()>,
                                    }
                                    impl<'de, U, V, Tz: TimeZone> _serde::Deserialize<'de> for __DeserializeWith<'de, U, V, Tz>
                                        where
                                            DateTime<Tz>: From<DateTime<chrono::FixedOffset>>,
                                            UnitValue<U, V>: Deserialize<'de>,
                                    {
                                        fn deserialize<__D>(
                                            __deserializer: __D,
                                        ) -> _serde::export::Result<Self, __D::Error>
                                            where
                                                __D: _serde::Deserializer<'de>,
                                        {
                                            _serde::export::Ok(__DeserializeWith {
                                                value:
                                                match crate::serde::date_time::deserialize(
                                                    __deserializer,
                                                ) {
                                                    _serde::export::Ok(__val) => __val,
                                                    _serde::export::Err(__err) => {
                                                        return _serde::export::Err(__err);
                                                    }
                                                },
                                                phantom: _serde::export::PhantomData,
                                                lifetime: _serde::export::PhantomData,
                                            })
                                        }
                                    }
                                    match _serde::de::MapAccess::next_value::<
                                        __DeserializeWith<'de, U, V, Tz>,
                                    >(&mut __map)
                                    {
                                        _serde::export::Ok(__wrapper) => __wrapper.value,
                                        _serde::export::Err(__err) => {
                                            return _serde::export::Err(__err);
                                        }
                                    }
                                });
                            }
                            _ => {
                                let _ = match _serde::de::MapAccess::next_value::<
                                    _serde::de::IgnoredAny,
                                >(&mut __map)
                                {
                                    _serde::export::Ok(__val) => __val,
                                    _serde::export::Err(__err) => {
                                        return _serde::export::Err(__err);
                                    }
                                };
                            }
                        }
                    }
                    let __field0 = match __field0 {
                        _serde::export::Some(__field0) => __field0,
                        _serde::export::None => {
                            match _serde::private::de::missing_field("price") {
                                _serde::export::Ok(__val) => __val,
                                _serde::export::Err(__err) => {
                                    return _serde::export::Err(__err);
                                }
                            }
                        }
                    };
                    let __field1 = match __field1 {
                        _serde::export::Some(__field1) => __field1,
                        _serde::export::None => {
                            return _serde::export::Err(
                                <__A::Error as _serde::de::Error>::missing_field(
                                    "priceDateTime",
                                ),
                            );
                        }
                    };
                    _serde::export::Ok(_serde::export::Into::<TimeBoundedUnitValue<U, V, Tz>>::into(
                        RemoteTimeBoundedUnitValue {
                            unit_value: __field0,
                            time: __field1,
                        },
                    ))
                }
            }
            const FIELDS: &'static [&'static str] = &["price", "priceDateTime"];
            _serde::Deserializer::deserialize_struct(
                __deserializer,
                "RemoteTimeBoundedUnitValue",
                FIELDS,
                __Visitor {
                    marker: _serde::export::PhantomData::<TimeBoundedUnitValue<U, V, Tz>>,
                    lifetime: _serde::export::PhantomData,
                },
            )
        }
    }
};

impl<U, V, Tz: TimeZone> From<RemoteTimeBoundedUnitValue<U, V, Tz>>
for TimeBoundedUnitValue<U, V, Tz>
{
    fn from(r: RemoteTimeBoundedUnitValue<U, V, Tz>) -> Self {
        let (unit, value) = r.unit_value.into();
        TimeBoundedUnitValue::new(value, unit, r.time)
    }
}