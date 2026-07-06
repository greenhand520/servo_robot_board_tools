//! STM32 系统信息（3列布局）

use ratatui::Frame;
use ratatui::layout::Rect;
use ratatui::style::{Color, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Paragraph};
use crate::app::App;
use crate::ui::colors;

/// 列宽定义
const COL1_W: usize = 16;
const COL2_W: usize = 13;
const COL3_W: usize = 19;

fn cell(text: &str, width: usize, style: Style) -> Span<'static> {
    Span::styled(format!("{:<width$}", text, width = width), style)
}

fn spacer() -> Span<'static> {
    Span::raw(" ")
}

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
            Span::styled("MCU", Style::default().fg(Color::Blue).add_modifier(ratatui::style::Modifier::BOLD)),
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

        // PD格式化
        let pd_str = if sys.pd_request_voltage > 0 {
            let pd_v = sys.pd_request_voltage as f32 / 1000.0;
            let pd_i = sys.pd_request_current as f32 / 1000.0;
            format!("{:.0}V/{:.0}A", pd_v, pd_i)
        } else {
            "NAN/NAN".to_string()
        };

        // 错误计数颜色
        let err_color = |n: u16| if n > 0 { Color::Red } else { Color::Green };

        // Row 1: MCU温度 | CPU占有率 | TX
        text.push(Line::from(vec![
            cell(&format!("MCU: {:.1}°C", mcu_temp), COL1_W, Style::default().fg(mcu_temp_color)),
            spacer(),
            cell(&format!("CPU:  {}%", sys.cpu_usage_percent), COL2_W, Style::default().fg(Color::Green)),
            spacer(),
            cell(&format!("TX:     {}", sys.frames_sent_total), COL3_W, Style::default().fg(Color::Yellow)),
        ]));
        // Row 2: 5V温度 | I2C错误 | Parsed
        text.push(Line::from(vec![
            cell(&format!("5V:  {:.1}°C", temp_5v), COL1_W, Style::default().fg(temp_5v_color)),
            spacer(),
            cell(&format!("I2C:  {}", sys.i2c_error_count), COL2_W, Style::default().fg(err_color(sys.i2c_error_count))),
            spacer(),
            cell(&format!("Parsed: {}", frames_parsed), COL3_W, Style::default().fg(Color::Cyan)),
        ]));
        // Row 3: Heap | SPI错误 | Loss
        text.push(Line::from(vec![
            cell(&format!("Heap: {}KB", sys.free_heap_kb), COL1_W, Style::default().fg(Color::Cyan)),
            spacer(),
            cell(&format!("SPI:  {}", sys.spi_error_count), COL2_W, Style::default().fg(err_color(sys.spi_error_count))),
            spacer(),
            cell(&format!("Loss:   {:.1}%", loss_rate), COL3_W, Style::default().fg(loss_color)),
        ]));
        // Row 4: Stack | UART错误 | 运行时间
        text.push(Line::from(vec![
            cell(&format!("Stack:{:3.0}KB", sys.stack_watermark_min_kb), COL1_W, Style::default().fg(Color::Cyan)),
            spacer(),
            cell(&format!("UART: {}", sys.uart_error_count), COL2_W, Style::default().fg(err_color(sys.uart_error_count))),
            spacer(),
            cell(&format!("Run:    {}", uptime_str), COL3_W, Style::default().fg(Color::White)),
        ]));
        // Row 5: PD | USB错误
        text.push(Line::from(vec![
            cell(&format!("PD:  {}", pd_str), COL1_W, Style::default().fg(Color::White)),
            spacer(),
            cell(&format!("USB:  {}", sys.usb_error_count), COL2_W, Style::default().fg(err_color(sys.usb_error_count))),
        ]));
    } else {
        text.push(Line::from(vec![
            Span::styled("STM32", Style::default().fg(Color::Blue).add_modifier(ratatui::style::Modifier::BOLD)),
        ]));
        text.push(Line::from(vec![
            Span::styled("Waiting data...", Style::default().fg(Color::DarkGray)),
        ]));
    }

    let paragraph = Paragraph::new(text)
        .block(
            Block::default()
                .title("📊System Info")
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::Blue)),
        );

    f.render_widget(paragraph, area);
}
