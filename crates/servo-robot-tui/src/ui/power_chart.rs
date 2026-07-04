//! 舵机电源折线图 + 文本（仅显示电流）

use ratatui::Frame;
use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::style::{Color, Style};
use ratatui::symbols;
use ratatui::text::{Line, Span};
use ratatui::widgets::{Axis, Block, Borders, Chart, Dataset, GraphType, Paragraph};
use crate::app::App;
use crate::ui::colors;

pub fn render(f: &mut Frame, area: Rect, app: &App) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Min(5),    // 折线图
            Constraint::Length(3), // 文本信息
        ])
        .split(area);

    render_chart(f, chunks[0], app);
    render_text(f, chunks[1], app);
}

fn render_chart(f: &mut Frame, area: Rect, app: &App) {
    let current_data: Vec<(f64, f64)> = app.power_chart.current.data.iter().cloned().collect();

    // 计算时间范围
    let x_min = app.power_chart.current.data.front().map(|(t, _)| *t).unwrap_or(0.0);
    let x_max = app.power_chart.current.data.back().map(|(t, _)| *t).unwrap_or(60.0);

    // Y 轴范围：0-25A
    let y_min = 0.0;
    let y_max = 25.0;

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
                .title("🦿 Servo Power")
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::Cyan)),
        )
        .x_axis(
            Axis::default()
                .title("时间 (s)")
                .style(Style::default().fg(Color::Gray))
                .bounds([x_min.max(x_max - 60.0), x_max])
                .labels(vec![
                    Span::styled(format!("{:.0}", x_min.max(x_max - 60.0)), Style::default().fg(Color::Gray)),
                    Span::styled(format!("{:.0}", x_max), Style::default().fg(Color::Gray)),
                ]),
        )
        .y_axis(
            Axis::default()
                .title("电流(A)")
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
}

fn render_text(f: &mut Frame, area: Rect, app: &App) {
    let config = app.config.as_ref();
    let (voltage, current, power, temp) = if let Some(p) = &app.current_data.power {
        let power = p.servo_voltage * p.servo_current;
        let temp = app.current_data.thermal.as_ref().map(|t| t.temp_servo_power).unwrap_or(0.0);
        (p.servo_voltage, p.servo_current, power, temp)
    } else {
        (0.0, 0.0, 0.0, 0.0)
    };

    let current_limit = config.map(|c| c.servo_current_limit).unwrap_or(25.0);
    let temp_limit = config.map(|c| c.servo_temp_limit).unwrap_or(80.0);

    let current_color = colors::get_power_color(current, current_limit);
    let temp_color = colors::get_temp_color(temp, temp_limit);

    let text = vec![
        Line::from(vec![
            Span::styled("🦿 ", Style::default().fg(Color::Yellow)),
            Span::styled(" 电压:", Style::default().fg(Color::White)),
            Span::styled(format!("{:>5.1}V", voltage), Style::default().fg(Color::Cyan)),
            Span::raw(" "),
            Span::styled("电流:", Style::default().fg(Color::White)),
            Span::styled(format!("{:>5.1}A", current), Style::default().fg(current_color)),
            Span::raw(" "),
            Span::styled("功率:", Style::default().fg(Color::White)),
            Span::styled(format!("{:>6.1}W", power), Style::default().fg(Color::Yellow)),
            Span::raw(" "),
            Span::styled("温度:", Style::default().fg(Color::White)),
            Span::styled(format!("{:>5.1}°C", temp), Style::default().fg(temp_color)),
        ]),
    ];

    let paragraph = Paragraph::new(text)
        .block(Block::default().borders(Borders::ALL).border_style(Style::default().fg(Color::DarkGray)));

    f.render_widget(paragraph, area);
}
