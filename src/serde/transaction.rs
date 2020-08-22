use chrono::NaiveDate;
use finance_utils::iso_codes::units::currency::Currency;
use finance_utils::market_values::price::Price;
use finance_utils::market_values::unit_value::UnitValue;
use serde::Deserialize;

use crate::transaction::{BookingStatus, TransactionDirection, TransactionId, TransactionType};

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RawTransactionDeserializer {
    pub(crate) transaction_id: Option<TransactionId>,
    pub(crate) booking_status: BookingStatus,
    #[serde(with = "crate::serde::naive_date")]
    pub(crate) booking_date: NaiveDate,
    pub(crate) transaction_value: UnitValue<Currency, Price>,
    pub(crate) transaction_direction: TransactionDirection,
    pub(crate) transaction_type: TransactionType,
}
