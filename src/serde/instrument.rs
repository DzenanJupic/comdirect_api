use serde::Deserialize;
use stock_market_utils::derivative::{ISIN, SYMBOL, WKN};

use crate::instrument::{DerivativeData, FundData, InstrumentId, InstrumentName, StaticInstrumentData};

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct InstrumentDeserializer {
    pub(crate) instrument_id: InstrumentId,
    pub(crate) wkn: WKN,
    pub(crate) isin: ISIN,
    pub(crate) mnemonic: SYMBOL,
    pub(crate) name: InstrumentName,
    pub(crate) static_data: StaticInstrumentData,
    pub(crate) derivative_data: Option<DerivativeData>,
    pub(crate) fund_distribution: Option<FundData>,
}
