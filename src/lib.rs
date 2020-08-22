macro_rules! new_type_ids {
    ($($(#[$meta:meta])? pub struct $struct_:ident)*) => {
        $(
            #[derive(
                Clone, Debug, serde::Serialize, serde::Deserialize, PartialEq, Eq,
                derive_more::Display, derive_more::Into, derive_more::AsRef
            )]
            $(#[$meta])?
            pub struct $struct_(pub(crate) String);

            impl $struct_ {
                pub fn as_str(&self) -> &str { &self.0 }
                pub fn take(self) -> String { self.0 }
            }
        )*
    };
}

pub mod api_interface;
pub mod deposit;
pub mod error;
pub mod instrument;
pub mod order;
pub mod market_place;
pub mod position;
pub mod transaction;

#[doc(hidden)]
mod tan;
#[doc(hidden)]
mod session;
#[doc(hidden)]
mod serde;
