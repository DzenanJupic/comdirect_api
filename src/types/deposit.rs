use derive_more::Display;
use serde::{Deserialize, Serialize};
use stock_market_utils::deposit::Deposit;

new_type_ids!(
    pub struct DepositId
    pub struct DepositDisplayId
);

// todo: create a different kind of deposit, that enforces, that every Position and Order is valid
// this can be done, by creating a DepositPositions and DepositOrders struct (it's important
// to have one for both, so you can keep references to positions, if you update orders)
// then store these structs in a Deposit. More thinking needs to be done...

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
