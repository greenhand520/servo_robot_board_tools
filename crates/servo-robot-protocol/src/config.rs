//! 配置类型

use crate::error::FrameError;
use crate::frame::{FromPayload, ToPayload};
use alloc::vec::Vec;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum ConfigType {
    Reset = 0x01,
    Shutdown = 0x02,
    SwitchServoPower = 0x10,
    Switch5VPower = 0x11,
    SwitchCharge = 0x12,
    SwitchBatExtOut = 0x13,
    SetRGB = 0x14,
    PowerServoCurrentLimit = 0x21,
    PowerServoTempLimit = 0x22,
    Power5vTempLimit = 0x23,
    ChargeMaxCurrent = 0x24,
    ChargeTempDerating = 0x25,
    ChargeTempLimit = 0x26,
    ChargeStopVoltage = 0x27,
    ChargeStopCapacity = 0x28,
}

impl ConfigType {
    pub fn from_u8(v: u8) -> Option<Self> {
        match v {
            0x01 => Some(Self::Reset),
            0x02 => Some(Self::Shutdown),
            0x10 => Some(Self::SwitchServoPower),
            0x11 => Some(Self::Switch5VPower),
            0x12 => Some(Self::SwitchCharge),
            0x13 => Some(Self::SwitchBatExtOut),
            0x14 => Some(Self::SetRGB),
            0x21 => Some(Self::PowerServoCurrentLimit),
            0x22 => Some(Self::PowerServoTempLimit),
            0x23 => Some(Self::Power5vTempLimit),
            0x24 => Some(Self::ChargeMaxCurrent),
            0x25 => Some(Self::ChargeTempDerating),
            0x26 => Some(Self::ChargeTempLimit),
            0x27 => Some(Self::ChargeStopVoltage),
            0x28 => Some(Self::ChargeStopCapacity),
            _ => None,
        }
    }


    pub fn name(&self) -> &'static str {
        match self {
            Self::Reset => "Reset",
            Self::Shutdown => "Shutdown",
            Self::SwitchServoPower => "Servo Power",
            Self::Switch5VPower => "5V Power",
            Self::SwitchCharge => "Charge",
            Self::SwitchBatExtOut => "Battery Extra Output",
            Self::SetRGB => "RGB LED",
            Self::PowerServoCurrentLimit => "Servo Current Limit",
            Self::PowerServoTempLimit => "Servo Temp Limit",
            Self::Power5vTempLimit => "5V Temp Limit",
            Self::ChargeMaxCurrent => "Charge Max Current",
            Self::ChargeTempDerating => "Charge Temp Derating",
            Self::ChargeTempLimit => "Charge Temp Limit",
            Self::ChargeStopVoltage => "Charge Stop Voltage",
            Self::ChargeStopCapacity => "Charge Stop Capacity",
        }
    }

    pub fn unit(&self) -> &'static str {
        match self {
            Self::PowerServoCurrentLimit | Self::ChargeMaxCurrent => "A",
            Self::PowerServoTempLimit
            | Self::Power5vTempLimit
            | Self::ChargeTempDerating
            | Self::ChargeTempLimit => "°C",
            Self::ChargeStopVoltage => "V",
            Self::ChargeStopCapacity => "%",
            _ => "",
        }
    }
}

/// 配置值
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Config {
    Reset,
    Shutdown,
    SwitchServoPower(bool),
    Switch5VPower(bool),
    SwitchCharge(bool),
    SwitchBatExtOut(bool),
    PowerServoCurrentLimit(f32),
    PowerServoTempLimit(f32),
    Power5vTempLimit(f32),
    ChargeMaxCurrent(f32),
    ChargeTempDerating(f32),
    ChargeTempLimit(f32),
    ChargeStopVoltage(f32),
    ChargeStopCapacity(f32),
}

impl Config {
    pub fn config_type(&self) -> ConfigType {
        match self {
            Self::Reset => ConfigType::Reset,
            Self::Shutdown => ConfigType::Shutdown,
            Self::SwitchServoPower(_) => ConfigType::SwitchServoPower,
            Self::Switch5VPower(_) => ConfigType::Switch5VPower,
            Self::SwitchCharge(_) => ConfigType::SwitchCharge,
            Self::SwitchBatExtOut(_) => ConfigType::SwitchBatExtOut,
            Self::PowerServoCurrentLimit(_) => ConfigType::PowerServoCurrentLimit,
            Self::PowerServoTempLimit(_) => ConfigType::PowerServoTempLimit,
            Self::Power5vTempLimit(_) => ConfigType::Power5vTempLimit,
            Self::ChargeMaxCurrent(_) => ConfigType::ChargeMaxCurrent,
            Self::ChargeTempDerating(_) => ConfigType::ChargeTempDerating,
            Self::ChargeTempLimit(_) => ConfigType::ChargeTempLimit,
            Self::ChargeStopVoltage(_) => ConfigType::ChargeStopVoltage,
            Self::ChargeStopCapacity(_) => ConfigType::ChargeStopCapacity,
        }
    }

