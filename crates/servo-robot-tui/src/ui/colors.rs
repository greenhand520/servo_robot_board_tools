//! 颜色配置（阈值告警）

use ratatui::style::Color;

/// 根据电流/功率比例获取颜色
pub fn get_power_color(value: f32, limit: f32) -> Color {
    if limit <= 0.0 {
        return Color::White;
    }
    let ratio = value / limit;
    if ratio > 0.9 {
        Color::Red
    } else if ratio > 0.7 {
        Color::Yellow
    } else {
        Color::Green
    }
}

/// 根据温度获取颜色
pub fn get_temp_color(temp: f32, limit: f32) -> Color {
    if limit <= 0.0 {
        return Color::White;
    }
    if temp > limit {
        Color::Red
    } else if temp > limit * 0.8 {
        Color::Yellow
    } else {
        Color::Green
    }
}

/// 根据电池电量获取颜色
pub fn get_battery_color(soc: f32) -> Color {
    if soc < 20.0 {
        Color::Red
    } else if soc < 50.0 {
        Color::Yellow
    } else {
        Color::Green
    }
}

/// 根据电芯电压获取颜色
pub fn get_cell_voltage_color(voltage: f32) -> Color {
    if voltage < 3.3 {
        Color::Red
    } else if voltage < 3.5 {
        Color::Yellow
    } else if voltage > 4.2 {
        Color::Red
    } else {
        Color::Green
    }
}

/// 获取保护标志颜色
pub fn get_protection_color() -> Color {
    Color::Red
}

/// 获取充电状态颜色
pub fn get_charge_status_color(status: &servo_robot_driver::protocol::battery_state::BatteryChargeStatus) -> Color {
    use servo_robot_driver::protocol::battery_state::BatteryChargeStatus;
    match status {
        BatteryChargeStatus::Charging => Color::Green,
        BatteryChargeStatus::Full => Color::Cyan,
        BatteryChargeStatus::Discharging => Color::Yellow,
        _ => Color::White,
    }
}
