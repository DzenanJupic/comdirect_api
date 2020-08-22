use derive_more::Display;
use serde::Deserialize;
use stock_market_utils::deposit::Deposit;

use crate::serde::deposit::ComdirectDepositDeserializer;

new_type_ids!(
    pub struct DepositId
    pub struct DepositDisplayId
);

#[derive(Clone, Debug, Deserialize, Display, PartialEq)]
#[serde(from = "crate::serde::deposit::ComdirectDepositDeserializer")]
#[display(fmt = "{}", display_id)]
pub struct ComdirectDeposit {
    id: DepositId,
    display_id: DepositDisplayId,
}

impl Deposit for ComdirectDeposit {
    fn id(&self) -> &str { &self.id.as_str() }
}

impl From<ComdirectDepositDeserializer> for ComdirectDeposit {
    fn from(d: ComdirectDepositDeserializer) -> Self {
        Self {
            id: d.depot_id,
            display_id: d.depot_display_id,
        }
    }
}
