//! 事件显示组件（队列方式显示，带标题）

use ratatui::Frame;
use ratatui::layout::Rect;
use ratatui::style::{Color, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Paragraph};
use crate::app::App;

pub fn render(f: &mut Frame, area: Rect, app: &App) {
    let mut lines = vec![];

    // 事件队列
    if app.event_log.is_empty() {
        lines.push(Line::from(vec![
            Span::styled("No events", Style::default().fg(Color::DarkGray)),
        ]));
    } else {
        for event in &app.event_log {
            let color = if event.contains("OVERCURRENT") || event.contains("THERMAL") || event.contains("LOW") {
                Color::Red
            } else if event.contains("DERATING") {
                Color::Yellow
            } else {
                Color::Cyan
            };
            lines.push(Line::from(vec![
                Span::styled(format!("• {}", event), Style::default().fg(color)),
            ]));
        }
    }

    let paragraph = Paragraph::new(lines)
        .block(
            Block::default()
                .title("💡Events")
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::Yellow)),
        );

    f.render_widget(paragraph, area);
}
