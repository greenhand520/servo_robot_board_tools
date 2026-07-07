//! # Authors
//! greenhand520
//! # Since
//! version: 0.1.0
//! # Date
//! 2026/7/6 21:19

//! 舵机电源折线图 + 底部数据叠加

use crate::app::App;
use crate::ui::colors;
use ratatui::Frame;
use ratatui::layout::Rect;
use ratatui::style::{Color, Style};
use ratatui::symbols;
use ratatui::text::{Line, Span};
use ratatui::widgets::{Axis, Block, Borders, Chart, Dataset, GraphType, Paragraph};

pub fn render(f: &mut Frame, area: Rect, app: &App) {
    let current_data: Vec<(f64, f64)> = app.power_chart.current.data.iter().cloned().collect();

    let x_min = app
        .power_chart
        .current
        .data
        .front()
        .map(|(t, _)| *t)
        .unwrap_or(0.0);
    let x_max = app
        .power_chart
        .current
        .data
        .back()
        .map(|(t, _)| *t)
        .unwrap_or(60.0);

    let y_min = 0.0;
    let y_max = 25.0;

    // 渲染折线图
    let datasets = vec![
        Dataset::default()
            .marker(symbols::Marker::Braille)
            .graph_type(GraphType::Line)
            .style(Style::default().fg(Color::Yellow))
            .data(&current_data),
    ];

    let chart = Chart::new(datasets)
        .block(
            Block::default()
                .title("🦿Servo Power")
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::Cyan)),
        )
        .x_axis(
            Axis::default()
                .title("Time (s)")
                .style(Style::default().fg(Color::Gray))
                .bounds([x_min.max(x_max - 60.0), x_max])
                .labels(vec![Span::styled("0", Style::default().fg(Color::Gray))]),
        )
        .y_axis(
            Axis::default()
                .title("Current(A)")
                .style(Style::default().fg(Color::Gray))
                .bounds([y_min, y_max])
                .labels(vec![
                    Span::styled("0", Style::default().fg(Color::Gray)),
                    Span::styled("5", Style::default().fg(Color::Gray)),
                    Span::styled("10", Style::default().fg(Color::Gray)),
                    Span::styled("15", Style::default().fg(Color::Gray)),
                    Span::styled("20", Style::default().fg(Color::Gray)),
                    Span::styled("25", Style::default().fg(Color::Gray)),
                ]),
        );

    f.render_widget(chart, area);

    // 在图表底部叠加数据文本（内部1行）
    if area.height > 3 {
        let inner = Rect {
            x: area.x + 1,
            y: area.y + area.height - 2,
            width: area.width.saturating_sub(2),
            height: 1,
        };

        let config = app.config.as_ref();
        let (voltage, current, power, temp) = if let Some(p) = &app.current_data.power {
            let power = p.servo_voltage * p.servo_current;
            let temp = app
                .current_data
                .thermal
                .as_ref()
                .map(|t| t.temp_servo_power)
                .unwrap_or(0.0);
            (p.servo_voltage, p.servo_current, power, temp)
        } else {
            (0.0, 0.0, 0.0, 0.0)
        };

        let current_limit = config.map(|c| c.servo_current_limit).unwrap_or(25.0);
        let temp_limit = config.map(|c| c.servo_temp_limit).unwrap_or(80.0);
        let current_color = colors::get_power_color(current, current_limit);
        let temp_color = colors::get_temp_color(temp, temp_limit);

        let text = vec![Line::from(vec![
            Span::styled("⚡ ", Style::default().fg(Color::White)),
            Span::styled("V:", Style::default().fg(Color::White)),
            Span::styled(
                format!("{:>4.1}V", voltage),
                Style::default().fg(Color::Cyan),
            ),
            Span::raw("  "),
            Span::styled("I:", Style::default().fg(Color::White)),
            Span::styled(
                format!("{:>5.1}A", current),
                Style::default().fg(current_color),
            ),
            Span::raw("  "),
            Span::styled("P:", Style::default().fg(Color::White)),
            Span::styled(
                format!("{:>6.1}W", power),
                Style::default().fg(Color::Yellow),
            ),
            Span::raw("  "),
            Span::styled("T:", Style::default().fg(Color::White)),
            Span::styled(format!("{:>5.1}°C", temp), Style::default().fg(temp_color)),
        ])];

        let paragraph = Paragraph::new(text);
        f.render_widget(paragraph, inner);
    }
}
