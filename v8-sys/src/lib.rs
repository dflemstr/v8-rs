#[cfg(test)]
#[macro_use]
extern crate lazy_static;

include!(concat!(env!("OUT_DIR"), "/ffi.rs"));

pub use ffi::*;
