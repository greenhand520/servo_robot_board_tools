//! # Authors
//! greenhand520
//! # Since
//! version: 0.1.0
//! # Date
//! 2026/7/3 13:22

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum ConfigType {
    PowerServoCurrentLimit = 0x01,
    PowerServoTempLimit = 0x02,
    Power5vTempLimit = 0x03,
    ChargeMaxCurrent = 0x04,
    ChargeTempDerating = 0x05,
    ChargeTempLimit = 0x06,
    ChargeStopVoltage = 0x07,
    ChargeStopCapacity = 0x08,
}

impl ConfigType {
    pub fn from_u8(v: u8) -> Option<Self> {
        match v {
            0x01 => Some(Self::PowerServoCurrentLimit),
            0x02 => Some(Self::PowerServoTempLimit),
            0x03 => Some(Self::Power5vTempLimit),
            0x04 => Some(Self::ChargeMaxCurrent),
            0x05 => Some(Self::ChargeTempDerating),
            0x06 => Some(Self::ChargeTempLimit),
            0x07 => Some(Self::ChargeStopVoltage),
            0x08 => Some(Self::ChargeStopCapacity),
            _ => None,
        }
    }

    pub fn name(&self) -> &'static str {
        match self {
            Self::PowerServoCurrentLimit => "Power Servo Current Limit",
            Self::PowerServoTempLimit => "Power Servo Temp Limit",
            Self::Power5vTempLimit => "Power 5V Temp Limit",
            Self::ChargeMaxCurrent => "Charge Max Current",
            Self::ChargeTempDerating => "Charge Temp Derating",
            Self::ChargeTempLimit => "Charge Temp Cutoff",
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
            Self::ChargeStopCapacity => "mAh",
        }
    }
}

impl std::fmt::Display for ConfigType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.name())
    }
}

/// 完整配置项（类型 + 具体数值）
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Config {
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
    /// 从 ConfigType + 原始值构造完整配置
    pub fn from_type_value(typ: ConfigType, value: f32) -> Self {
        match typ {
            ConfigType::PowerServoCurrentLimit => Self::PowerServoCurrentLimit(value),
            ConfigType::PowerServoTempLimit => Self::PowerServoTempLimit(value),
            ConfigType::Power5vTempLimit => Self::Power5vTempLimit(value),
            ConfigType::ChargeMaxCurrent => Self::ChargeMaxCurrent(value),
            ConfigType::ChargeTempDerating => Self::ChargeTempDerating(value),
            ConfigType::ChargeTempLimit => Self::ChargeTempLimit(value),
            ConfigType::ChargeStopVoltage => Self::ChargeStopVoltage(value),
            ConfigType::ChargeStopCapacity => Self::ChargeStopCapacity(value),
        }
    }

    /// 获取类型标识
    pub fn config_type(&self) -> ConfigType {
        match self {
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

    /// 获取配置值
    pub fn value(&self) -> f32 {
        match self {
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
}

impl std::fmt::Display for Config {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let typ = self.config_type();
        write!(f, "{}: {:.2} {}", typ.name(), self.value(), typ.unit())
    }
}

/// 板级配置快照 (事件触发)
#[derive(Debug, Clone)]
pub struct BoardConfigSnapshot {
    // 触发舵机电源关断的电流
    pub servo_current_limit: f32,
    // 触发舵机电源关断的温度
    pub servo_temp_limit: f32,
    // 触发5V输出关断的温度
    pub temp_5v_limit: f32,
    // 充电最大电流
    pub charge_max_current: f32,
    // 触发充电降流的温度
    pub charge_temp_derating: f32,
    // 触发充电关断的温度
    pub charge_temp_limit: f32,
    // 充电停止电压
    pub charge_stop_voltage: f32,
    // 充电停止百分比（如0.8）
    pub charge_stop_percentage: f32,
    // pd握手电压（mV）
    pub pd_negotiated_mv: u16,
    // pd握手电流（mA）
    pub pd_negotiated_ma: u16,
    // 启用/禁用充电
    pub charge_enable: bool,
}

impl Default for BoardConfigSnapshot {
    fn default() -> Self {
        Self {
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
        }
    }
}
