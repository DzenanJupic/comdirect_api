use chrono::NaiveDate;
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use serde::de::{Error, Unexpected};

pub(crate) fn serialize<S>(date: &NaiveDate, serializer: S) -> Result<S::Ok, S::Error> where S: Serializer {
    date
        .format("%F") // YYYY-mm-dd
        .to_string()
        .serialize(serializer)
}

pub(crate) fn deserialize<'de, D>(deserializer: D) -> Result<NaiveDate, D::Error>
    where D: Deserializer<'de> {
    let s = <&'de str>::deserialize(deserializer)?;
    NaiveDate::parse_from_str(&s, "%F") // YYYY-mm-dd
        .map_err(|_| D::Error::invalid_value(
            Unexpected::Str(&s),
            &"YYYY-MM-DD")
        )
}
serde_option!(NaiveDate, "crate::serde::naive_date");