    pub fn value(&self) -> f32 {
        match self {
            Self::Reset | Self::Shutdown => 0.0,
            Self::SwitchServoPower(on)
            | Self::Switch5VPower(on)
            | Self::SwitchCharge(on)
            | Self::SwitchBatExtOut(on) => {
                if *on {
                    1.0
                } else {
                    0.0
                }
            }
            Self::PowerServoCurrentLimit(v)
            | Self::PowerServoTempLimit(v)
            | Self::Power5vTempLimit(v)
            | Self::ChargeMaxCurrent(v)
            | Self::ChargeTempDerating(v)
            | Self::ChargeTempLimit(v)
            | Self::ChargeStopVoltage(v)
            | Self::ChargeStopCapacity(v) => *v,
        }
    }

    pub fn from_type_value(typ: ConfigType, value: f32) -> Self {
        match typ {
            ConfigType::Reset => Self::Reset,
            ConfigType::Shutdown => Self::Shutdown,
            ConfigType::SwitchServoPower => Self::SwitchServoPower(value != 0.0),
            ConfigType::Switch5VPower => Self::Switch5VPower(value != 0.0),
            ConfigType::SwitchCharge => Self::SwitchCharge(value != 0.0),
            ConfigType::SwitchBatExtOut => Self::SwitchBatExtOut(value != 0.0),
            ConfigType::PowerServoCurrentLimit => Self::PowerServoCurrentLimit(value),
            ConfigType::PowerServoTempLimit => Self::PowerServoTempLimit(value),
            ConfigType::Power5vTempLimit => Self::Power5vTempLimit(value),
            ConfigType::ChargeMaxCurrent => Self::ChargeMaxCurrent(value),
            ConfigType::ChargeTempDerating => Self::ChargeTempDerating(value),
            ConfigType::ChargeTempLimit => Self::ChargeTempLimit(value),
            ConfigType::ChargeStopVoltage => Self::ChargeStopVoltage(value),
            ConfigType::ChargeStopCapacity => Self::ChargeStopCapacity(value),
            _ => Self::Reset,
        }
    }

    pub fn from_bytes(data: &[u8]) -> Result<Self, FrameError> {
        if data.is_empty() {
            return Err(FrameError::PayloadTooShort {
                expected: 1,
                got: 0,
            });
        }
        let config_type =
            ConfigType::from_u8(data[0]).ok_or(FrameError::PayloadDecode("Unknown config type"))?;
        if (config_type as u8) < 0x20 {
            return Ok(Config::from_type_value(config_type, 0.0));
        }
        if data.len() < 5 {
            return Err(FrameError::PayloadTooShort {
                expected: 5,
                got: data.len(),
            });
        }
        let value = f32::from_le_bytes([data[1], data[2], data[3], data[4]]);
        Ok(Config::from_type_value(config_type, value))
    }

    pub fn to_bytes(&self) -> Vec<u8> {
        let mut buf = Vec::with_capacity(5);
        buf.push(self.config_type() as u8);
        match self {
            Self::Reset | Self::Shutdown => {}
            Self::SwitchServoPower(on)
            | Self::Switch5VPower(on)
            | Self::SwitchCharge(on)
            | Self::SwitchBatExtOut(on) => buf.push(*on as u8),
            _ => buf.extend_from_slice(&self.value().to_le_bytes()),
        }
        buf
    }
}

impl ToPayload for Config {
    fn to_payload(&self) -> Vec<u8> {
        self.to_bytes()
    }
}
impl FromPayload for Config {
    fn from_payload(p: &[u8]) -> Result<Self, FrameError> {
        Self::from_bytes(p)
    }
}

/// 板级配置快照
#[derive(Debug, Clone)]
pub struct BoardConfigSnapshot {
    pub servo_current_limit: f32,
    pub servo_temp_limit: f32,
    pub temp_5v_limit: f32,
    pub charge_max_current: f32,
    pub charge_temp_derating: f32,
    pub charge_temp_limit: f32,
    pub charge_stop_voltage: f32,
    pub charge_stop_percentage: f32,
    pub pd_negotiated_mv: u16,
    pub pd_negotiated_ma: u16,
    pub charge_enable: bool,
    pub servo_power_on: bool,
    pub power_5v_on: bool,
    pub charge_on: bool,
    pub bat_ext_out_on: bool,
}

