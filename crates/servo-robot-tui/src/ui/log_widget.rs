//! # Authors
//! greenhand520
//! # Since
//! version: 0.1.0
//! # Date
//! 2026/7/6 23:05

//! 日志显示组件（基于 snapshot logs）

use crate::app::App;
use ratatui::Frame;
use ratatui::layout::Rect;
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Paragraph, Wrap};
use servo_robot_driver::protocol::log::LogLevel;

pub fn render(f: &mut Frame, area: Rect, app: &App) {
    let title = if app.log_focus {
        "📝Board Log [FocusMode] [(Esc)Exit] [(↑↓)Scroll] [(←→)Level]"
    } else {
        "📝Board Log [(F)ocus]"
    };

    let border_style = if app.log_focus {
        Style::default().fg(Color::Cyan)
    } else {
        Style::default().fg(Color::Magenta)
    };

    // 按等级过滤日志
    let filter_level = app.log_level_filter as u8;
    let filtered: Vec<&servo_robot_driver::LogEntry> = app
        .logs
        .iter()
        .filter(|e| (e.msg.level as u8) >= filter_level)
        .collect();

    // 可用行数 = 区域高度 - 上下边框(2行)
    let visible_lines = area.height.saturating_sub(2) as usize;
    let total = filtered.len();

    // 计算显示范围：log_scroll=0 表示最新（底部）
    let end = total.saturating_sub(app.log_scroll);
    let start = end.saturating_sub(visible_lines);

    let mut lines: Vec<Line> = Vec::new();

    if total == 0 {
        lines.push(Line::from(Span::styled(
            "Waiting data...",
            Style::default().fg(Color::DarkGray),
        )));
    } else {
        for entry in &filtered[start..end] {
            let (level_str, color) = match entry.msg.level {
                LogLevel::Error => ("E", Color::Red),
                LogLevel::Warn => ("W", Color::Yellow),
                LogLevel::Info => ("I", Color::Green),
                LogLevel::Debug => ("D", Color::DarkGray),
                LogLevel::OFF => (" ", Color::DarkGray),
            };

            // 时间戳格式化
            let total_s = entry.ts / 1000;
            let ms = entry.ts % 1000;
            let h = (total_s % 86400) / 3600;
            let m = (total_s % 3600) / 60;
            let s = total_s % 60;

            lines.push(Line::from(vec![
                Span::styled(
                    format!("[{:02}:{:02}:{:02}.{:03}] ", h, m, s, ms),
                    Style::default().fg(Color::DarkGray),
                ),
                Span::styled(
                    format!("{} ", level_str),
                    Style::default().fg(color).add_modifier(Modifier::BOLD),
                ),
                Span::styled(
                    format!("{}::{} ", entry.msg.file_name, entry.msg.fun_name),
                    Style::default().fg(Color::Cyan),
                ),
                Span::styled(&entry.msg.msg, Style::default().fg(Color::White)),
            ]));
        }
    }

    // 底部状态栏
    let level_str = match app.log_level_filter {
        LogLevel::Error => "E",
        LogLevel::Warn => "W+",
        LogLevel::Info => "I+",
        LogLevel::Debug => "D+",
        LogLevel::OFF => "OFF",
    };
    let scroll_info = if total > 0 {
        if app.log_scroll == 0 {
            format!(" [Lv:{}] [{}/{}]", level_str, total, total)
        } else {
            format!(" [Lv:{}] [{}/{} ↓{}]", level_str, end, total, app.log_scroll)
        }
    } else {
        format!(" [Lv:{}]", level_str)
    };

    let paragraph = Paragraph::new(lines).wrap(Wrap { trim: false }).block(
        Block::default()
            .title(format!("{}{}", title, scroll_info))
            .borders(Borders::ALL)
            .border_style(border_style),
    );

    f.render_widget(paragraph, area);
}
