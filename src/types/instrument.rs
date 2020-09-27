use pecunia::units::currency::Currency;
use serde::Deserialize;
use stock_market_utils::derivative::{Derivative, ISIN, SYMBOL, WKN};

new_type_ids!(
    pub struct InstrumentId
    pub struct InstrumentName
);

#[derive(Clone, Debug, Deserialize, PartialEq, getset::Getters)]
#[getset(get = "pub")]
#[serde(rename_all = "camelCase")]
pub struct Instrument {
    #[serde(rename = "instrumentId")]
    id: InstrumentId,
    name: InstrumentName,
    isin: ISIN,
    #[serde(rename = "mnemonic")]
    symbol: SYMBOL,
    wkn: WKN,
    static_data: StaticInstrumentData,
    derivative_data: Option<DerivativeData>,
    fund_data: Option<FundData>,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Eq, getset::Getters)]
#[getset(get = "pub")]
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