impl Default for BoardConfigSnapshot {
    fn default() -> Self {
        BoardConfigSnapshot {
            servo_current_limit: 5.0,
            servo_temp_limit: 80.0,
            temp_5v_limit: 70.0,
            charge_max_current: 9.0,
            charge_temp_derating: 60.0,
            charge_temp_limit: 70.0,
            charge_stop_voltage: 16.8,
            charge_stop_percentage: 1.0,
            pd_negotiated_mv: 20000,
            pd_negotiated_ma: 5000,
            charge_enable: true,
            servo_power_on: true,
            power_5v_on: true,
            charge_on: false,
            bat_ext_out_on: false,
        }
    }
}

impl BoardConfigSnapshot {
    pub fn from_bytes(data: &[u8]) -> Result<Self, FrameError> {
        if data.len() < 41 {
            return Err(FrameError::PayloadTooShort {
                expected: 41,
                got: data.len(),
            });
        }
        let mut o = 0;
        let servo_current_limit =
            f32::from_le_bytes([data[o], data[o + 1], data[o + 2], data[o + 3]]);
        o += 4;
        let servo_temp_limit = f32::from_le_bytes([data[o], data[o + 1], data[o + 2], data[o + 3]]);
        o += 4;
        let temp_5v_limit = f32::from_le_bytes([data[o], data[o + 1], data[o + 2], data[o + 3]]);
        o += 4;
        let charge_max_current =
            f32::from_le_bytes([data[o], data[o + 1], data[o + 2], data[o + 3]]);
        o += 4;
        let charge_temp_derating =
            f32::from_le_bytes([data[o], data[o + 1], data[o + 2], data[o + 3]]);
        o += 4;
        let charge_temp_limit =
            f32::from_le_bytes([data[o], data[o + 1], data[o + 2], data[o + 3]]);
        o += 4;
        let charge_stop_voltage =
            f32::from_le_bytes([data[o], data[o + 1], data[o + 2], data[o + 3]]);
        o += 4;
        let charge_stop_percentage =
            f32::from_le_bytes([data[o], data[o + 1], data[o + 2], data[o + 3]]);
        o += 4;
        let pd_negotiated_mv = u16::from_le_bytes([data[o], data[o + 1]]);
        o += 2;
        let pd_negotiated_ma = u16::from_le_bytes([data[o], data[o + 1]]);
        o += 2;
        let charge_enable = data[o] != 0;
        o += 1;
        let servo_power_on = data[o] != 0;
        o += 1;
        let power_5v_on = data[o] != 0;
        o += 1;
        let charge_on = data[o] != 0;
        o += 1;
        let bat_ext_out_on = data[o] != 0;
        Ok(BoardConfigSnapshot {
            servo_current_limit,
            servo_temp_limit,
            temp_5v_limit,
            charge_max_current,
            charge_temp_derating,
            charge_temp_limit,
            charge_stop_voltage,
            charge_stop_percentage,
            pd_negotiated_mv,
            pd_negotiated_ma,
            charge_enable,
            servo_power_on,
            power_5v_on,
            charge_on,
            bat_ext_out_on,
        })
    }

    pub fn to_bytes(&self) -> Vec<u8> {
        let mut buf = Vec::with_capacity(41);
        buf.extend_from_slice(&self.servo_current_limit.to_le_bytes());
        buf.extend_from_slice(&self.servo_temp_limit.to_le_bytes());
        buf.extend_from_slice(&self.temp_5v_limit.to_le_bytes());
        buf.extend_from_slice(&self.charge_max_current.to_le_bytes());
        buf.extend_from_slice(&self.charge_temp_derating.to_le_bytes());
        buf.extend_from_slice(&self.charge_temp_limit.to_le_bytes());
        buf.extend_from_slice(&self.charge_stop_voltage.to_le_bytes());
        buf.extend_from_slice(&self.charge_stop_percentage.to_le_bytes());
        buf.extend_from_slice(&self.pd_negotiated_mv.to_le_bytes());
        buf.extend_from_slice(&self.pd_negotiated_ma.to_le_bytes());
        buf.push(self.charge_enable as u8);
        buf.push(self.servo_power_on as u8);
        buf.push(self.power_5v_on as u8);
        buf.push(self.charge_on as u8);
        buf.push(self.bat_ext_out_on as u8);
        buf
    }
}

impl ToPayload for BoardConfigSnapshot {
    fn to_payload(&self) -> Vec<u8> {
        self.to_bytes()
    }
}
impl FromPayload for BoardConfigSnapshot {
    fn from_payload(p: &[u8]) -> Result<Self, FrameError> {
        Self::from_bytes(p)
    }
}
