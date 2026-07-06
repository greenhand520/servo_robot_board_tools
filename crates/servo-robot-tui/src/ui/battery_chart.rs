//! 电池折线图 + 底部数据叠加

use ratatui::Frame;
use ratatui::layout::Rect;
use ratatui::style::{Color, Style};
use ratatui::symbols;
use ratatui::text::{Line, Span};
use ratatui::widgets::{Axis, Block, Borders, Chart, Dataset, GraphType, Paragraph};
use crate::app::App;
use crate::ui::colors;

pub fn render(f: &mut Frame, area: Rect, app: &App) {
    let current_data: Vec<(f64, f64)> = app.battery_chart.current.data.iter().cloned().collect();
    let charge_current_data: Vec<(f64, f64)> = app.battery_chart.charge_current
        .as_ref()
        .map(|d| d.data.iter().cloned().collect())
        .unwrap_or_default();

    let x_min = app.battery_chart.current.data.front().map(|(t, _)| *t).unwrap_or(0.0);
    let x_max = app.battery_chart.current.data.back().map(|(t, _)| *t).unwrap_or(60.0);

    let y_min = -5.0;
    let y_max = 25.0;

    let has_charge = !charge_current_data.is_empty();

    let mut datasets = vec![
        Dataset::default()
            .marker(symbols::Marker::Braille)
            .graph_type(GraphType::Line)
            .style(Style::default().fg(Color::Green))
            .data(&current_data),
    ];

    if has_charge {
        datasets.push(
            Dataset::default()
                .marker(symbols::Marker::Braille)
                .graph_type(GraphType::Line)
                .style(Style::default().fg(Color::Yellow))
                .data(&charge_current_data),
        );
    }

    let chart = Chart::new(datasets)
        .block(
            Block::default()
                .title("🔋Battery")
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::Yellow)),
        )
        .x_axis(
            Axis::default()
                .title("Time (s)")
                .style(Style::default().fg(Color::Gray))
                .bounds([x_min.max(x_max - 60.0), x_max])
                .labels(vec![
                    Span::styled("0", Style::default().fg(Color::Gray)),
                ]),
        )
        .y_axis(
            Axis::default()
                .title("Current(A)")
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

    // 在图表底部叠加数据文本（内部1行）
    if area.height > 3 {
        let inner = Rect {
            x: area.x + 1,
            y: area.y + area.height - 2,
            width: area.width.saturating_sub(2),
            height: 1,
        };

        let (voltage, current, temp) = if let Some(b) = &app.current_data.battery {
            (b.voltage, b.current, b.temperature)
        } else {
            (0.0, 0.0, 0.0)
        };

        let (charge_v, charge_i) = if let Some(p) = &app.current_data.power {
            if p.charge_in_voltage > 0.5 {
                (p.charge_in_voltage, p.charge_in_current)
            } else {
                (0.0, 0.0)
            }
        } else {
            (0.0, 0.0)
        };

        let temp_color = colors::get_temp_color(temp, 45.0);

        let is_charging = charge_v > 0.5;

        let mut spans = vec![
            Span::styled("🔋 ", Style::default().fg(Color::Green)),
            Span::styled("V:", Style::default().fg(Color::White)),
            Span::styled(format!("{:>4.1}V", voltage), Style::default().fg(Color::Cyan)),
            Span::raw(" "),
            Span::styled("I:", Style::default().fg(Color::White)),
            Span::styled(format!("{:>4.1}A", current), Style::default().fg(Color::Green)),
            Span::raw(" "),
        ];

        // 不充电时显示电池功率
        if !is_charging {
            let bat_power = voltage * current;
            spans.push(Span::styled("P:", Style::default().fg(Color::White)));
            spans.push(Span::styled(format!("{:>5.1}W", bat_power), Style::default().fg(Color::Green)));
            spans.push(Span::raw(" "));
        }

        spans.push(Span::styled("T:", Style::default().fg(Color::White)));
        spans.push(Span::styled(format!("{:>4.1}°C", temp), Style::default().fg(temp_color)));

        // 充电数据
        if is_charging {
            let charge_t = app.current_data.thermal.as_ref().map(|t| t.temp_charge).unwrap_or(0.0);
            let charge_temp_color = colors::get_temp_color(charge_t, 70.0);
            let charge_power = charge_v * charge_i;
            spans.push(Span::raw("  |  "));
            spans.push(Span::styled("🔌CHG ", Style::default().fg(Color::Yellow)));
            spans.push(Span::styled("V:", Style::default().fg(Color::White)));
            spans.push(Span::styled(format!("{:>5.1}V", charge_v), Style::default().fg(Color::LightCyan)));
            spans.push(Span::raw(" "));
            spans.push(Span::styled("I:", Style::default().fg(Color::White)));
            spans.push(Span::styled(format!("{:>5.1}A", charge_i), Style::default().fg(Color::Yellow)));
            spans.push(Span::raw(" "));
            spans.push(Span::styled("P:", Style::default().fg(Color::White)));
            spans.push(Span::styled(format!("{:>5.1}W", charge_power), Style::default().fg(Color::Yellow)));
            spans.push(Span::raw(" "));
            spans.push(Span::styled("T:", Style::default().fg(Color::White)));
            spans.push(Span::styled(format!("{:>5.1}°C", charge_t), Style::default().fg(charge_temp_color)));
        }

        let text = vec![Line::from(spans)];
        let paragraph = Paragraph::new(text);
        f.render_widget(paragraph, inner);
    }
}
