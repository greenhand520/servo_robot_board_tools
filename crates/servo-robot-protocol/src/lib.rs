//! servo-robot-protocol - STM32 通信协议定义
//!
//! 支持 no_std + alloc，可用于 PC 和嵌入式平台。

#![cfg_attr(not(feature = "std"), no_std)]

extern crate alloc;

pub mod crc;
pub mod error;
pub mod frame;
pub mod event;
pub mod config;
pub mod system;
pub mod power;
pub mod battery_state;
pub mod imu;
pub mod thermal;

/// 通用枚举转换宏
#[macro_export]
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
