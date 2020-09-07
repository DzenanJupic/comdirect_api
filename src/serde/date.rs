//! The different date formats are also described in official documentation in section 3.4.6
//!
//! The Comdirect API knows three types of date (time) stamps
//! 1. DateString       : YYYY-mm-ss
//! 2. DateTimeString   : YYYY-mm-ssTHH:MM:SS+zz
//! 3. TimeStampString  : YYYY-mm-ssTHH:MM:SS,ffffff+zz
//!
//!

pub(crate) mod date_string {
    use chrono::NaiveDate;
    use serde::{Deserialize, Deserializer};
    use serde::de::{Error, Unexpected};

    const DATE_STRING: &str = "%F";
    const HELP_TEXT: &str = "YYYY-mm-dd";

    pub(crate) fn deserialize<'de, D>(deserializer: D) -> Result<NaiveDate, D::Error>
        where
            D: Deserializer<'de> {
        let date = <&str>::deserialize(deserializer)?;
        NaiveDate::parse_from_str(date, DATE_STRING)
            .map_err(|_| D::Error::invalid_value(Unexpected::Str(date), &HELP_TEXT))
    }
}

macro_rules! date_time {
    (mod $mod_:ident <$tz:ident> { $date_time_str:literal, $help_text:literal }) => {
        pub(crate) mod $mod_ {
            use serde::{Deserialize, Deserializer, Serializer, Serialize};
            use serde::de::{Error, Unexpected};
            use chrono::{DateTime, $tz};
        
            #[allow(unused)]
            pub(crate) fn serialize<S>(date: &DateTime<$tz>, serializer: S) -> Result<S::Ok, S::Error>
                where
                    S: Serializer {
                date
                    .format($date_time_str)
                    .to_string()
                    .serialize(serializer)
            }
        
            #[allow(unused)]
            pub(crate) fn deserialize<'de, D>(deserializer: D) -> Result<DateTime<$tz>, D::Error>
                where
                    D: Deserializer<'de> {
                let date = <&str>::deserialize(deserializer)?;
                DateTime::parse_from_str(date, $date_time_str)
                    .map(|d| d.with_timezone(&$tz))
                    .map_err(|_| D::Error::invalid_value(Unexpected::Str(date), &$help_text))
            }
        }
    };
}

date_time!(mod date_time_string_utc <Utc> { "%FT%T%#z", "YYYY-mm-ddTHH:MM:SS+zz" });
date_time!(mod time_stamp_string_utc <Utc> { "%FT%T,%6f%#z", "YYYY-mm-ddTHH:MM:SS,ffffff+zz" });

pub(crate) mod seconds {
    use chrono::{DateTime, Duration, Local};
    use serde::{Deserialize, Deserializer};

    pub(crate) fn deserialize<'de, D>(deserializer: D) -> Result<DateTime<Local>, D::Error>
        where D: Deserializer<'de> {
        let expires_in = i64::deserialize(deserializer)?;
        let expires_at = Local::now() + Duration::seconds(expires_in);
        Ok(expires_at)
    }
}
