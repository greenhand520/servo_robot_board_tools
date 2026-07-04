//! STM32 系统信息（2列布局，含运行时间）

use ratatui::Frame;
use ratatui::layout::Rect;
use ratatui::style::{Color, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Paragraph};
use crate::app::App;
use crate::ui::colors;

pub fn render(f: &mut Frame, area: Rect, app: &App) {
    let system = &app.current_data.system;
    let thermal = &app.current_data.thermal;

    let mut text = vec![];

    // 温度信息
    let (mcu_temp, temp_5v) = if let Some(t) = thermal {
        (t.temp_mcu, t.temp_5v_power)
    } else {
        (0.0, 0.0)
    };

    let mcu_temp_color = colors::get_temp_color(mcu_temp, 60.0);
    let temp_5v_color = colors::get_temp_color(temp_5v, 70.0);

    if let Some(sys) = system {
        // 标题行
        text.push(Line::from(vec![
            Span::styled("💻 STM32", Style::default().fg(Color::Blue).add_modifier(ratatui::style::Modifier::BOLD)),
            Span::styled(format!(" (0x{:04X} 0x{:08X})", sys.device_id, sys.uid), Style::default().fg(Color::DarkGray)),
        ]));

        // 丢包率计算
        let frames_sent = sys.frames_sent_total as u64;
        let frames_parsed = app.current_data.frames_parsed;
        let loss_rate = if frames_sent > 0 {
            ((frames_sent as f64 - frames_parsed as f64) / frames_sent as f64 * 100.0).max(0.0)
        } else {
            0.0
        };
        let loss_color = if loss_rate > 5.0 {
            Color::Red
        } else if loss_rate > 1.0 {
            Color::Yellow
        } else {
            Color::Green
        };

        // 运行时间格式化
        let uptime = sys.uptime_s;
        let hours = uptime / 3600;
        let minutes = (uptime % 3600) / 60;
        let seconds = uptime % 60;
        let uptime_str = format!("{:02}:{:02}:{:02}", hours, minutes, seconds);

        // 第1列
        text.push(Line::from(vec![
            Span::styled(format!("MCU:   {:<8}", format!("{:.1}°C", mcu_temp)), Style::default().fg(mcu_temp_color)),
            Span::raw("  "),
            Span::styled(format!("CPU:   {:<8}", format!("{}%", sys.cpu_usage_percent)), Style::default().fg(Color::Green)),
        ]));
        text.push(Line::from(vec![
            Span::styled(format!("5V:    {:<8}", format!("{:.1}°C", temp_5v)), Style::default().fg(temp_5v_color)),
            Span::raw("  "),
            Span::styled(format!("Stack: {:<8}", format!("{}KB", sys.stack_watermark_min_kb)), Style::default().fg(Color::Cyan)),
        ]));
        text.push(Line::from(vec![
            Span::styled(format!("I2C:   {:<8}", format!("{}", sys.i2c_error_count)), Style::default().fg(
                if sys.i2c_error_count > 0 { Color::Red } else { Color::Green }
            )),
            Span::raw("  "),
            Span::styled(format!("Heap:  {:<8}", format!("{}KB", sys.free_heap_kb)), Style::default().fg(Color::Cyan)),
        ]));
        text.push(Line::from(vec![
            Span::styled(format!("UART:  {:<8}", format!("{}", sys.uart_error_count)), Style::default().fg(
                if sys.uart_error_count > 0 { Color::Red } else { Color::Green }
            )),
            Span::raw("  "),
            Span::styled(format!("发送:  {:<8}", format!("{}", sys.frames_sent_total)), Style::default().fg(Color::Yellow)),
        ]));
        text.push(Line::from(vec![
            Span::styled(format!("运行:  {:<8}", uptime_str), Style::default().fg(Color::White)),
            Span::raw("  "),
            Span::styled(format!("解析:  {:<8}", format!("{}", frames_parsed)), Style::default().fg(Color::Cyan)),
        ]));

        // PD握手（如果有）
        if sys.pd_request_voltage > 0 {
            let pd_v = sys.pd_request_voltage as f32 / 1000.0;
            let pd_i = sys.pd_request_current as f32 / 1000.0;
            text.push(Line::from(vec![
                Span::styled(format!("PD:    {:.0}V/{:.0}A  ", pd_v, pd_i), Style::default().fg(Color::White)),
                Span::raw("  "),
                Span::styled(format!("丢包:  {:<8}", format!("{:.1}%", loss_rate)), Style::default().fg(loss_color)),
            ]));
        } else {
            text.push(Line::from(vec![
                Span::styled("PD:    -", Style::default().fg(Color::DarkGray)),
                Span::raw("  "),
                Span::styled(format!("丢包:  {:<8}", format!("{:.1}%", loss_rate)), Style::default().fg(loss_color)),
            ]));
        }
    } else {
        text.push(Line::from(vec![
            Span::styled("💻 STM32", Style::default().fg(Color::Blue).add_modifier(ratatui::style::Modifier::BOLD)),
        ]));
        text.push(Line::from(vec![
            Span::styled("等待数据...", Style::default().fg(Color::DarkGray)),
        ]));
    }

    let paragraph = Paragraph::new(text)
        .block(
            Block::default()
                .title("系统信息")
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::Blue)),
        );

    f.render_widget(paragraph, area);
}
