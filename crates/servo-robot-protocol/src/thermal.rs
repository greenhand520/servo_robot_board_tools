//! 温度数据类型

use crate::error::FrameError;
use crate::frame::{FromPayload, ToPayload};
use alloc::vec::Vec;

/// 温度数据
#[derive(Debug, Clone, Default)]
pub struct ThermalData {
    pub temp_servo_power: f32,
    pub temp_5v_power: f32,
    pub temp_mcu: f32,
    pub temp_charge: f32,
    pub temp_battery: f32,
    pub reserved: f32,
}

impl ThermalData {
    pub fn from_bytes(data: &[u8]) -> Result<Self, FrameError> {
        if data.len() < 24 {
            return Err(FrameError::PayloadTooShort { expected: 24, got: data.len() });
        }
        let mut offset = 0;
        let temp_servo_power = f32::from_le_bytes([data[offset], data[offset+1], data[offset+2], data[offset+3]]); offset += 4;
        let temp_5v_power = f32::from_le_bytes([data[offset], data[offset+1], data[offset+2], data[offset+3]]); offset += 4;
        let temp_mcu = f32::from_le_bytes([data[offset], data[offset+1], data[offset+2], data[offset+3]]); offset += 4;
        let temp_charge = f32::from_le_bytes([data[offset], data[offset+1], data[offset+2], data[offset+3]]); offset += 4;
        let temp_battery = f32::from_le_bytes([data[offset], data[offset+1], data[offset+2], data[offset+3]]); offset += 4;
        let reserved = f32::from_le_bytes([data[offset], data[offset+1], data[offset+2], data[offset+3]]);
        Ok(ThermalData { temp_servo_power, temp_5v_power, temp_mcu, temp_charge, temp_battery, reserved })
    }

    pub fn to_bytes(&self) -> Vec<u8> {
        let mut buf = Vec::with_capacity(24);
        buf.extend_from_slice(&self.temp_servo_power.to_le_bytes());
        buf.extend_from_slice(&self.temp_5v_power.to_le_bytes());
        buf.extend_from_slice(&self.temp_mcu.to_le_bytes());
        buf.extend_from_slice(&self.temp_charge.to_le_bytes());
        buf.extend_from_slice(&self.temp_battery.to_le_bytes());
        buf.extend_from_slice(&self.reserved.to_le_bytes());
        buf
    }
}

impl ToPayload for ThermalData { fn to_payload(&self) -> Vec<u8> { self.to_bytes() } }
impl FromPayload for ThermalData { fn from_payload(p: &[u8]) -> Result<Self, FrameError> { Self::from_bytes(p) } }
