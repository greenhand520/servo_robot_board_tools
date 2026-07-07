//! # Authors
//! greenhand520
//! # Since
//! version: 0.1.0
//! # Date
//! 2026/7/4 16:37

//! 电池组件（SOC在电池图形中间，3列行布局）

use crate::app::App;
use crate::ui::colors;
use ratatui::Frame;
use ratatui::layout::Rect;
use ratatui::style::{Color, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Paragraph};

/// 列宽定义
const COL1_W: usize = 18;
const COL2_W: usize = 16;
const COL3_W: usize = 20;

fn cell(text: &str, width: usize, style: Style) -> Span<'static> {
    Span::styled(format!("{:<width$}", text, width = width), style)
}

fn spacer() -> Span<'static> {
    Span::raw("  ")
}

pub fn render(f: &mut Frame, area: Rect, app: &App) {
    let battery = &app.current_data.battery;
    let event = &app.current_data.event;

    let mut text = vec![];

    if let Some(bat) = battery {
        // percentage 范围 0~1，转换为 0~100 用于显示
        let soc_percent = bat.percentage * 100.0;
        let soc_color = colors::get_battery_color(soc_percent);

        let bar_width = 10;
        let filled = (bat.percentage * bar_width as f32) as usize;
        let empty = bar_width - filled;

        // 电池顶部
        text.push(Line::from(vec![
            Span::styled("┌", Style::default().fg(Color::DarkGray)),
            Span::styled("─".repeat(bar_width), Style::default().fg(Color::DarkGray)),
            Span::styled("┐", Style::default().fg(Color::DarkGray)),
        ]));

        // 电池中间 + SOC + 容量
        text.push(Line::from(vec![
            Span::styled("│", Style::default().fg(Color::DarkGray)),
            Span::styled("█".repeat(filled), Style::default().fg(soc_color)),
            Span::styled("░".repeat(empty), Style::default().fg(Color::DarkGray)),
            Span::styled("│ ", Style::default().fg(Color::DarkGray)),
            Span::styled("       SOC: ", Style::default().fg(Color::White)),
            Span::styled(format!("{:.1}%", soc_percent), Style::default().fg(soc_color)),
            Span::raw("  "),
            Span::styled("Capacity: ", Style::default().fg(Color::White)),
            Span::styled(
                format!("{:.0}mAh", bat.capacity),
                Style::default().fg(Color::Cyan),
            ),
        ]));

        // 电池底部
        text.push(Line::from(vec![
            Span::styled("└", Style::default().fg(Color::DarkGray)),
            Span::styled("─".repeat(bar_width), Style::default().fg(Color::DarkGray)),
            Span::styled("┘", Style::default().fg(Color::DarkGray)),
        ]));

        // 充电状态颜色
        let state_str = match bat.charge_status {
            servo_robot_driver::protocol::battery_state::BatteryChargeStatus::Charging => {
                "Charging"
            }
            servo_robot_driver::protocol::battery_state::BatteryChargeStatus::Discharging => {
                "Discharging"
            }
            servo_robot_driver::protocol::battery_state::BatteryChargeStatus::Full => "Full",
            servo_robot_driver::protocol::battery_state::BatteryChargeStatus::NotCharging => {
                "NotCharging"
            }
            servo_robot_driver::protocol::battery_state::BatteryChargeStatus::Unknown => "Unknown",
        };

        // 充电器状态
        use servo_robot_driver::protocol::event::StateChangeFlags;
        let charger_str = match event {
            Some(evt) if evt.state_change_flags.contains(StateChangeFlags::CHARGER_CONNECTED) => "CONNECTED",
            Some(_) => "NC",
            None => "UNKNOW",
        };

        // Row 1: Cell1 | Full容量 | State充电状态
        let c1v = bat.cell_voltages.get(0).copied().unwrap_or(0.0);
        let c1t = bat.cell_temperatures.get(0).copied().unwrap_or(0.0);
        text.push(Line::from(vec![
            cell(
                &format!("Cell1: {:.2}V {:.0}°C", c1v, c1t),
                COL1_W,
                Style::default().fg(colors::get_cell_voltage_color(c1v)),
            ),
            spacer(),
            cell(
                &format!("Full: {:.0}mAh", bat.capacity),
                COL2_W,
                Style::default().fg(Color::White),
            ),
            spacer(),
            cell(
                &format!("State: {}", state_str),
                COL3_W,
                Style::default().fg(get_charge_status_color(&bat.charge_status)),
            ),
        ]));

        // Row 2: Cell2 | Design设计容量 | Health健康
        let c2v = bat.cell_voltages.get(1).copied().unwrap_or(0.0);
        let c2t = bat.cell_temperatures.get(1).copied().unwrap_or(0.0);
        text.push(Line::from(vec![
            cell(
                &format!("Cell2: {:.2}V {:.0}°C", c2v, c2t),
                COL1_W,
                Style::default().fg(colors::get_cell_voltage_color(c2v)),
            ),
            spacer(),
            cell(
                &format!("Design: {:.0}mAh", bat.design_capacity),
                COL2_W,
                Style::default().fg(Color::White),
            ),
            spacer(),
            cell(
                &format!("Health: {:?}", bat.health),
                COL3_W,
                Style::default().fg(Color::White),
            ),
        ]));

        // Row 3: Cell3 | Tech电池类型 | Present
        let c3v = bat.cell_voltages.get(2).copied().unwrap_or(0.0);
        let c3t = bat.cell_temperatures.get(2).copied().unwrap_or(0.0);
        text.push(Line::from(vec![
            cell(
                &format!("Cell3: {:.2}V {:.0}°C", c3v, c3t),
                COL1_W,
                Style::default().fg(colors::get_cell_voltage_color(c3v)),
            ),
            spacer(),
            cell(
                &format!("Tech: {:?}", bat.technology),
                COL2_W,
                Style::default().fg(Color::White),
            ),
            spacer(),
            cell(
                &format!("Present: {}", if bat.present { "YES" } else { "NO" }),
                COL3_W,
                Style::default().fg(if bat.present {
                    Color::Green
                } else {
                    Color::Red
                }),
            ),
        ]));

        // Row 4: Cell4 | SN序列号 | Charger充电器
        let c4v = bat.cell_voltages.get(3).copied().unwrap_or(0.0);
        let c4t = bat.cell_temperatures.get(3).copied().unwrap_or(0.0);
        text.push(Line::from(vec![
            cell(
                &format!("Cell4: {:.2}V {:.0}°C", c4v, c4t),
                COL1_W,
                Style::default().fg(colors::get_cell_voltage_color(c4v)),
            ),
            spacer(),
            cell(
                &format!("SN: {}", bat.serial_number),
                COL2_W,
                Style::default().fg(Color::White),
            ),
            spacer(),
            cell(
                &format!("Charger: {}", charger_str),
                COL3_W,
                Style::default().fg(if charger_str == "CON" {
                    Color::Green
                } else {
                    Color::DarkGray
                }),
            ),
        ]));
    } else {
        text.push(Line::from(vec![Span::styled(
            "🔋 Battery",
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(ratatui::style::Modifier::BOLD),
        )]));
        text.push(Line::from(vec![Span::styled(
            "Waiting data...",
            Style::default().fg(Color::DarkGray),
        )]));
    }

    let paragraph = Paragraph::new(text).block(
        Block::default()
            .title("🔋Battery State")
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::Yellow)),
    );

    f.render_widget(paragraph, area);
}

/// 获取充电状态颜色
fn get_charge_status_color(
    status: &servo_robot_driver::protocol::battery_state::BatteryChargeStatus,
) -> Color {
    use servo_robot_driver::protocol::battery_state::BatteryChargeStatus;
    match status {
        BatteryChargeStatus::Charging => Color::Green,
        BatteryChargeStatus::Full => Color::Cyan,
        BatteryChargeStatus::Discharging => Color::Yellow,
        _ => Color::White,
    }
}
