use derive_more::Display;
use serde::{Deserialize, Serialize};
use stock_market_utils::deposit::Deposit;

new_type_ids!(
    pub struct DepositId
    pub struct DepositDisplayId
);

#[derive(Clone, Debug, Serialize, Deserialize, Display, PartialEq, getset::Getters)]
#[getset(get = "pub")]
#[display(fmt = "{}", display_id)]
pub struct ComdirectDeposit {
    #[serde(rename = "depotId")]
    id: DepositId,
    #[serde(rename = "depotDisplayId")]
    display_id: DepositDisplayId,
}

impl Deposit for ComdirectDeposit {
    fn id(&self) -> &str { &self.id.as_str() }
}
