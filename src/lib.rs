//! This API is based on the official [comdirect_REST_API_Dokumentation.pdf][PDF_docs].
//! For official examples have a look at the [PostMan collection][PostMan].  
//!
//! The [comdirect_api] crate provides Rust bindings to/abstractions over the official REST API
//! of the german [Comdirect][] bank.  
//! The goal of the crate is to provide a low level interface for programmers just working with
//! this API, as well as an abstraction - using the [pecunia] and [wall_street] crate - for
//! those, that need to work with a variety of APIs at the same time.
//!
//! [Comdirect]: https://www.comdirect.de/
//! [PDF_docs]: https://kunde.comdirect.de/cms/media/comdirect_REST_API_Dokumentation.pdf
//! [PostMan]: https://kunde.comdirect.de/cms/media/comdirect_REST_API_Postman_Collection.json
#![feature(never_type)]

macro_rules! new_type_constructors {
    (Deserialize $struct_:ident) => {
        impl<'de> ::serde::Deserialize<'de> for $struct_ {
            fn deserialize<D>(deserializer: D) -> ::std::result::Result<Self, <D as ::serde::Deserializer<'de>>::Error>
                where D: ::serde::Deserializer<'de> {
                Ok(Self(::serde::Deserialize::deserialize(deserializer)?))
            }
        } 
    };
    ($vis:vis deserialize $struct_:ident) => {
        impl $struct_ {
            #[doc(hidden)]
            #[allow(unused)]
            $vis fn deserialize<'de, D>(deserializer: D) -> ::std::result::Result<Self, <D as ::serde::Deserializer<'de>>::Error>
                where D: ::serde::Deserializer<'de> {
                Ok(Self(::serde::Deserialize::deserialize(deserializer)?))
            }
        }
    };
    ($vis:vis de_option $struct_:ident) => {
        impl $struct_ {
            #[doc(hidden)]
            #[allow(unused)]
            $vis fn de_option<'de, D>(deserializer: D) -> ::std::result::Result<Option<Self>, <D as ::serde::Deserializer<'de>>::Error>
                where D: ::serde::Deserializer<'de> {
                let o: Option<String> = ::serde::Deserialize::deserialize(deserializer)?;
                Ok(o.map(Self))
            }
        }
    };
    ($vis:vis new $struct_:ident) => {
        impl $struct_ {
            #[allow(unused)]
            $vis fn new(value: String) -> Self { Self(value) }
        }
    };
    
}

macro_rules! new_type_ids {
    ($($(#[$meta:meta])? $vis:vis struct $struct_:ident)*) => {
        $(
            #[derive(
                Clone, Debug, serde::Serialize, PartialEq, Eq,
                derive_more::Display, derive_more::Into, derive_more::AsRef
            )]
            $(#[$meta])?
            $vis struct $struct_(pub(crate) String);

            impl $struct_ {
                #[allow(unused)]
                $vis fn as_str(&self) -> &str { &self.0 }
                #[allow(unused)]
                $vis fn take(self) -> String { self.0 }
            }
            
            new_type_constructors!(Deserialize $struct_);
            new_type_constructors!(deserialize $struct_);
            new_type_constructors!(de_option $struct_);
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

#[cfg(feature = "raw_interface")]
pub mod interface;
#[cfg(not(feature = "raw_interface"))]
mod interface;
pub mod types;
pub mod error;

#[doc(hidden)]
mod session;
#[doc(hidden)]
mod serde;
