//! # Authors
//! greenhand520
//! # Since
//! version: 0.1.0
//! # Date
//! 2026/7/3 13:48

/// IMU 数据
#[derive(Debug, Clone, Default)]
pub struct ImuData {
    pub accel: [f32; 3],
    pub gyro: [f32; 3],
    pub quaternion: [f32; 4],  // w, x, y, z
    pub timestamp_ms: u32,
    // 计算值
    pub roll: f32,
    pub pitch: f32,
    pub yaw: f32,
}

/// 电源 (20Hz) - 纯电气量，24B payload
#[derive(Debug, Clone, Default)]
pub struct PowerData {
    pub servo_voltage: f32,
    pub servo_current: f32,
    pub charge_in_voltage: f32,
    pub charge_in_current: f32,
    pub bat_voltage: f32,
    pub bat_current: f32,
}

/// 温度 (5Hz) - 纯温度，24B payload
#[derive(Debug, Clone, Default)]
pub struct ThermalData {
    pub temp_servo_power: f32,
    pub temp_5v_power: f32,
    pub temp_mcu: f32,
    pub temp_charge: f32,
    pub temp_battery: f32,
    // 预留位
    pub reserved: f32,
}