//! UI 渲染模块

pub mod power_chart;
pub mod battery_chart;
pub mod imu_display;
pub mod system_info;
pub mod battery_widget;
pub mod event_bar;
pub mod command_bar;
pub mod colors;

use ratatui::Frame;
use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::style::{Color, Style};
use ratatui::widgets::Paragraph;
use crate::app::App;

pub fn render(f: &mut Frame, app: &App) {
    // 根据模式选择不同的布局
    if app.show_config_query {
        render_with_config_query(f, app);
    } else if app.config_editor.active {
        render_with_config_editor(f, app);
    } else {
        render_normal(f, app);
    }
}

fn render_normal(f: &mut Frame, app: &App) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(1),  // 连接状态栏
            Constraint::Min(10),   // 图表区域
            Constraint::Length(9),  // 详细信息区域
            Constraint::Length(3), // 事件栏
            Constraint::Length(3), // 命令栏
        ])
        .split(f.area());

    render_connection_status(f, chunks[0], app);

    let chart_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
        .split(chunks[1]);

    power_chart::render(f, chart_chunks[0], app);
    battery_chart::render(f, chart_chunks[1], app);

    let info_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Ratio(1, 3),
            Constraint::Ratio(1, 3),
            Constraint::Ratio(1, 3),
        ])
        .split(chunks[2]);

    imu_display::render(f, info_chunks[0], app);
    system_info::render(f, info_chunks[1], app);
    battery_widget::render(f, info_chunks[2], app);

    event_bar::render(f, chunks[3], app);
    command_bar::render(f, chunks[4], app);
}

fn render_with_config_query(f: &mut Frame, app: &App) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(1),  // 连接状态栏
            Constraint::Min(10),   // 图表区域
            Constraint::Length(7),  // 详细信息区域
            Constraint::Length(3), // 事件栏
            Constraint::Min(5),    // 配置查询结果
        ])
        .split(f.area());

    render_connection_status(f, chunks[0], app);

    let chart_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
        .split(chunks[1]);

    power_chart::render(f, chart_chunks[0], app);
    battery_chart::render(f, chart_chunks[1], app);

    let info_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Ratio(1, 3),
            Constraint::Ratio(1, 3),
            Constraint::Ratio(1, 3),
        ])
        .split(chunks[2]);

    imu_display::render(f, info_chunks[0], app);
    system_info::render(f, info_chunks[1], app);
    battery_widget::render(f, info_chunks[2], app);

    event_bar::render(f, chunks[3], app);
    command_bar::render(f, chunks[4], app);
}

fn render_with_config_editor(f: &mut Frame, app: &App) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(1),  // 连接状态栏
            Constraint::Min(10),   // 图表区域
            Constraint::Length(7),  // 详细信息区域
            Constraint::Length(3), // 事件栏
            Constraint::Min(5),    // 配置编辑器
        ])
        .split(f.area());

    render_connection_status(f, chunks[0], app);

    let chart_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
        .split(chunks[1]);

    power_chart::render(f, chart_chunks[0], app);
    battery_chart::render(f, chart_chunks[1], app);

    let info_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Ratio(1, 3),
            Constraint::Ratio(1, 3),
            Constraint::Ratio(1, 3),
        ])
        .split(chunks[2]);

    imu_display::render(f, info_chunks[0], app);
    system_info::render(f, info_chunks[1], app);
    battery_widget::render(f, info_chunks[2], app);

    event_bar::render(f, chunks[3], app);
    command_bar::render(f, chunks[4], app);
}

fn render_connection_status(f: &mut Frame, area: Rect, app: &App) {
    let snap = &app.current_data;
    let status = if snap.connected {
        let config_status = if app.config_received { "已同步" } else { "未同步" };
        format!(
            "🟢 已连接 | 帧: {} | 配置: {} | 上次更新: {:.1}s前",
            snap.frame_count,
            config_status,
            snap.last_update.elapsed().as_secs_f64()
        )
    } else {
        "🔴 未连接 | 尝试重连中...".to_string()
    };

    let style = if snap.connected {
        Style::default().fg(Color::Green)
    } else {
        Style::default().fg(Color::Red)
    };

    let paragraph = Paragraph::new(status).style(style);
    f.render_widget(paragraph, area);
}
