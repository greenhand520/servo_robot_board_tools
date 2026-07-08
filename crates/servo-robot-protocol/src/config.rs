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
    ChargeStopSoc = 0x20,
    // servo robot board发送的日志等级
    TxLogLevel = 0x21,
    PowerServoCurrentLimitMa = 0x30,
    PowerServoTempLimit = 0x31,
    Power5vTempLimit = 0x32,
    ChargeMaxCurrentMa = 0x33,
    ChargeTempDerating = 0x34,
    ChargeTempLimit = 0x35,
    ChargeStopVoltageMv = 0x36,
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
            0x20 => Some(Self::ChargeStopSoc),
            0x21 => Some(Self::TxLogLevel),
            0x30 => Some(Self::PowerServoCurrentLimitMa),
            0x31 => Some(Self::PowerServoTempLimit),
            0x32 => Some(Self::Power5vTempLimit),
            0x33 => Some(Self::ChargeMaxCurrentMa),
            0x34 => Some(Self::ChargeTempDerating),
            0x35 => Some(Self::ChargeTempLimit),
            0x36 => Some(Self::ChargeStopVoltageMv),
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
            Self::ChargeStopSoc => "Charge Stop Soc",
            Self::TxLogLevel => "TxLog Level",
            Self::PowerServoCurrentLimitMa => "Servo Current Limit(ma)",
            Self::PowerServoTempLimit => "Servo Temp Limit",
            Self::Power5vTempLimit => "5V Temp Limit",
            Self::ChargeMaxCurrentMa => "Charge Max Current(ma)",
            Self::ChargeTempDerating => "Charge Temp Derating",
            Self::ChargeTempLimit => "Charge Temp Limit",
            Self::ChargeStopVoltageMv => "Charge Stop Voltage(mv)",
        }
    }

    pub fn unit(&self) -> &'static str {
        match self {
            Self::PowerServoCurrentLimitMa | Self::ChargeMaxCurrentMa => "mA",
            Self::PowerServoTempLimit
            | Self::Power5vTempLimit
            | Self::ChargeTempDerating
            | Self::ChargeTempLimit => "°C",
            Self::ChargeStopVoltageMv => "mV",
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
    // Switch the servo power supply
    SwitchPowerServo(bool),
    // Switch 5v power supply
    SwitchPower5V(bool),
    // Switch on and off to charge the battery
    SwitchCharge(bool),
    // Switching on battery extra output
    SwitchBatExtOut(bool),
    // Charging capacity limit, such as charging only up to 80%
    ChargeStopSoc(u8),
    // The log level of to send
    TxLogLevel(LogLevel),
    // Servo power supply current limiting
    PowerServoCurrentLimitMa(u16),
    // Servo power supply temperature restriction
    PowerServoTempLimit(u16),
    // 5V power temperature limit
    Power5vTempLimit(u16),
    // Maximum charging current
    ChargeMaxCurrentMa(u16),
    // The temperature of the charging circuit when charging starts to drop current
    ChargeTempDerating(u16),
    // The temperature of the charging circuit when charging is stopped
    ChargeTempLimit(u16),
    // Charging stop-voltage range
    ChargeStopVoltageMv(u16),
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
            Self::ChargeStopSoc(_) => ConfigType::ChargeStopSoc,
            Self::TxLogLevel(_) => ConfigType::TxLogLevel,
            Self::PowerServoCurrentLimitMa(_) => ConfigType::PowerServoCurrentLimitMa,
            Self::PowerServoTempLimit(_) => ConfigType::PowerServoTempLimit,
            Self::Power5vTempLimit(_) => ConfigType::Power5vTempLimit,
            Self::ChargeMaxCurrentMa(_) => ConfigType::ChargeMaxCurrentMa,
            Self::ChargeTempDerating(_) => ConfigType::ChargeTempDerating,
            Self::ChargeTempLimit(_) => ConfigType::ChargeTempLimit,
            Self::ChargeStopVoltageMv(_) => ConfigType::ChargeStopVoltageMv,
        }
    }

    pub fn value(&self) -> u16 {
        match self {
            Self::Reset | Self::Shutdown => 0,
            Self::SwitchPowerServo(on)
            | Self::SwitchPower5V(on)
            | Self::SwitchCharge(on)
            | Self::SwitchBatExtOut(on) => {
                if *on {
                    1
                } else {
                    0
                }
            }
            Self::ChargeStopSoc(v) => *v as u16,
            Self::TxLogLevel(level) => *level as u8 as u16,
            Self::PowerServoCurrentLimitMa(v)
            | Self::ChargeStopVoltageMv(v)
            | Self::ChargeMaxCurrentMa(v)
            | Self::PowerServoTempLimit(v)
            | Self::Power5vTempLimit(v)
            | Self::ChargeTempDerating(v)
            | Self::ChargeTempLimit(v) => *v,
        }
    }

    pub fn from_type_value(typ: ConfigType, value: u16) -> Self {
        match typ {
            ConfigType::Reset => Self::Reset,
            ConfigType::Shutdown => Self::Shutdown,
            ConfigType::SwitchServoPower => Self::SwitchPowerServo(value != 0),
            ConfigType::Switch5VPower => Self::SwitchPower5V(value != 0),
            ConfigType::SwitchCharge => Self::SwitchCharge(value != 0),
            ConfigType::SwitchBatExtOut => Self::SwitchBatExtOut(value != 0),
            ConfigType::ChargeStopSoc => Self::ChargeStopSoc(value as _),
            ConfigType::TxLogLevel => Self::TxLogLevel(LogLevel::from_u8(value as _)),
            ConfigType::PowerServoCurrentLimitMa => Self::PowerServoCurrentLimitMa(value),
            ConfigType::PowerServoTempLimit => Self::PowerServoTempLimit(value),
            ConfigType::Power5vTempLimit => Self::Power5vTempLimit(value),
            ConfigType::ChargeMaxCurrentMa => Self::ChargeMaxCurrentMa(value),
            ConfigType::ChargeTempDerating => Self::ChargeTempDerating(value),
            ConfigType::ChargeTempLimit => Self::ChargeTempLimit(value),
            ConfigType::ChargeStopVoltageMv => Self::ChargeStopVoltageMv(value),
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

        // Reset and Shutdown have no value payload
        match config_type {
            ConfigType::Reset => return Ok(Config::Reset),
            ConfigType::Shutdown => return Ok(Config::Shutdown),
            _ => {}
        }

        if (config_type as u8) < 0x30 {
            // For type value less than 0x30, only need one byte to get the configuration value
            if data.len() < 2 {
                return Err(FrameError::PayloadTooShort {
                    expected: 2,
                    got: data.len(),
                });
            }
            let value = u16::from_le_bytes([data[1], 0]);
            return Ok(Config::from_type_value(config_type, value));
        }
        if data.len() < 3 {
            return Err(FrameError::PayloadTooShort {
                expected: 3,
                got: data.len(),
            });
        }
        let value = u16::from_le_bytes([data[1], data[2]]);
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
    pub servo_current_limit_ma: u16,
    pub servo_temp_limit: u16,
    pub temp_5v_limit: u16,
    pub charge_max_current_ma: u16,
    pub charge_temp_derating: u16,
    pub charge_temp_limit: u16,
    pub charge_stop_voltage_mv: u16,
    // 1~100
    pub charge_stop_percentage: u8,
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
            servo_current_limit_ma: 50,
            servo_temp_limit: 800,
            temp_5v_limit: 700,
            charge_max_current_ma: 90,
            charge_temp_derating: 600,
            charge_temp_limit: 700,
            charge_stop_voltage_mv: 168,
            charge_stop_percentage: 100,
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
        if data.len() < 21 {
            return Err(FrameError::PayloadTooShort {
                expected: 21,
                got: data.len(),
            });
        }
        let mut o = 0;
        let servo_current_limit_ma = u16::from_le_bytes([data[o], data[o + 1]]);
        o += 2;
        let servo_temp_limit = u16::from_le_bytes([data[o], data[o + 1]]);
        o += 2;
        let temp_5v_limit = u16::from_le_bytes([data[o], data[o + 1]]);
        o += 2;
        let charge_max_current_ma = u16::from_le_bytes([data[o], data[o + 1]]);
        o += 2;
        let charge_temp_derating = u16::from_le_bytes([data[o], data[o + 1]]);
        o += 2;
        let charge_temp_limit = u16::from_le_bytes([data[o], data[o + 1]]);
        o += 2;
        let charge_stop_voltage_mv = u16::from_le_bytes([data[o], data[o + 1]]);
        o += 2;
        let charge_stop_percentage = data[o];
        o += 1;
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
            servo_current_limit_ma,
            servo_temp_limit,
            temp_5v_limit,
            charge_max_current_ma,
            charge_temp_derating,
            charge_temp_limit,
            charge_stop_voltage_mv,
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
        let mut buf = Vec::with_capacity(22);
        buf.extend_from_slice(&self.servo_current_limit_ma.to_le_bytes());
        buf.extend_from_slice(&self.servo_temp_limit.to_le_bytes());
        buf.extend_from_slice(&self.temp_5v_limit.to_le_bytes());
        buf.extend_from_slice(&self.charge_max_current_ma.to_le_bytes());
        buf.extend_from_slice(&self.charge_temp_derating.to_le_bytes());
        buf.extend_from_slice(&self.charge_temp_limit.to_le_bytes());
        buf.extend_from_slice(&self.charge_stop_voltage_mv.to_le_bytes());
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
        // Convert u16 (*10) to f32 for display
        write!(
            f,
            "servo={:.1}mA/{:.1}°C 5v={:.1}°C chg={:.1}A/{:.1}-{:.1}°C/{:.1}mV/{}% sw=[{},{},{},{},{}] lvl={}",
            self.servo_current_limit_ma,
            self.servo_temp_limit as f32 / 10.0,
            self.temp_5v_limit as f32 / 10.0,
            self.charge_max_current_ma,
            self.charge_temp_derating as f32 / 10.0,
            self.charge_temp_limit as f32 / 10.0,
            self.charge_stop_voltage_mv,
            self.charge_stop_percentage,
            if self.power_servo_on { "S" } else { "-" },
            if self.power_5v_on { "5" } else { "-" },
            if self.charge_on { "C" } else { "-" },
            if self.bat_ext_out_on { "B" } else { "-" },
            if self.charge_enable { "E" } else { "-" },
            self.tx_log_level as u8
        )
    }
}
