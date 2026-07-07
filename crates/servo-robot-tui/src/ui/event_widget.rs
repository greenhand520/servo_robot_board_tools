//! # Authors
//! greenhand520
//! # Since
//! version: 0.1.0
//! # Date
//! 2026/7/6 23:05

//! 事件显示组件（支持滚动浏览）

use crate::app::App;
use ratatui::Frame;
use ratatui::layout::Rect;
use ratatui::style::{Color, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Paragraph};
use servo_robot_driver::protocol::event::EventCategory;

pub fn render(f: &mut Frame, area: Rect, app: &App) {
    let visible_lines = area.height.saturating_sub(2) as usize;
    let total = app.event_log.len();

    let end = total.saturating_sub(app.event_scroll);
    let start = end.saturating_sub(visible_lines);

    let mut lines = vec![];

    if total == 0 {
        lines.push(Line::from(vec![Span::styled(
            "No events",
            Style::default().fg(Color::DarkGray),
        )]));
    } else {
        for event in app.event_log.range(start..end) {
            let (emoji, color) = match event.kind.category() {
                EventCategory::Protection => ("⚠️", Color::Yellow),
                EventCategory::Error => ("❌", Color::Red),
                EventCategory::Charge => ("🔌", Color::Cyan),
                EventCategory::StateChange => ("🔵", Color::Blue),
            };

            // 时间戳
            let total_s = event.ts / 1000;
            let ms = event.ts % 1000;
            let h = (total_s % 86400) / 3600;
            let m = (total_s % 3600) / 60;
            let s = total_s % 60;

            lines.push(Line::from(vec![
                Span::styled(
                    format!("[{:02}:{:02}:{:02}.{:03}] ", h, m, s, ms),
                    Style::default().fg(Color::DarkGray),
                ),
                Span::styled(
                    format!("{} ", emoji),
                    Style::default(),
                ),
                Span::styled(
                    event.kind.to_string(),
                    Style::default().fg(color),
                ),
            ]));
        }
    }

    let title = if total > 0 {
        if app.event_scroll == 0 {
            format!("💡Events [{}/{}]", total, total)
        } else {
            format!("💡Events [{}/{} ↑{}]", end, total, app.event_scroll)
        }
    } else {
        "💡Events".to_string()
    };

    let paragraph = Paragraph::new(lines).block(
        Block::default()
            .title(title)
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::Yellow)),
    );

    f.render_widget(paragraph, area);
}
