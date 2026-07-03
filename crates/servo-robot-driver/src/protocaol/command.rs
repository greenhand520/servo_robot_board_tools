//! # Authors
//! greenhand520
//! # Since
//! version: 0.1.0
//! # Date
//! 2026/7/3 14:37

/// 单个 WS2812 灯珠颜色
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct RgbLed {
    pub r: u8,
    pub g: u8,
    pub b: u8,
}

impl RgbLed {
    pub const fn new(r: u8, g: u8, b: u8) -> Self {
        Self { r, g, b }
    }

    pub const OFF: Self = Self::new(0, 0, 0);
    pub const RED: Self = Self::new(255, 0, 0);
    pub const GREEN: Self = Self::new(0, 255, 0);
    pub const BLUE: Self = Self::new(0, 0, 255);
    pub const WHITE: Self = Self::new(255, 255, 255);
    pub const YELLOW: Self = Self::new(255, 255, 0);
    pub const CYAN: Self = Self::new(0, 255, 255);
    pub const ORANGE: Self = Self::new(255, 128, 0);
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum CommandType {
    Reset = 0x01,
    Shutdown = 0x02,
    // 开关舵机电源
    SwitchServoPower = 0x03,
    // 开关5V电源
    Switch5VPower = 0x04,
    // 开关电池充电
    SwitchCharge = 0x05,
    // 开关电池额外输出
    SwitchBatExtOut = 0x06,
    // 设置3个RGB灯颜色
    SetRGB = 0x07,
}

impl CommandType {
    pub fn from_u8(v: u8) -> Option<Self> {
        match v {
            0x01 => Some(Self::Reset),
            0x02 => Some(Self::Shutdown),
            0x03 => Some(Self::SwitchServoPower),
            0x04 => Some(Self::Switch5VPower),
            0x05 => Some(Self::SwitchCharge),
            0x06 => Some(Self::SwitchBatExtOut),
            0x07 => Some(Self::SetRGB),
            _ => None,
        }
    }

    pub fn name(&self) -> &'static str {
        match self {
            Self::Reset => "Reset all device",
            Self::Shutdown => "Shutdown all device",
            Self::SwitchServoPower => "Turn the servo power on or off",
            Self::Switch5VPower => "Turn on or off 5V output",
            Self::SwitchCharge => "Turn the battery on or off to charge",
            Self::SwitchBatExtOut => "Turn the battery on or off for extra output",
            Self::SetRGB => "Set 3 RGB indicator colors",
        }
    }
}

impl std::fmt::Display for CommandType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.name())
    }
}

/// 完整命令，包含参数
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Command {
    Reset,
    Shutdown,
    SwitchServoPower(bool),
    Switch5VPower(bool),
    SwitchCharge(bool),
    SwitchBatExtOut(bool),
    SetRGB([RgbLed; 3]),
}

impl Command {
    pub fn command_type(&self) -> CommandType {
        match self {
            Self::Reset => CommandType::Reset,
            Self::Shutdown => CommandType::Shutdown,
            Self::SwitchServoPower(_) => CommandType::SwitchServoPower,
            Self::Switch5VPower(_) => CommandType::Switch5VPower,
            Self::SwitchCharge(_) => CommandType::SwitchCharge,
            Self::SwitchBatExtOut(_) => CommandType::SwitchBatExtOut,
            Self::SetRGB(_) => CommandType::SetRGB,
        }
    }
}

impl std::fmt::Display for Command {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Reset => write!(f, "Reset all device"),
            Self::Shutdown => write!(f, "Shutdown all device"),
            Self::SwitchServoPower(on) => {
                write!(f, "Switch Servo Power: {}", if *on { "ON" } else { "OFF" })
            }
            Self::Switch5VPower(on) => {
                write!(f, "Switch 5V Power: {}", if *on { "ON" } else { "OFF" })
            }
            Self::SwitchCharge(on) => {
                write!(f, "Switch Charge: {}", if *on { "ON" } else { "OFF" })
            }
            Self::SwitchBatExtOut(on) => {
                write!(
                    f,
                    "Switch Battery Extra Output: {}",
                    if *on { "ON" } else { "OFF" }
                )
            }
            Self::SetRGB(leds) => {
                write!(
                    f,
                    "Set RGB: [{},{},{}] [{},{},{}] [{},{},{}]",
                    leds[0].r,
                    leds[0].g,
                    leds[0].b,
                    leds[1].r,
                    leds[1].g,
                    leds[1].b,
                    leds[2].r,
                    leds[2].g,
                    leds[2].b,
                )
            }
        }
    }
}
