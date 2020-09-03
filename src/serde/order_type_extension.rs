use serde::{Deserialize,  Serialize};
use stock_market_utils::order::OrderTypeExtension;
use pecunia::{serde_with, serde_vec};

#[derive(Serialize, Deserialize)]
#[serde(remote = "OrderTypeExtension")]
enum OrderTypeExtensionDef {
    #[serde(rename = "AON")]
    AllOrNone,
    #[serde(rename = "IOC")]
    ImmediateOrCancel,
    #[serde(rename = "FOK")]
    FillOrKill,
    #[serde(skip)]
    None,
}

serde_with!(Serializer for OrderTypeExtension as pub(crate) OrderTypeExtensionSerializer with "OrderTypeExtensionDef");
serde_with!(Deserializer for OrderTypeExtension as pub(crate) OrderTypeExtensionDeserializer with "OrderTypeExtensionDef");

pub(crate) mod vec3 {
    use super::*;
    serde_vec!(serialize OrderTypeExtension as pub(crate) OrderTypeExtensionOptionSerializer with OrderTypeExtensionSerializer);
    serde_vec!(deserialize OrderTypeExtension as pub(crate) OrderTypeExtensionOptionDeserializer with OrderTypeExtensionDeserializer, max=3);
}
