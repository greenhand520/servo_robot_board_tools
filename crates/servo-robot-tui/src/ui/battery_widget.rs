//! 电池组件（SOC在电池图形中间）

use ratatui::Frame;
use ratatui::layout::Rect;
use ratatui::style::{Color, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Paragraph};
use crate::app::App;
use crate::ui::colors;

pub fn render(f: &mut Frame, area: Rect, app: &App) {
    let battery = &app.current_data.battery;
    let event = &app.current_data.event;

    let mut text = vec![];

    if let Some(bat) = battery {
        let soc = bat.percentage;
        let soc_color = colors::get_battery_color(soc);

        let bar_width = 10;
        let filled = (soc / 100.0 * bar_width as f32) as usize;
        let empty = bar_width - filled;

        // 第1行：电池顶部
        text.push(Line::from(vec![
            Span::styled("┌", Style::default().fg(Color::DarkGray)),
            Span::styled("─".repeat(bar_width), Style::default().fg(Color::DarkGray)),
            Span::styled("┐", Style::default().fg(Color::DarkGray)),
        ]));

        // 第2行：电池中间 + SOC + 容量
        text.push(Line::from(vec![
            Span::styled("│", Style::default().fg(Color::DarkGray)),
            Span::styled("█".repeat(filled), Style::default().fg(soc_color)),
            Span::styled("░".repeat(empty), Style::default().fg(Color::DarkGray)),
            Span::styled("│ ", Style::default().fg(Color::DarkGray)),
            Span::styled("       SOC: ", Style::default().fg(Color::White)),
            Span::styled(format!("{:.1}%", soc), Style::default().fg(soc_color)),
            Span::raw("  "),
            Span::styled("容量: ", Style::default().fg(Color::White)),
            Span::styled(format!("{:.0}mAh", bat.soc), Style::default().fg(Color::Cyan)),
        ]));

        // 第3行：电池底部
        text.push(Line::from(vec![
            Span::styled("└", Style::default().fg(Color::DarkGray)),
            Span::styled("─".repeat(bar_width), Style::default().fg(Color::DarkGray)),
            Span::styled("┘", Style::default().fg(Color::DarkGray)),
        ]));

        // 3列数据（值左对齐）
        for i in 0..4 {
            let cell_v = bat.cell_voltages.get(i).copied().unwrap_or(0.0);
            let cell_t = bat.cell_temperatures.get(i).copied().unwrap_or(0.0);
            let v_color = colors::get_cell_voltage_color(cell_v);
            let t_color = colors::get_temp_color(cell_t, 45.0);

            // 第2列（值左对齐，宽度10）
            let col2 = match i {
                0 => format!("充满: {:<10}", format!("{:.0}mAh", bat.capacity)),
                1 => format!("设计: {:<10}", format!("{:.0}mAh", bat.design_capacity)),
                2 => format!("状态: {:<10}", format!("{:?}", bat.charge_status)),
                3 => format!("健康: {:<10}", format!("{:?}", bat.health)),
                _ => String::new(),
            };

            // 第3列（值左对齐，宽度8）
            let col3 = match i {
                0 => format!("类型: {:<8}", format!("{:?}", bat.technology)),
                1 => format!("SN:   {:<8}", bat.serial_number),
                2 => format!("在位: {:<8}", if bat.present { "是" } else { "否" }),
                3 => {
                    if let Some(evt) = event {
                        format!("充电: {:<8}", if evt.charger_connected { "已连接" } else { "未连接" })
                    } else {
                        format!("充电: {:<8}", "未知")
                    }
                }
                _ => String::new(),
            };

            text.push(Line::from(vec![
                Span::styled(format!("Cell{}: ", i + 1), Style::default().fg(Color::White)),
                Span::styled(format!("{:.2}V", cell_v), Style::default().fg(v_color)),
                Span::raw(" "),
                Span::styled(format!("{:.0}°C", cell_t), Style::default().fg(t_color)),
                Span::raw("   "),
                Span::styled(col2, Style::default().fg(Color::White)),
                Span::raw(" "),
                Span::styled(col3, Style::default().fg(Color::White)),
            ]));
        }
    } else {
        text.push(Line::from(vec![
            Span::styled("🔋 电池", Style::default().fg(Color::Yellow).add_modifier(ratatui::style::Modifier::BOLD)),
        ]));
        text.push(Line::from(vec![
            Span::styled("等待数据...", Style::default().fg(Color::DarkGray)),
        ]));
    }

    let paragraph = Paragraph::new(text)
        .block(
            Block::default()
                .title("电池状态")
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::Yellow)),
        );

    f.render_widget(paragraph, area);
}
