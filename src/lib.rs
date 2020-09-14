//! This API is based on the official [comdirect_REST_API_Dokumentation.pdf][PDF_docs].
//! For official examples have a look at the [PostMan collection][PostMan].  
//!
//! The [comdirect_api] crate provides Rust bindings to/abstractions over the official REST API
//! of the german [Comdirect][] bank.  
//! The goal of the crate is to provide a low level interface for programmers just working with
//! this API, as well as an abstraction - using the [pecunia] and [stock-market-utils] crate - for
//! those, that need to work with a variety of APIs at the same time.
//!
//! [Comdirect]: https://www.comdirect.de/
//! [PDF_docs]: https://kunde.comdirect.de/cms/media/comdirect_REST_API_Dokumentation.pdf
//! [PostMan]: https://kunde.comdirect.de/cms/media/comdirect_REST_API_Postman_Collection.json

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

macro_rules! option_builder_fn {
    ($($vis:vis fn $field:ident($field_ty:ty))*) =>
        (option_builder_fn!($($vis fn $field($field: $field_ty))*););
    ($($vis:vis fn $method_name:ident($field:ident: $field_ty:ty))*) => {
        $(
            $vis fn $method_name(mut self, $field: $field_ty) -> Self {
                self.$field = Some($field);
                self
            }
        )*
    };
}

pub mod api_interface;
pub mod api_types;
pub mod error;

#[doc(hidden)]
mod session;
#[doc(hidden)]
mod serde;
