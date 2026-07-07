//! # Authors
//! greenhand520
//! # Since
//! version: 0.1.0
//! # Date
//! 2026/7/3 11:30
//! Board Config

use crate::error::FrameError;
use crate::frame::{FromPayload, ToPayload};
use crate::log::LogLevel;
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
    PowerServoCurrentLimit = 0x21,
    PowerServoTempLimit = 0x22,
    Power5vTempLimit = 0x23,
    ChargeMaxCurrent = 0x24,
    ChargeTempDerating = 0x25,
    ChargeTempLimit = 0x26,
    ChargeStopVoltage = 0x27,
    ChargeStopSoc = 0x28,
    // servo robot board发送的日志等级
    TxLogLevel = 0x29,
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
            0x21 => Some(Self::PowerServoCurrentLimit),
            0x22 => Some(Self::PowerServoTempLimit),
            0x23 => Some(Self::Power5vTempLimit),
            0x24 => Some(Self::ChargeMaxCurrent),
            0x25 => Some(Self::ChargeTempDerating),
            0x26 => Some(Self::ChargeTempLimit),
            0x27 => Some(Self::ChargeStopVoltage),
            0x28 => Some(Self::ChargeStopSoc),
            0x29 => Some(Self::TxLogLevel),
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
            Self::PowerServoCurrentLimit => "Servo Current Limit",
            Self::PowerServoTempLimit => "Servo Temp Limit",
            Self::Power5vTempLimit => "5V Temp Limit",
            Self::ChargeMaxCurrent => "Charge Max Current",
            Self::ChargeTempDerating => "Charge Temp Derating",
            Self::ChargeTempLimit => "Charge Temp Limit",
            Self::ChargeStopVoltage => "Charge Stop Voltage",
            Self::ChargeStopSoc => "Charge Stop Soc",
            Self::TxLogLevel => "TxLog Level",
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
            Self::ChargeStopSoc => "%",
            _ => "",
        }
    }
}

/// Configuration values
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Config {
    Reset,
    Shutdown,
    SwitchPowerServo(bool),
    SwitchPower5V(bool),
    SwitchCharge(bool),
    SwitchBatExtOut(bool),
    PowerServoCurrentLimit(f32),
    PowerServoTempLimit(f32),
    Power5vTempLimit(f32),
    ChargeMaxCurrent(f32),
    // 充电开始降流时的充电电路温度
    ChargeTempDerating(f32),
    // 停止充电时的充电电路温度
    ChargeTempLimit(f32),
    // 充电停止电压
    ChargeStopVoltage(f32),
    // 充电电量限制，如只充到80%
    ChargeStopSoc(f32),
    // 发送的日志等级
    TxLogLevel(LogLevel),
}

impl Config {
    pub fn config_type(&self) -> ConfigType {
        match self {
            Self::Reset => ConfigType::Reset,
            Self::Shutdown => ConfigType::Shutdown,
            Self::SwitchPowerServo(_) => ConfigType::SwitchServoPower,
            Self::SwitchPower5V(_) => ConfigType::Switch5VPower,
            Self::SwitchCharge(_) => ConfigType::SwitchCharge,
            Self::SwitchBatExtOut(_) => ConfigType::SwitchBatExtOut,
            Self::PowerServoCurrentLimit(_) => ConfigType::PowerServoCurrentLimit,
            Self::PowerServoTempLimit(_) => ConfigType::PowerServoTempLimit,
            Self::Power5vTempLimit(_) => ConfigType::Power5vTempLimit,
            Self::ChargeMaxCurrent(_) => ConfigType::ChargeMaxCurrent,
            Self::ChargeTempDerating(_) => ConfigType::ChargeTempDerating,
            Self::ChargeTempLimit(_) => ConfigType::ChargeTempLimit,
            Self::ChargeStopVoltage(_) => ConfigType::ChargeStopVoltage,
            Self::ChargeStopSoc(_) => ConfigType::ChargeStopSoc,
            Self::TxLogLevel(_) => ConfigType::TxLogLevel,
        }
    }

    pub fn value(&self) -> f32 {
        match self {
            Self::Reset | Self::Shutdown => 0.0,
            Self::SwitchPowerServo(on)
            | Self::SwitchPower5V(on)
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
            | Self::ChargeStopSoc(v) => *v,
            Self::TxLogLevel(level) => *level as u8 as f32,
        }
    }

