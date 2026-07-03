//! # Authors
//! greenhand520
//! # Since
//! version: 0.1.0
//! # Date
//! 2026/7/3 13:43

/// 充电状态枚举
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum ChargePhase {
    Unknown = 0,
    NotCharging = 1,
    PreCharge = 2,
    Cc = 3, // Constant Current
    Cv = 4, // Constant Voltage
    Full = 5,
    Husb238Fault = 6,
    Unsupported = 7,
}

impl ChargePhase {
    pub fn from_u8(v: u8) -> Self {
        match v {
            0 => Self::Unknown,
            1 => Self::NotCharging,
            2 => Self::PreCharge,
            3 => Self::Cc,
            4 => Self::Cv,
            5 => Self::Full,
            6 => Self::Husb238Fault,
            7 => Self::Unsupported,
            _ => Self::Unknown,
        }
    }

    pub fn name(&self) -> &'static str {
        match self {
            Self::Unknown => "Unknown",
            Self::NotCharging => "Not Charging",
            Self::PreCharge => "Pre-Charge",
            Self::Cc => "CC",
            Self::Cv => "CV",
            Self::Full => "Full",
            Self::Husb238Fault => "HUSB238 Fault",
            Self::Unsupported => "Unsupported",
        }
    }
}

// ═══ 保护标志位 (可组合 bitmask) ═══

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

/// 事件 (触发式)
#[derive(Debug, Clone)]
pub struct BoardEvent {
    pub charger_connected: bool,
    pub charge_phase: ChargePhase,
    pub protection_flags: ProtectionFlags,
}