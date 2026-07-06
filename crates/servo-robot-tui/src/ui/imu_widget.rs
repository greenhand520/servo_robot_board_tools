//! IMU 姿态显示（左可视化 + 右数据列，左对齐）

use ratatui::Frame;
use ratatui::layout::Rect;
use ratatui::style::{Color, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Paragraph};
use crate::app::App;

pub fn render(f: &mut Frame, area: Rect, app: &App) {
    let imu = &app.current_data.imu;

    let mut text = vec![
        Line::from(vec![
            Span::styled("IMU", Style::default().fg(Color::Cyan).add_modifier(ratatui::style::Modifier::BOLD)),
            Span::styled(" (0x70)", Style::default().fg(Color::DarkGray)),
        ]),
    ];

    if let Some(imu) = imu {
        let roll = imu.roll;
        let pitch = imu.pitch;
        let yaw = imu.yaw;

        // 姿态可视化符号
        let (roll_sym, roll_color) = if roll > 5.0 {
            ("→", Color::Red)
        } else if roll < -5.0 {
            ("←", Color::Red)
        } else {
            ("-", Color::Green)
        };

        let (pitch_sym, pitch_color) = if pitch > 5.0 {
            ("↑", Color::Red)
        } else if pitch < -5.0 {
            ("↓", Color::Red)
        } else {
            ("|", Color::Green)
        };

        // RPY 颜色
        let roll_color_val = if roll.abs() > 30.0 { Color::Red } else if roll.abs() > 10.0 { Color::Yellow } else { Color::Green };
        let pitch_color_val = if pitch.abs() > 30.0 { Color::Red } else if pitch.abs() > 10.0 { Color::Yellow } else { Color::Green };

        // 第1行：可视化 + RPY
        text.push(Line::from(vec![
            Span::raw("        "),
            Span::styled(pitch_sym, Style::default().fg(pitch_color)),
            Span::raw("        "),
            Span::styled(format!("R: {:<7}", format!("{:.1}°", roll)), Style::default().fg(roll_color_val)),
            Span::styled(format!("P: {:<7}", format!("{:.1}°", pitch)), Style::default().fg(pitch_color_val)),
            Span::styled(format!("Y: {:<7}", format!("{:.1}°", yaw)), Style::default().fg(Color::Green)),
        ]));

        // 第2行：可视化 + 四元数
        text.push(Line::from(vec![
            Span::raw("    "),
            Span::styled(roll_sym, Style::default().fg(roll_color)),
            Span::raw("---+---"),
            Span::styled(roll_sym, Style::default().fg(roll_color)),
            Span::raw("    "),
            Span::styled(format!("Q: [{:.2}, {:.2}, {:.2}, {:.2}]", 
                imu.quaternion[0], imu.quaternion[1], imu.quaternion[2], imu.quaternion[3]),
                Style::default().fg(Color::White)),
        ]));

        // 第3行：可视化 + 加速度
        text.push(Line::from(vec![
            Span::raw("        "),
            Span::styled(pitch_sym, Style::default().fg(pitch_color)),
            Span::raw("        "),
            Span::styled(format!("A: [{:>5.1}, {:>5.1}, {:>5.1}]", imu.accel[0], imu.accel[1], imu.accel[2]),
                Style::default().fg(Color::White)),
        ]));

        // 第4行：角速度
        text.push(Line::from(vec![
            Span::raw("                 "),
            Span::styled(format!("G: [{:>5.1}, {:>5.1}, {:>5.1}]", imu.gyro[0], imu.gyro[1], imu.gyro[2]),
                Style::default().fg(Color::White)),
        ]));

        // 第5行：时间戳
        text.push(Line::from(vec![
            Span::raw("                 "),
            Span::styled(format!("T: {}ms", imu.timestamp_ms), Style::default().fg(Color::DarkGray)),
        ]));
    } else {
        text.push(Line::from(vec![
            Span::styled("Waiting data...", Style::default().fg(Color::DarkGray)),
        ]));
    }

    let paragraph = Paragraph::new(text)
        .block(
            Block::default()
                .title("📐IMU posture")
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::Cyan)),
        );

    f.render_widget(paragraph, area);
}
