//! 电源数据类型

use crate::error::FrameError;
use crate::frame::{FromPayload, ToPayload};
use alloc::vec::Vec;

/// 电源数据
#[derive(Debug, Clone, Default)]
pub struct PowerData {
    pub servo_voltage: f32,
    pub servo_current: f32,
    pub charge_in_voltage: f32,
    pub charge_in_current: f32,
    pub bat_voltage: f32,
    pub bat_current: f32,
}

impl PowerData {
    pub fn from_bytes(data: &[u8]) -> Result<Self, FrameError> {
        if data.len() < 24 {
            return Err(FrameError::PayloadTooShort { expected: 24, got: data.len() });
        }
        let mut offset = 0;
        let servo_voltage = f32::from_le_bytes([data[offset], data[offset+1], data[offset+2], data[offset+3]]); offset += 4;
        let servo_current = f32::from_le_bytes([data[offset], data[offset+1], data[offset+2], data[offset+3]]); offset += 4;
        let charge_in_voltage = f32::from_le_bytes([data[offset], data[offset+1], data[offset+2], data[offset+3]]); offset += 4;
        let charge_in_current = f32::from_le_bytes([data[offset], data[offset+1], data[offset+2], data[offset+3]]); offset += 4;
        let bat_voltage = f32::from_le_bytes([data[offset], data[offset+1], data[offset+2], data[offset+3]]); offset += 4;
        let bat_current = f32::from_le_bytes([data[offset], data[offset+1], data[offset+2], data[offset+3]]);
        Ok(PowerData { servo_voltage, servo_current, charge_in_voltage, charge_in_current, bat_voltage, bat_current })
    }

    pub fn to_bytes(&self) -> Vec<u8> {
        let mut buf = Vec::with_capacity(24);
        buf.extend_from_slice(&self.servo_voltage.to_le_bytes());
        buf.extend_from_slice(&self.servo_current.to_le_bytes());
        buf.extend_from_slice(&self.charge_in_voltage.to_le_bytes());
        buf.extend_from_slice(&self.charge_in_current.to_le_bytes());
        buf.extend_from_slice(&self.bat_voltage.to_le_bytes());
        buf.extend_from_slice(&self.bat_current.to_le_bytes());
        buf
    }
}

impl ToPayload for PowerData { fn to_payload(&self) -> Vec<u8> { self.to_bytes() } }
impl FromPayload for PowerData { fn from_payload(p: &[u8]) -> Result<Self, FrameError> { Self::from_bytes(p) } }

impl core::fmt::Display for PowerData {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "servo={:.1}V/{:.1}A pd_in={:.1}V/{:.1}A bat={:.1}V/{:.1}A",
            self.servo_voltage, self.servo_current,
            self.charge_in_voltage, self.charge_in_current,
            self.bat_voltage, self.bat_current)
    }
}
