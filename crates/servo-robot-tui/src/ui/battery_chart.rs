//! 电池折线图 + 文本

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
            Constraint::Length(4), // 文本信息（电池+充电+效率）
        ])
        .split(area);

    render_chart(f, chunks[0], app);
    render_text(f, chunks[1], app);
}

fn render_chart(f: &mut Frame, area: Rect, app: &App) {
    let current_data: Vec<(f64, f64)> = app.battery_chart.current.data.iter().cloned().collect();
    let charge_current_data: Vec<(f64, f64)> = app.battery_chart.charge_current
        .as_ref()
        .map(|d| d.data.iter().cloned().collect())
        .unwrap_or_default();

    // 计算时间范围
    let x_min = app.battery_chart.current.data.front().map(|(t, _)| *t).unwrap_or(0.0);
    let x_max = app.battery_chart.current.data.back().map(|(t, _)| *t).unwrap_or(60.0);

    // Y 轴范围：-5A ~ 25A
    let y_min = -5.0;
    let y_max = 25.0;

    let has_charge = !charge_current_data.is_empty();

    let mut datasets = vec![
        Dataset::default()
            .marker(symbols::Marker::Braille)
            .graph_type(GraphType::Line)
            .style(Style::default().fg(Color::Green))  // 绿色=电池电流
            .data(&current_data),
    ];

    if has_charge {
        datasets.push(
            Dataset::default()
                .marker(symbols::Marker::Braille)
                .graph_type(GraphType::Line)
                .style(Style::default().fg(Color::Yellow))  // 黄色=充电电流
                .data(&charge_current_data),
        );
    }

    let chart = Chart::new(datasets)
        .block(
            Block::default()
                .title("🔋 Battery")
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::Yellow)),
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
                    Span::styled("-5", Style::default().fg(Color::Gray)),
                    Span::styled("0", Style::default().fg(Color::Gray)),
                    Span::styled("10", Style::default().fg(Color::Gray)),
                    Span::styled("20", Style::default().fg(Color::Gray)),
                ]),
        );

    f.render_widget(chart, area);
}

fn render_text(f: &mut Frame, area: Rect, app: &App) {
    let (voltage, current, power, temp) = if let Some(b) = &app.current_data.battery {
        let power = b.voltage * b.current;
        (b.voltage, b.current, power, b.temperature)
    } else {
        (0.0, 0.0, 0.0, 0.0)
    };

    let (charge_v, charge_i, charge_p, charge_t) = if let (Some(p), Some(t)) = (&app.current_data.power, &app.current_data.thermal) {
        if p.charge_in_voltage > 0.5 {
            (p.charge_in_voltage, p.charge_in_current, p.charge_in_voltage * p.charge_in_current, t.temp_charge)
        } else {
            (0.0, 0.0, 0.0, 0.0)
        }
    } else {
        (0.0, 0.0, 0.0, 0.0)
    };

    let temp_color = colors::get_temp_color(temp, 45.0);
    let charge_temp_color = colors::get_temp_color(charge_t, 70.0);

    // 计算效率
    let efficiency = if charge_p > 0.0 {
        (power / charge_p * 100.0).min(100.0)
    } else {
        0.0
    };

    let mut text = vec![
        // 电池数据行
        Line::from(vec![
            Span::styled("🔋 ", Style::default().fg(Color::Green)),
            Span::styled("电池", Style::default().fg(Color::Green)),
            Span::styled(" 电压:", Style::default().fg(Color::White)),
            Span::styled(format!("{:>5.1}V", voltage), Style::default().fg(Color::Cyan)),
            Span::raw(" "),
            Span::styled("电流:", Style::default().fg(Color::White)),
            Span::styled(format!("{:>5.1}A", current), Style::default().fg(Color::Green)),
            Span::raw(" "),
            Span::styled("功率:", Style::default().fg(Color::White)),
            Span::styled(format!("{:>6.1}W", power), Style::default().fg(Color::Magenta)),
            Span::raw(" "),
            Span::styled("温度:", Style::default().fg(Color::White)),
            Span::styled(format!("{:>5.1}°C", temp), Style::default().fg(temp_color)),
        ]),
    ];

    // 充电数据行
    if charge_v > 0.5 {
        text.push(Line::from(vec![
            Span::styled("🔌 ", Style::default().fg(Color::Yellow)),
            Span::styled("充电", Style::default().fg(Color::Yellow)),
            Span::styled(" 电压:", Style::default().fg(Color::White)),
            Span::styled(format!("{:>5.1}V", charge_v), Style::default().fg(Color::LightCyan)),
            Span::raw(" "),
            Span::styled("电流:", Style::default().fg(Color::White)),
            Span::styled(format!("{:>5.1}A", charge_i), Style::default().fg(Color::Yellow)),
            Span::raw(" "),
            Span::styled("功率:", Style::default().fg(Color::White)),
            Span::styled(format!("{:>6.1}W", charge_p), Style::default().fg(Color::LightMagenta)),
            Span::raw(" "),
            Span::styled("温度:", Style::default().fg(Color::White)),
            Span::styled(format!("{:>5.1}°C", charge_t), Style::default().fg(charge_temp_color)),
            Span::raw(" "),
            Span::styled("效率:", Style::default().fg(Color::White)),
            Span::styled(format!("{:>5.1}%", efficiency), Style::default().fg(Color::Green)),
        ]));
    }

    let paragraph = Paragraph::new(text)
        .block(Block::default().borders(Borders::ALL).border_style(Style::default().fg(Color::DarkGray)));

    f.render_widget(paragraph, area);
}
