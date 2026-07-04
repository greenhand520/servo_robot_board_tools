//! 事件类型

use crate::error::FrameError;
use crate::frame::{FromPayload, ToPayload};
use alloc::vec::Vec;

bitflags::bitflags! {
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub struct ProtectionFlags: u16 {
        const SERVO_OVERCURRENT = 1 << 0;
        const SERVO_THERMAL     = 1 << 1;
        const DCDC_5V_THERMAL   = 1 << 2;
        const CHARGE_DERATING   = 1 << 3;
        const CHARGE_THERMAL    = 1 << 4;
        const BATTERY_LOW       = 1 << 5;
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum ChargePhase {
    Unknown = 0,
    NotCharging = 1,
    PreCharge = 2,
    Cc = 3,
    Cv = 4,
    Full = 5,
    Husb238Fault = 6,
    Unsupported = 7,
}

impl ChargePhase {
    pub fn from_u8(v: u8) -> Self {
        match v {
            0 => Self::Unknown, 1 => Self::NotCharging, 2 => Self::PreCharge,
            3 => Self::Cc, 4 => Self::Cv, 5 => Self::Full,
            6 => Self::Husb238Fault, 7 => Self::Unsupported, _ => Self::Unknown,
        }
    }
}

/// 事件
#[derive(Debug, Clone)]
pub struct BoardEvent {
    pub charger_connected: bool,
    pub fan_enabled: bool,
    pub charge_phase: ChargePhase,
    pub protection_flags: ProtectionFlags,
}

impl Default for BoardEvent {
    fn default() -> Self {
        BoardEvent { charger_connected: false, fan_enabled: false, charge_phase: ChargePhase::Unknown, protection_flags: ProtectionFlags::empty() }
    }
}

impl BoardEvent {
    pub fn from_bytes(data: &[u8]) -> Result<Self, FrameError> {
        if data.len() < 5 {
            return Err(FrameError::PayloadTooShort { expected: 5, got: data.len() });
        }
        let charger_connected = data[0] != 0;
        let fan_enabled = data[1] != 0;
        let charge_phase = ChargePhase::from_u8(data[2]);
        let protection_flags = ProtectionFlags::from_bits(u16::from_le_bytes([data[3], data[4]])).unwrap_or(ProtectionFlags::empty());
        Ok(BoardEvent { charger_connected, fan_enabled, charge_phase, protection_flags })
    }

    pub fn to_bytes(&self) -> Vec<u8> {
        let mut buf = Vec::with_capacity(5);
        buf.push(self.charger_connected as u8);
        buf.push(self.fan_enabled as u8);
        buf.push(self.charge_phase as u8);
        buf.extend_from_slice(&self.protection_flags.bits().to_le_bytes());
        buf
    }
}

impl ToPayload for BoardEvent { fn to_payload(&self) -> Vec<u8> { self.to_bytes() } }
impl FromPayload for BoardEvent { fn from_payload(p: &[u8]) -> Result<Self, FrameError> { Self::from_bytes(p) } }
