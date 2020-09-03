use chrono::NaiveDate;
use pecunia::serde_option;
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use serde::de::{Error, Unexpected};

#[derive(Serialize)]
#[serde(transparent)]
pub(crate) struct NaiveDateSerializer<'a> {
    #[serde(with = "self")]
    pub(crate) remote: &'a NaiveDate
}

#[derive(Deserialize)]
#[serde(transparent)]
pub(crate) struct NaiveDateDeserializer {
    #[serde(with = "self")]
    pub(crate) remote: NaiveDate
}

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

pub(crate) mod option {
    use super::*;
    serde_option!(serialize NaiveDate as pub(crate) NaiveDateOptionSerializer with NaiveDateSerializer);
    serde_option!(deserialize NaiveDate as pub(crate) NaiveDateOptionDeserializer with NaiveDateDeserializer);
}
