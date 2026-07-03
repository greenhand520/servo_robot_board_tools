//! # Authors
//! greenhand520
//! # Since
//! version: 0.1.0
//! # Date
//! 2026/7/3 13:40
//!
//! 上下行采用完全一样的帧格式，格式如下
//! ┌─────────────────────────────────────────────────────────────────┐
//! │                     通用帧格式 (所有方向统一)                       │
//! │                                                                 │
//! │  ┌──────┬──────┬──────┬───────────────┬──────┐                  │
//! │  │ HEAD │ TYPE │ LEN  │   PAYLOAD     │ CRC  │                  │
//! │  │ 1B   │ 1B   │ 2B   │   0~255B      │ 2B   │                  │
//! │  └──────┴──────┴──────┴───────────────┴──────┘                  │
//! │                                                                 │
//! │  HEAD:    0xAA (固定帧头)                                         │
//! │  TYPE:    消息类型 (见下表)                                        │
//! │  LEN:     payload 长度 (小端 uint16)                             │
//! │  PAYLOAD: 数据内容                                               │
//! │  CRC:     CRC-16/CCITT 校验 (从 TYPE 到 PAYLOAD 末尾)             │
//! │                                                                 │
//! │  帧头之外的数据如果出现 0xAA，不需要转义                              │
//! │  因为接收方是先找帧头，再按 LEN 读取固定长度                           │
//! │  只要 LEN 正确 + CRC 通过，就不会误判                               │
//! └─────────────────────────────────────────────────────────────────┘

use crate::protocaol::battery_state::BatteryState;
use crate::protocaol::config::{BoardConfigSnapshot, ConfigType};
use crate::protocaol::event::BoardEvent;
use crate::protocaol::sensors::{ImuData, PowerData, ThermalData};
use crate::protocaol::system::SystemInfo;

// ═══ 帧头与类型 ═══
pub const FRAME_HEAD: u8 = 0xAA;

// 上行 (STM32 → PC)
pub const TYPE_IMU: u8 = 0x01;
pub const TYPE_POWER: u8 = 0x02;
pub const TYPE_THERMAL: u8 = 0x03;
pub const TYPE_CONFIG: u8 = 0x04;
pub const TYPE_BATTERY: u8 = 0x05;
pub const TYPE_SYSTEM: u8 = 0x06;
pub const TYPE_EVENT: u8 = 0x07;

// 下行 (PC → STM32)
pub const TYPE_CFG_WRITE: u8 = 0x80;
pub const TYPE_CFG_QUERY: u8 = 0x81;
// 具体命令和
pub const TYPE_CMD: u8 = 0x82;

// 应答

// TYPE_CFG_WRITE -> TYPE_ACK
pub const TYPE_ACK: u8 = 0xC0;
// TYPE_CFG_QUERY -> TYPE_CFG_VALUE
pub const TYPE_CFG_VALUE: u8 = 0xC1;
