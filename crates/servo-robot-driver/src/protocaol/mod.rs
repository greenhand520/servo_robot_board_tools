//! # Authors
//! greenhand520
//! # Since
//! version: 0.1.0
//! # Date
//! 2026/7/3 13:19

macro_rules! enum_from_u8 {
    ($ty:ty, Unknown, $( $variant:ident = $val:literal ),*) => {
        impl $ty {
            pub fn from_u8(v: u8) -> Self {
                match v {
                    $( $val => Self::$variant, )*
                    _ => Self::Unknown,
                }
            }
        }
    };
}

pub mod battery_state;
pub mod config;
pub mod frame;
pub mod event;
pub mod system;
pub mod sensors;
pub mod command;