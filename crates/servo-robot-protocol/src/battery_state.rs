//! 电池状态类型

use crate::error::FrameError;
use crate::frame::{FromPayload, ToPayload};
use alloc::vec::Vec;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
#[repr(u8)]
pub enum BatteryChargeStatus {
    #[default]
    Unknown = 0,
    Charging = 1,
    Discharging = 2,
    NotCharging = 3,
    Full = 4,
}
impl BatteryChargeStatus {
    pub fn from_u8(v: u8) -> Self {
        match v {
            1 => Self::Charging,
            2 => Self::Discharging,
            3 => Self::NotCharging,
            4 => Self::Full,
            _ => Self::Unknown,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
#[repr(u8)]
pub enum BatteryHealth {
    #[default]
    Unknown = 0,
    Good = 1,
    Overheat = 2,
    Dead = 3,
    Overvoltage = 4,
}
impl BatteryHealth {
    pub fn from_u8(v: u8) -> Self {
        match v {
            1 => Self::Good,
            2 => Self::Overheat,
            3 => Self::Dead,
            4 => Self::Overvoltage,
            _ => Self::Unknown,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
#[repr(u8)]
pub enum BatteryTechnology {
    #[default]
    Unknown = 0,
    NiMh = 1,
    LiOn = 2,
    LiPo = 3,
    LiFe = 4,
    NiCd = 5,
    LiMn = 6,
}
impl BatteryTechnology {
    pub fn from_u8(v: u8) -> Self {
        match v {
            1 => Self::NiMh,
            2 => Self::LiOn,
            3 => Self::LiPo,
            4 => Self::LiFe,
            5 => Self::NiCd,
            6 => Self::LiMn,
            _ => Self::Unknown,
        }
    }
}

/// 电池状态
#[derive(Debug, Clone, Default)]
pub struct BatteryState {
    pub voltage: f32,
    pub current: f32,
    pub soc: f32,
    pub capacity: f32,
    pub design_capacity: f32,
    pub percentage: f32,
    pub temperature: f32,
    pub charge_status: BatteryChargeStatus,
    pub health: BatteryHealth,
    pub technology: BatteryTechnology,
    pub present: bool,
    pub serial_number: u32,
    pub cell_voltages: Vec<f32>,
    pub cell_temperatures: Vec<f32>,
}

impl BatteryState {
    pub fn from_bytes(data: &[u8]) -> Result<Self, FrameError> {
        if data.len() < 36 {
            return Err(FrameError::PayloadTooShort {
                expected: 36,
                got: data.len(),
            });
        }
        let mut o = 0;
        let voltage = f32::from_le_bytes([data[o], data[o + 1], data[o + 2], data[o + 3]]);
        o += 4;
        let current = f32::from_le_bytes([data[o], data[o + 1], data[o + 2], data[o + 3]]);
        o += 4;
        let soc = f32::from_le_bytes([data[o], data[o + 1], data[o + 2], data[o + 3]]);
        o += 4;
        let capacity = f32::from_le_bytes([data[o], data[o + 1], data[o + 2], data[o + 3]]);
        o += 4;
        let design_capacity = f32::from_le_bytes([data[o], data[o + 1], data[o + 2], data[o + 3]]);
        o += 4;
        let percentage = f32::from_le_bytes([data[o], data[o + 1], data[o + 2], data[o + 3]]);
        o += 4;
        let temperature = f32::from_le_bytes([data[o], data[o + 1], data[o + 2], data[o + 3]]);
        o += 4;
        let charge_status = BatteryChargeStatus::from_u8(data[o]);
        o += 1;
        let health = BatteryHealth::from_u8(data[o]);
        o += 1;
        let technology = BatteryTechnology::from_u8(data[o]);
        o += 1;
        let present = data[o] != 0;
        o += 1;
        let serial_number = u32::from_le_bytes([data[o], data[o + 1], data[o + 2], data[o + 3]]);
        o += 4;

        let remaining = data.len() - o;
        let cell_count = remaining / 8;
        let mut cell_voltages = Vec::with_capacity(cell_count);
        let mut cell_temperatures = Vec::with_capacity(cell_count);
        for _ in 0..cell_count {
            cell_voltages.push(f32::from_le_bytes([
                data[o],
                data[o + 1],
                data[o + 2],
                data[o + 3],
            ]));
            o += 4;
            cell_temperatures.push(f32::from_le_bytes([
                data[o],
                data[o + 1],
                data[o + 2],
                data[o + 3],
            ]));
            o += 4;
        }

        Ok(BatteryState {
            voltage,
            current,
            soc,
            capacity,
            design_capacity,
            percentage,
            temperature,
            charge_status,
            health,
            technology,
            present,
            serial_number,
            cell_voltages,
            cell_temperatures,
        })
    }

    pub fn to_bytes(&self) -> Vec<u8> {
        let cell_count = self.cell_voltages.len();
        let mut buf = Vec::with_capacity(36 + cell_count * 8);
        buf.extend_from_slice(&self.voltage.to_le_bytes());
        buf.extend_from_slice(&self.current.to_le_bytes());
        buf.extend_from_slice(&self.soc.to_le_bytes());
        buf.extend_from_slice(&self.capacity.to_le_bytes());
        buf.extend_from_slice(&self.design_capacity.to_le_bytes());
        buf.extend_from_slice(&self.percentage.to_le_bytes());
        buf.extend_from_slice(&self.temperature.to_le_bytes());
        buf.push(self.charge_status as u8);
        buf.push(self.health as u8);
        buf.push(self.technology as u8);
        buf.push(self.present as u8);
        buf.extend_from_slice(&self.serial_number.to_le_bytes());
        for v in &self.cell_voltages {
            buf.extend_from_slice(&v.to_le_bytes());
        }
        for t in &self.cell_temperatures {
            buf.extend_from_slice(&t.to_le_bytes());
        }
        buf
    }
}

impl ToPayload for BatteryState {
    fn to_payload(&self) -> Vec<u8> {
        self.to_bytes()
    }
}
impl FromPayload for BatteryState {
    fn from_payload(p: &[u8]) -> Result<Self, FrameError> {
        Self::from_bytes(p)
    }
}
