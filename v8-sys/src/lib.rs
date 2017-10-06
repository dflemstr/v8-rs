#![allow(non_upper_case_globals)]
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]

include!(concat!(env!("OUT_DIR"), "/bindings.rs"));

#[doc(inline)]
pub use root::v8::*;

#[doc(inline)]
pub use root::rust_v8_impls as impls;
