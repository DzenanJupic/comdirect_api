use serde::Deserialize;

use crate::deposit::{DepositDisplayId, DepositId};

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct ComdirectDepositDeserializer {
    pub(crate) depot_id: DepositId,
    pub(crate) depot_display_id: DepositDisplayId,
}