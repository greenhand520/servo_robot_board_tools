//! # Authors
//! greenhand520
//! # Since
//! version: 0.1.0
//! # Date
//! 2026/7/4 21:21

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
