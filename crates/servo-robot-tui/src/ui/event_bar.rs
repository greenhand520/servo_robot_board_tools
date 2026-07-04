//! 事件显示栏

use ratatui::Frame;
use ratatui::layout::Rect;
use ratatui::style::{Color, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Paragraph};
use crate::app::App;
use servo_robot_driver::protocol::event::ProtectionFlags;

pub fn render(f: &mut Frame, area: Rect, app: &App) {
    let event = &app.current_data.event;

    let mut spans = vec![
        Span::styled("⚠️ Events: ", Style::default().fg(Color::White)),
    ];

    if let Some(evt) = event {
        // 充电状态
        if evt.charger_connected {
            spans.push(Span::styled("⚡充电中 ", Style::default().fg(Color::Green)));
        }

        // 风扇状态
        if evt.fan_enabled {
            spans.push(Span::styled("🌀风扇开启 ", Style::default().fg(Color::Cyan)));
        } else {
            spans.push(Span::styled("🌀风扇关闭 ", Style::default().fg(Color::DarkGray)));
        }

        // 保护标志
        let flags = evt.protection_flags;
        if flags.contains(ProtectionFlags::SERVO_OVERCURRENT) {
            spans.push(Span::styled("🔴舵机过流 ", Style::default().fg(Color::Red)));
        }
        if flags.contains(ProtectionFlags::SERVO_THERMAL) {
            spans.push(Span::styled("🔴舵机过热 ", Style::default().fg(Color::Red)));
        }
        if flags.contains(ProtectionFlags::DCDC_5V_THERMAL) {
            spans.push(Span::styled("🟡5V过热 ", Style::default().fg(Color::Yellow)));
        }
        if flags.contains(ProtectionFlags::CHARGE_DERATING) {
            spans.push(Span::styled("🟡充电降流 ", Style::default().fg(Color::Yellow)));
        }
        if flags.contains(ProtectionFlags::CHARGE_THERMAL) {
            spans.push(Span::styled("🔴充电过热 ", Style::default().fg(Color::Red)));
        }
        if flags.contains(ProtectionFlags::BATTERY_LOW) {
            spans.push(Span::styled("🔴电池低电量 ", Style::default().fg(Color::Red)));
        }

        if flags.is_empty() && !evt.charger_connected && !evt.fan_enabled {
            spans.push(Span::styled("正常", Style::default().fg(Color::Green)));
        }
    } else {
        spans.push(Span::styled("无数据", Style::default().fg(Color::DarkGray)));
    }

    let paragraph = Paragraph::new(Line::from(spans))
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::DarkGray)),
        );

    f.render_widget(paragraph, area);
}
