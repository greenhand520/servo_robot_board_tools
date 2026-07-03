//! # Authors
//! greenhand520
//! # Since
//! version: 0.1.0
//! # Date
//! 2026/7/3 13:46

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
#[repr(u8)]
pub enum ResetReason {
    #[default]
    Unknown = 0,
    Watchdog = 1,  // IWDG
    WindowWdg = 2, // WWDG
    Software = 3,
    PowerOn = 4,
    PinReset = 5, // NRST
    BrownOut = 6, // BOR
}

enum_from_u8!(
    ResetReason,
    Unknown,
    Watchdog = 1,
    WindowWdg = 2,
    Software = 3,
    PowerOn = 4,
    PinReset = 5,
    BrownOut = 6
);
impl ResetReason {
    pub fn description(&self) -> &'static str {
        match self {
            Self::Unknown => "Unknown",
            Self::Watchdog => "Independent Watchdog Reset",
            Self::WindowWdg => "Window Watchdog Reset",
            Self::Software => "Software Reset (NVIC)",
            Self::PowerOn => "Power-On Reset",
            Self::PinReset => "NRST Pin Reset",
            Self::BrownOut => "Brown-Out Reset",
        }
    }
}

/// 系统信息 (1Hz)
#[derive(Debug, Clone, Default)]
pub struct SystemInfo {
    // ── 运行状态 ──
    pub uptime_s: u32,             //  4B  运行秒数
    pub reset_reason: ResetReason, //  1B  复位原因
    pub error_code: u8,            //  1B  全局错误码

    // ── 资源余量 ──
    pub cpu_usage_percent: u8,       //  1B  0~100 (来自 idle 任务统计)
    pub free_heap_kb: u16,           //  2B  FreeRTOS 剩余堆
    pub stack_watermark_min_kb: u16, //  2B  所有任务中最小的栈空余

    // ── 通信质量 ──
    pub i2c_error_count: u16,   //  2B  I2C 总线错误累计
    pub uart_error_count: u16,  //  2B  UART 错误累计 (帧/噪声/溢出)
    pub frames_sent_total: u32, //  4B  已发送帧总数 (上位机算丢包率)

    // ── 充电数据 ──
    pub pd_request_voltage: u16, // 2B PD握手电压
    pub pd_request_current: u16, // 2B PD握手电流
}
