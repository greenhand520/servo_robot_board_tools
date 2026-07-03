//! # Authors
//! greenhand520
//! # Since
//! version: 0.1.0
//! # Date
//! 2026/7/3 13:14

// ═══ BatteryState 兼容枚举 ═══

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
#[repr(u8)]
pub enum BatteryChargeStatus {
    #[default]
    Unknown = 0,
    Charging = 1,
    // 放电
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
    // 镍氢电池
    NiMh = 1,
    // 锂离子电池
    LiOn = 2,
    // 锂聚合物电池
    LiPo = 3,
    // 磷酸铁锂电池
    LiFe = 4,
    // 镍铬电池
    NiCd = 5,
    // 锂锰电池
    LiMn = 6,
}

enum_from_u8!(
    BatteryTechnology,
    Unknown,
    NiMh = 1,
    LiOn = 2,
    LiPo = 3,
    LiFe = 4,
    NiCd = 5,
    LiMn = 6
);

/// 电池状态 (10Hz)
#[derive(Debug, Clone, Default)]
pub struct BatteryState {
    pub voltage: f32,
    pub current: f32,
    // 剩余电量
    pub soc: f32,
    // 满充容量
    pub capacity: f32,
    // 设计容量
    pub design_capacity: f32,
    // 电量百分比
    pub percentage: f32,
    pub temperature: f32,
    pub charge_status: BatteryChargeStatus,
    pub health: BatteryHealth,
    pub technology: BatteryTechnology,
    // 电池是否存在
    pub present: bool,
    // 序列号
    pub serial_number: u32,
    // 各节电池电压
    pub cell_voltages: Vec<f32>,
    // 各节电池温度
    pub cell_temperatures: Vec<f32>,
}
