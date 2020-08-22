use finance_utils::iso_codes::units::currency::Currency;
use serde::Deserialize;
use stock_market_utils::derivative::{Derivative, ISIN, SYMBOL, WKN};

use crate::serde::instrument::InstrumentDeserializer;

new_type_ids!(
    pub struct InstrumentId
    pub struct InstrumentName
);

#[derive(Clone, Debug, Deserialize, PartialEq)]
#[serde(from = "crate::serde::instrument::InstrumentDeserializer")]
#[serde(rename_all = "camelCase")]
pub struct Instrument {
    id: InstrumentId,
    name: InstrumentName,
    isin: ISIN,
    symbol: SYMBOL,
    wkn: WKN,
    static_data: StaticInstrumentData,
    derivative_data: Option<DerivativeData>,
    fund_data: Option<FundData>,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct StaticInstrumentData {
    instrument_type: InstrumentType,
    currency: Currency,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum InstrumentType {
    Share,
    Bonds,
    SubscriptionRight,
    ETF,
    ProfitPartCertificate,
    Fund,
    Warrant,
    Certificate,
    NotAvailable,
}

#[derive(Clone, Debug, Deserialize, PartialEq)]
pub struct DerivativeData {
    // todo
}

#[derive(Clone, Debug, Deserialize, PartialEq)]
pub struct FundData {
    // todo
}

impl From<Derivative> for InstrumentId {
    fn from(d: Derivative) -> Self {
        Self(d.into())
    }
}

impl From<InstrumentDeserializer> for Instrument {
    fn from(d: InstrumentDeserializer) -> Self {
        Self {
            id: d.instrument_id,
            name: d.name,
            isin: d.isin,
            symbol: d.mnemonic,
            wkn: d.wkn,
            static_data: d.static_data,
            derivative_data: d.derivative_data,
            fund_data: d.fund_distribution,
        }
    }
}