    pub fn from_type_value(typ: ConfigType, value: f32) -> Self {
        match typ {
            ConfigType::Reset => Self::Reset,
            ConfigType::Shutdown => Self::Shutdown,
            ConfigType::SwitchServoPower => Self::SwitchPowerServo(value != 0.0),
            ConfigType::Switch5VPower => Self::SwitchPower5V(value != 0.0),
            ConfigType::SwitchCharge => Self::SwitchCharge(value != 0.0),
            ConfigType::SwitchBatExtOut => Self::SwitchBatExtOut(value != 0.0),
            ConfigType::PowerServoCurrentLimit => Self::PowerServoCurrentLimit(value),
            ConfigType::PowerServoTempLimit => Self::PowerServoTempLimit(value),
            ConfigType::Power5vTempLimit => Self::Power5vTempLimit(value),
            ConfigType::ChargeMaxCurrent => Self::ChargeMaxCurrent(value),
            ConfigType::ChargeTempDerating => Self::ChargeTempDerating(value),
            ConfigType::ChargeTempLimit => Self::ChargeTempLimit(value),
            ConfigType::ChargeStopVoltage => Self::ChargeStopVoltage(value),
            ConfigType::ChargeStopSoc => Self::ChargeStopSoc(value),
            ConfigType::TxLogLevel => Self::TxLogLevel(LogLevel::from_u8(value as _)),
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
            Self::SwitchPowerServo(on)
            | Self::SwitchPower5V(on)
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

/// Snapshot of board-level configuration
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
    pub charge_enable: bool,
    pub power_servo_on: bool,
    pub power_5v_on: bool,
    pub charge_on: bool,
    pub bat_ext_out_on: bool,
    pub tx_log_level: LogLevel,
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
            charge_enable: true,
            power_servo_on: true,
            power_5v_on: true,
            charge_on: true,
            bat_ext_out_on: true,
            tx_log_level: LogLevel::Info,
        }
    }
}

impl BoardConfigSnapshot {
    pub fn from_bytes(data: &[u8]) -> Result<Self, FrameError> {
        if data.len() < 38 {
            return Err(FrameError::PayloadTooShort {
                expected: 38,
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
        let charge_enable = data[o] != 0;
        o += 1;
        let power_servo_on = data[o] != 0;
        o += 1;
        let power_5v_on = data[o] != 0;
        o += 1;
        let charge_on = data[o] != 0;
        o += 1;
        let bat_ext_out_on = data[o] != 0;
        o += 1;
        let tx_log_level = LogLevel::from_u8(data[o]);
        Ok(BoardConfigSnapshot {
            servo_current_limit,
            servo_temp_limit,
            temp_5v_limit,
            charge_max_current,
            charge_temp_derating,
            charge_temp_limit,
            charge_stop_voltage,
            charge_stop_percentage,
            charge_enable,
            power_servo_on,
            power_5v_on,
            charge_on,
            bat_ext_out_on,
            tx_log_level,
        })
    }

    pub fn to_bytes(&self) -> Vec<u8> {
        let mut buf = Vec::with_capacity(38);
        buf.extend_from_slice(&self.servo_current_limit.to_le_bytes());
        buf.extend_from_slice(&self.servo_temp_limit.to_le_bytes());
        buf.extend_from_slice(&self.temp_5v_limit.to_le_bytes());
        buf.extend_from_slice(&self.charge_max_current.to_le_bytes());
        buf.extend_from_slice(&self.charge_temp_derating.to_le_bytes());
        buf.extend_from_slice(&self.charge_temp_limit.to_le_bytes());
        buf.extend_from_slice(&self.charge_stop_voltage.to_le_bytes());
        buf.extend_from_slice(&self.charge_stop_percentage.to_le_bytes());
        buf.push(self.charge_enable as u8);
        buf.push(self.power_servo_on as u8);
        buf.push(self.power_5v_on as u8);
        buf.push(self.charge_on as u8);
        buf.push(self.bat_ext_out_on as u8);
        buf.push(self.tx_log_level as u8);
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

impl core::fmt::Display for BoardConfigSnapshot {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(
            f,
            "servo={:.1}A/{:.1}°C 5v={:.1}°C chg={:.1}A/{:.1}-{:.1}°C/{:.1}V/{:.0}% sw=[{},{},{},{},{}] lvl={}",
            self.servo_current_limit,
            self.servo_temp_limit,
            self.temp_5v_limit,
            self.charge_max_current,
            self.charge_temp_derating,
            self.charge_temp_limit,
            self.charge_stop_voltage,
            self.charge_stop_percentage * 100.0,
            if self.power_servo_on { "S" } else { "-" },
            if self.power_5v_on { "5" } else { "-" },
            if self.charge_on { "C" } else { "-" },
            if self.bat_ext_out_on { "B" } else { "-" },
            if self.charge_enable { "E" } else { "-" },
            self.tx_log_level as u8
        )
    }
}
