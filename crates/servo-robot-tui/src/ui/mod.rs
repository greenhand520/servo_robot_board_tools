//! UI 渲染模块

pub mod servo_power_chart;
pub mod battery_chart;
pub mod imu_widget;
pub mod sys_info_widget;
pub mod battery_widget;
pub mod event_widget;
pub mod command_widget;
pub mod log_widget;
pub mod colors;

use ratatui::Frame;
use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::style::{Color, Style};
use ratatui::widgets::Paragraph;
use crate::app::App;
use crate::DataSourceMode;

// ═══ 列宽配置（可自定义）═══

/// 信息区列宽：IMU / System Info / Battery
const INFO_COL_IMU: u16 = 50;
const INFO_COL_SYSTEM: u16 = 50;
const INFO_COL_BATTERY: u16 = 60;

/// 底部区列宽：Log / Event / Command
const BOTTOM_COL_LOG: u16 = 80;
const BOTTOM_COL_EVENT: u16 = 30;
const BOTTOM_COL_CMD: u16 = 50;

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
    let chart_height = area.height.saturating_sub(status_height + info_height + bottom_height).max(10);

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
    Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Length(INFO_COL_IMU),
            Constraint::Length(INFO_COL_SYSTEM),
            Constraint::Length(INFO_COL_BATTERY),
        ])
        .split(area)
        .as_ref()
        .try_into()
        .unwrap()
}

/// 分割底部区：Log + Event + Command
fn split_bottom_area(area: Rect) -> [Rect; 3] {
    Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Length(BOTTOM_COL_LOG),
            Constraint::Length(BOTTOM_COL_EVENT),
            Constraint::Length(BOTTOM_COL_CMD),
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
        #[cfg(feature = "ros2")] DataSourceMode::Ros2 => "🤖 ROS2",
    };

    let status = if snap.connected {
        let config_status = if app.config_received { "SYNCED" } else { "NOT SYNCED" };
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
