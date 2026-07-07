//! # Authors
//! greenhand520
//! # Since
//! version: 0.1.0
//! # Date
//! 2026/7/6 23:05

//! 命令栏（支持配置编辑器、确认对话框、动态颜色、滚动）

use crate::app::App;
use ratatui::Frame;
use ratatui::layout::Rect;
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Paragraph};

pub fn render(f: &mut Frame, area: Rect, app: &App) {
    if app.confirm_dialog.active {
        render_confirm_dialog(f, area, app);
    } else if app.show_config_query {
        render_config_query(f, area, app);
    } else if app.config_editor.active {
        render_config_editor(f, area, app);
    } else {
        render_command_bar(f, area, app);
    }
}

fn render_command_bar(f: &mut Frame, area: Rect, app: &App) {
    let servo_on = app
        .config
        .as_ref()
        .map(|c| c.power_servo_on)
        .unwrap_or(false);
    let power_5v_on = app.config.as_ref().map(|c| c.power_5v_on).unwrap_or(false);
    let charging = app.config.as_ref().map(|c| c.charge_on).unwrap_or(false);
    let bat_ext_on = app
        .config
        .as_ref()
        .map(|c| c.bat_ext_out_on)
        .unwrap_or(false);

    let servo_color = if servo_on { Color::Green } else { Color::Red };
    let power_5v_color = if power_5v_on {
        Color::Green
    } else {
        Color::Red
    };
    let charge_color = if charging { Color::Green } else { Color::Red };
    let bat_ext_color = if bat_ext_on { Color::Green } else { Color::Red };

    let log_level_str = format!("{:?}", app.local_tx_log_level);

    let lines = vec![
        Line::from(vec![
            Span::styled(
                "[R]",
                Style::default().fg(Color::Red).add_modifier(Modifier::BOLD),
            ),
            Span::raw(" Reset  "),
            Span::styled(
                "[S]",
                Style::default().fg(Color::Red).add_modifier(Modifier::BOLD),
            ),
            Span::raw(" Shutdown  "),
            Span::styled(
                "[L]",
                Style::default()
                    .fg(Color::Cyan)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled(
                format!(" LogLvl:{}", log_level_str),
                Style::default().fg(Color::Cyan),
            ),
        ]),
        Line::from(vec![
            Span::styled(
                "[1]",
                Style::default()
                    .fg(servo_color)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::raw(" Servo  "),
            Span::styled(
                "[2]",
                Style::default()
                    .fg(power_5v_color)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::raw(" 5V  "),
            Span::styled(
                "[3]",
                Style::default()
                    .fg(charge_color)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::raw(" Charge  "),
            Span::styled(
                "[4]",
                Style::default()
                    .fg(bat_ext_color)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::raw(" BatExt"),
        ]),
        Line::from(vec![
            Span::styled(
                "[Q]",
                Style::default()
                    .fg(Color::Blue)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::raw(" QueryAllCfg  "),
            Span::styled(
                "[W]",
                Style::default()
                    .fg(Color::Blue)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::raw(" WriteCfg  "),
            Span::styled("[Esc]", Style::default().fg(Color::DarkGray)),
            Span::raw(" Quit"),
        ]),
    ];

    let paragraph = Paragraph::new(lines).block(
        Block::default()
            .title("📱Commands")
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::DarkGray)),
    );

    f.render_widget(paragraph, area);
}

fn render_confirm_dialog(f: &mut Frame, area: Rect, app: &App) {
    let spans = vec![
        Span::styled("⚠️ ", Style::default().fg(Color::Yellow)),
        Span::styled(
            &app.confirm_dialog.message,
            Style::default()
                .fg(Color::White)
                .add_modifier(Modifier::BOLD),
        ),
        Span::raw("  "),
        Span::styled(
            "[Y]",
            Style::default()
                .fg(Color::Green)
                .add_modifier(Modifier::BOLD),
        ),
        Span::raw(" Enter  "),
        Span::styled(
            "[N]",
            Style::default().fg(Color::Red).add_modifier(Modifier::BOLD),
        ),
        Span::raw(" Cancel"),
    ];

    let paragraph = Paragraph::new(Line::from(spans)).block(
        Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::Yellow)),
    );

    f.render_widget(paragraph, area);
}

fn render_config_query(f: &mut Frame, area: Rect, app: &App) {
    // 可用行数 = 区域高度 - 上下边框(2行)
    let visible_lines = area.height.saturating_sub(2) as usize;
    let scroll = app.config_editor.scroll_offset;

    let mut all_lines = vec![];

    if let Some(config) = &app.config {
        all_lines.push(Line::from(vec![Span::styled(
            format!("PowerServoCurrentLimit: {:.1} A", config.servo_current_limit),
            Style::default().fg(Color::White),
        )]));
        all_lines.push(Line::from(vec![Span::styled(
            format!("PowerServoTempLimit: {:.1} °C", config.servo_temp_limit),
            Style::default().fg(Color::White),
        )]));
        all_lines.push(Line::from(vec![Span::styled(
            format!("Power5vTempLimit: {:.1} °C", config.temp_5v_limit),
            Style::default().fg(Color::White),
        )]));
        all_lines.push(Line::from(vec![Span::styled(
            format!("ChargeMaxCurrent: {:.1} A", config.charge_max_current),
            Style::default().fg(Color::White),
        )]));
        all_lines.push(Line::from(vec![Span::styled(
            format!("ChargeTempDerating: {:.1} °C", config.charge_temp_derating),
            Style::default().fg(Color::White),
        )]));
        all_lines.push(Line::from(vec![Span::styled(
            format!("ChargeTempLimit: {:.1} °C", config.charge_temp_limit),
            Style::default().fg(Color::White),
        )]));
        all_lines.push(Line::from(vec![Span::styled(
            format!("ChargeStopVoltage: {:.1} V", config.charge_stop_voltage),
            Style::default().fg(Color::White),
        )]));
        all_lines.push(Line::from(vec![Span::styled(
            format!(
                "ChargeStopSoc: {:.1}%",
                config.charge_stop_percentage * 100.0
            ),
            Style::default().fg(Color::White),
        )]));
    } else {
        all_lines.push(Line::from(vec![Span::styled(
            "WaitingConfigData...",
            Style::default().fg(Color::DarkGray),
        )]));
    }

    // 只取可见范围内的行
    let start = scroll.min(all_lines.len().saturating_sub(1));
    let lines: Vec<Line> = all_lines
        .into_iter()
        .skip(start)
        .take(visible_lines)
        .collect();

    // 标题显示滚动位置和Esc提示
    let title = if scroll > 0 {
        format!(
            "CfgQueryRes [↑↓Scro] [Esc Quit] (↓{}/{})",
            start + 1,
            visible_lines
        )
    } else {
        "CfgQueryRes [↑↓Scro] [Esc Quit]".to_string()
    };

    let paragraph = Paragraph::new(lines).block(
        Block::default()
            .title(title)
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::Blue)),
    );

    f.render_widget(paragraph, area);
}

fn render_config_editor(f: &mut Frame, area: Rect, app: &App) {
    let editor = &app.config_editor;
    // 可用行数 = 区域高度 - 上下边框(2行)
    let visible_lines = area.height.saturating_sub(2) as usize;
    let scroll = editor.scroll_offset;

    // 确保选中项在可见范围内
    let mut lines = vec![];

    lines.push(Line::from(vec![
        Span::styled(
            "⚙️ ConfigEditor",
            Style::default()
                .fg(Color::Blue)
                .add_modifier(Modifier::BOLD),
        ),
        Span::raw("  "),
        Span::styled("[↑↓]", Style::default().fg(Color::Cyan)),
        Span::raw(" Choose  "),
        Span::styled("[Enter]", Style::default().fg(Color::Green)),
        Span::raw(" Edit/Enter  "),
        Span::styled("[Esc]", Style::default().fg(Color::Red)),
        Span::raw(" Quit"),
    ]));

    // 只显示可见范围内的配置项
    for (i, (config_type, value)) in editor
        .configs
        .iter()
        .enumerate()
        .skip(scroll)
        .take(visible_lines.saturating_sub(1))
    {
        let selected = i == editor.selected_index;
        let prefix = if selected { "▶ " } else { "  " };
        let style = if selected {
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD)
        } else {
            Style::default().fg(Color::White)
        };

        let value_str = if selected && editor.editing {
            editor.edit_buffer.clone()
        } else {
            format!("{:.1} {}", value, config_type.unit())
        };

        lines.push(Line::from(vec![
            Span::styled(prefix, style),
            Span::styled(format!("{:<25}", config_type.name()), style),
            Span::styled(
                value_str,
                if selected && editor.editing {
                    Style::default()
                        .fg(Color::Green)
                        .add_modifier(Modifier::UNDERLINED)
                } else {
                    style
                },
            ),
        ]));
    }

    let paragraph = Paragraph::new(lines).block(
        Block::default()
            .title("ConfigEditor")
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::Blue)),
    );

    f.render_widget(paragraph, area);
}
