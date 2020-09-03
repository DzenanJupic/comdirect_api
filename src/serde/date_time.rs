use chrono::{DateTime, Local};
use serde::{Deserialize, Deserializer, Serializer, Serialize};
use serde::de::{Error, Unexpected};

macro_rules! dt {
    ($dt_format:literal, $help:literal) => {
        dt!(serialize $dt_format, $help);
        dt!(deserialize $dt_format, $help);
    };
    (serialize $dt_format:literal, $help:literal) => {
        #[allow(unused)]
        pub(crate) fn serialize<S, Tz: chrono::TimeZone>(date: &DateTime<Tz>, serializer: S) -> Result<S::Ok, S::Error>
            where
                S: Serializer,
                <Tz as chrono::TimeZone>::Offset: std::fmt::Display {
            date
                .format($dt_format)
                .to_string()
                .serialize(serializer)
        }
    };
    (deserialize $dt_format:literal, $help:literal) => {
        #[allow(unused)]
        pub(crate) fn deserialize<'de, D, Tz: chrono::TimeZone>(deserializer: D) -> Result<DateTime<Tz>, D::Error>
            where
                D: Deserializer<'de>,
                DateTime<Tz>: From<DateTime<chrono::FixedOffset>> {
            let s = <&'de str>::deserialize(deserializer)?;
            DateTime::parse_from_str(s, $dt_format)
                .map(|dt| DateTime::<Tz>::from(dt))
                .map_err(|_| D::Error::invalid_value(
                    Unexpected::Str(s),
                    &$help,
                ))
        }
    };
}

dt!(deserialize "%FT%T%#z", "YYYY-mm-ddTHH:MM:SS+z");

pub(crate) mod mifid2 {
    use super::*;

    dt!("%FT%T,%6f%#z", "YYYY-mm-ddTHH:MM:SS,ffffff+zz");
}

pub(crate) mod seconds {
    use chrono::Duration;

    use super::*;

    pub(crate) fn deserialize<'de, D>(deserializer: D) -> Result<DateTime<Local>, D::Error>
        where D: Deserializer<'de> {
        let expires_in = i64::deserialize(deserializer)?;
        let expires_at = Local::now() + Duration::seconds(expires_in);
        Ok(expires_at)
    }
}
