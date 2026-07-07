//! # Authors
//! greenhand520
//! # Since
//! version: 0.1.0
//! # Date
//! 2026/7/4 16:19

//! UI 渲染模块

pub mod battery_chart;
pub mod battery_widget;
pub mod colors;
pub mod command_widget;
pub mod event_widget;
pub mod imu_widget;
pub mod log_widget;
pub mod servo_power_chart;
pub mod sys_info_widget;

use crate::DataSourceMode;
use crate::app::App;
use ratatui::Frame;
use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::style::{Color, Style};
use ratatui::widgets::Paragraph;

// ═══ 列宽比例配置（可自定义）═══

/// 信息区比例：IMU / System Info / Battery（总和 16）
const INFO_RATIO_IMU: u32 = 5;
const INFO_RATIO_SYSTEM: u32 = 5;
const INFO_RATIO_BATTERY: u32 = 6;

/// 底部区比例：Log / Event / Command（总和 16）
const BOTTOM_RATIO_LOG: u32 = 8;
const BOTTOM_RATIO_EVENT: u32 = 3;
const BOTTOM_RATIO_CMD: u32 = 5;

pub fn render(f: &mut Frame, app: &App, mode: DataSourceMode) {
    if app.show_config_query {
        render_with_config_query(f, app, mode);
    } else if app.config_editor.active {
        render_with_config_editor(f, app, mode);
    } else {
        render_normal(f, app, mode);
    }
}

/// 分割主布局：状态栏 + 图表区 + 信息区 + 底部区
fn split_main_layout(area: Rect) -> [Rect; 4] {
    let status_height = 1u16;
    let info_height = 9u16;
    let bottom_height = 5u16;
    let chart_height = area
        .height
        .saturating_sub(status_height + info_height + bottom_height)
        .max(10);

    Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(status_height),
            Constraint::Length(chart_height),
            Constraint::Length(info_height),
            Constraint::Length(bottom_height),
        ])
        .split(area)
        .as_ref()
        .try_into()
        .unwrap()
}

/// 分割图表区：左右各50%
fn split_chart_area(area: Rect) -> [Rect; 2] {
    Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
        .split(area)
        .as_ref()
        .try_into()
        .unwrap()
}

/// 分割信息区：IMU + System Info + Battery
fn split_info_area(area: Rect) -> [Rect; 3] {
    let total = INFO_RATIO_IMU + INFO_RATIO_SYSTEM + INFO_RATIO_BATTERY;
    Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Ratio(INFO_RATIO_IMU, total),
            Constraint::Ratio(INFO_RATIO_SYSTEM, total),
            Constraint::Ratio(INFO_RATIO_BATTERY, total),
        ])
        .split(area)
        .as_ref()
        .try_into()
        .unwrap()
}

/// 分割底部区：Log + Event + Command
fn split_bottom_area(area: Rect) -> [Rect; 3] {
    let total = BOTTOM_RATIO_LOG + BOTTOM_RATIO_EVENT + BOTTOM_RATIO_CMD;
    Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Ratio(BOTTOM_RATIO_LOG, total),
            Constraint::Ratio(BOTTOM_RATIO_EVENT, total),
            Constraint::Ratio(BOTTOM_RATIO_CMD, total),
        ])
        .split(area)
        .as_ref()
        .try_into()
        .unwrap()
}

/// 渲染图表区
fn render_charts(f: &mut Frame, area: Rect, app: &App) {
    let [left, right] = split_chart_area(area);
    servo_power_chart::render(f, left, app);
    battery_chart::render(f, right, app);
}

/// 渲染信息区
fn render_info_widgets(f: &mut Frame, area: Rect, app: &App) {
    let [imu, system, battery] = split_info_area(area);
    imu_widget::render(f, imu, app);
    sys_info_widget::render(f, system, app);
    battery_widget::render(f, battery, app);
}

/// 渲染底部区
fn render_bottom_widgets(f: &mut Frame, area: Rect, app: &App) {
    let [log, event, cmd] = split_bottom_area(area);
    log_widget::render(f, log, app);
    event_widget::render(f, event, app);
    command_widget::render(f, cmd, app);
}

fn render_normal(f: &mut Frame, app: &App, mode: DataSourceMode) {
    let [status, chart, info, bottom] = split_main_layout(f.area());

    render_connection_status(f, status, app, mode);
    render_charts(f, chart, app);
    render_info_widgets(f, info, app);
    render_bottom_widgets(f, bottom, app);
}

fn render_with_config_query(f: &mut Frame, app: &App, mode: DataSourceMode) {
    let [status, chart, info, bottom] = split_main_layout(f.area());

    render_connection_status(f, status, app, mode);
    render_charts(f, chart, app);
    render_info_widgets(f, info, app);
    render_bottom_widgets(f, bottom, app);
}

fn render_with_config_editor(f: &mut Frame, app: &App, mode: DataSourceMode) {
    let [status, chart, info, bottom] = split_main_layout(f.area());

    render_connection_status(f, status, app, mode);
    render_charts(f, chart, app);
    render_info_widgets(f, info, app);
    render_bottom_widgets(f, bottom, app);
}

fn render_connection_status(f: &mut Frame, area: Rect, app: &App, mode: DataSourceMode) {
    let snap = &app.current_data;

    let mode_str = match mode {
        DataSourceMode::Serial => "📡 Serial",
        DataSourceMode::Ros2 => "🤖 ROS2",
        DataSourceMode::Mock => "💯 Mock",
    };

    let status = if snap.connected {
        let config_status = if app.config_received {
            "SYNCED"
        } else {
            "NOT SYNCED"
        };
        format!(
            "{} | 🟢 Connected | Frame: {} | Config: {} | Last updated: {:.1}s ago",
            mode_str,
            snap.frame_count,
            config_status,
            snap.last_update.elapsed().as_secs_f64()
        )
    } else {
        format!("{} | 🔴 Not Connected | Trying to reconnect...", mode_str)
    };

    let style = if snap.connected {
        Style::default().fg(Color::Green)
    } else {
        Style::default().fg(Color::Red)
    };

    let paragraph = Paragraph::new(status).style(style);
    f.render_widget(paragraph, area);
}
