//! servo-robot-tui - STM32 监控界面
use servo_robot_driver::protocol::config::Config;

mod app;
mod data_source;
mod ui;

#[cfg(feature = "ros2")]
mod ros2_source;

use app::App;

use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode, KeyEventKind},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{backend::CrosstermBackend, Terminal};
use std::io;
use std::time::Duration;

/// 数据源模式
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum DataSourceMode {
    Serial,
    #[cfg(feature = "ros2")]
    Ros2,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 初始化 tui-logger
    tui_logger::init_logger(log::LevelFilter::Trace).unwrap();
    tui_logger::set_default_level(log::LevelFilter::Debug);

    // 根据 feature 选择数据源
    #[cfg(not(feature = "ros2"))]
    let (source, mode) = {
        use data_source::DriverSource;
        use servo_robot_driver::{Driver, MockTransport};

        let mut mock = MockTransport::new();
        mock.set_charging_probability(0.5);
        mock.set_battery_soc(75.0);
        let mut driver = Driver::new(mock);
        driver.start()?;
        (Box::new(DriverSource::new(driver)) as Box<dyn data_source::DataSource>, DataSourceMode::Serial)
    };

    #[cfg(feature = "ros2")]
    let (source, mode) = {
        let ros2_source = ros2_source::Ros2Source::new()?;
        (Box::new(ros2_source) as Box<dyn data_source::DataSource>, DataSourceMode::Ros2)
    };

    let mut app = App::new(source);

    // 设置终端
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // 主循环
    loop {
        if event::poll(Duration::from_millis(50))? {
            if let Event::Key(key) = event::read()? {
                if key.kind == KeyEventKind::Press {
                    handle_key_event(&mut app, key.code);
                }
            }
        }

        // Spin ROS2 executor more aggressively
        #[cfg(feature = "ros2")]
        {
            // The executor is already being spun in snapshot() calls
            // but we can add additional spinning here if needed
        }

        app.update();

        // 移动 tui-logger 事件到 widget
        tui_logger::move_events();

        terminal.draw(|f| ui::render(f, &app, mode))?;

        if app.should_quit {
            break;
        }
    }

    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;

    Ok(())
}

fn handle_key_event(app: &mut App, code: KeyCode) {
    if app.confirm_dialog.active {
        match code {
            KeyCode::Char('y') | KeyCode::Char('Y') => app.confirm_action(),
            KeyCode::Char('n') | KeyCode::Char('N') | KeyCode::Esc => app.cancel_action(),
            _ => {}
        }
        return;
    }

    // 配置查询模式 - 支持滚动
    if app.show_config_query {
        match code {
            KeyCode::Esc => {
                app.show_config_query = false;
                app.config_editor.scroll_offset = 0;
            }
            KeyCode::Up => {
                app.config_editor.scroll_offset = app.config_editor.scroll_offset.saturating_sub(1);
            }
            KeyCode::Down => {
                app.config_editor.scroll_offset += 1;
            }
            _ => {}
        }
        return;
    }

    // 配置编辑器模式 - 支持选择和滚动
    if app.config_editor.active {
        match code {
            KeyCode::Esc => {
                app.config_editor.active = false;
                app.config_editor.editing = false;
                app.config_editor.edit_buffer.clear();
                app.config_editor.scroll_offset = 0;
            }
            KeyCode::Up if !app.config_editor.editing && app.config_editor.selected_index > 0 => {
                app.config_editor.selected_index -= 1;
                if app.config_editor.selected_index < app.config_editor.scroll_offset {
                    app.config_editor.scroll_offset = app.config_editor.selected_index;
                }
            }
            KeyCode::Down if !app.config_editor.editing && app.config_editor.selected_index < app.config_editor.configs.len() - 1 => {
                app.config_editor.selected_index += 1;
                let visible_lines = 2;
                if app.config_editor.selected_index >= app.config_editor.scroll_offset + visible_lines {
                    app.config_editor.scroll_offset = app.config_editor.selected_index - visible_lines + 1;
                }
            }
            KeyCode::Enter => {
                if app.config_editor.editing {
                    if let Ok(value) = app.config_editor.edit_buffer.parse::<f32>() {
                        let idx = app.config_editor.selected_index;
                        app.config_editor.configs[idx].1 = value;
                        let config = Config::from_type_value(app.config_editor.configs[idx].0, value);
                        app.write_config(config);
                    }
                    app.config_editor.editing = false;
                    app.config_editor.edit_buffer.clear();
                } else {
                    app.config_editor.editing = true;
                    let idx = app.config_editor.selected_index;
                    app.config_editor.edit_buffer = format!("{:.1}", app.config_editor.configs[idx].1);
                }
            }
            KeyCode::Char(c) if app.config_editor.editing => {
                app.config_editor.edit_buffer.push(c);
            }
            KeyCode::Backspace if app.config_editor.editing => {
                app.config_editor.edit_buffer.pop();
            }
            _ => {}
        }
        return;
    }

    // 日志组件焦点模式 - 传递按键给 tui-logger
    if app.log_focus {
        match code {
            KeyCode::Esc => {
                app.log_focus = false;
                return;
            }
            KeyCode::PageUp => {
                app.log_state.transition(tui_logger::TuiWidgetEvent::PrevPageKey);
                return;
            }
            KeyCode::PageDown => {
                app.log_state.transition(tui_logger::TuiWidgetEvent::NextPageKey);
                return;
            }
            KeyCode::Up => {
                app.log_state.transition(tui_logger::TuiWidgetEvent::FocusKey);
                return;
            }
            KeyCode::Down => {
                app.log_state.transition(tui_logger::TuiWidgetEvent::FocusKey);
                return;
            }
            KeyCode::Left => {
                app.log_state.transition(tui_logger::TuiWidgetEvent::EscapeKey);
                return;
            }
            KeyCode::Right => {
                app.log_state.transition(tui_logger::TuiWidgetEvent::RightKey);
                return;
            }
            KeyCode::Char(' ') => {
                app.log_state.transition(tui_logger::TuiWidgetEvent::SpaceKey);
                return;
            }
            _ => {}
        }
    }

    match code {
        KeyCode::Esc => {
            if app.show_config_query {
                app.show_config_query = false;
            } else {
                app.should_quit = true;
            }
        }
        KeyCode::Char('r') => app.show_confirm(app::PendingAction::Reset),
        KeyCode::Char('s') => app.show_confirm(app::PendingAction::Shutdown),
        KeyCode::Char('1') => {
            let on = app.config.as_ref().map(|c| !c.servo_power_on).unwrap_or(true);
            app.show_confirm(app::PendingAction::SwitchServoPower(on));
        }
        KeyCode::Char('2') => {
            let on = app.config.as_ref().map(|c| !c.power_5v_on).unwrap_or(true);
            app.show_confirm(app::PendingAction::Switch5VPower(on));
        }
        KeyCode::Char('3') => {
            let on = app.config.as_ref().map(|c| !c.charge_on).unwrap_or(true);
            app.show_confirm(app::PendingAction::SwitchCharge(on));
        }
        KeyCode::Char('4') => {
            let on = app.config.as_ref().map(|c| !c.bat_ext_out_on).unwrap_or(true);
            app.show_confirm(app::PendingAction::SwitchBatExtOut(on));
        }
        KeyCode::Char('q') => {
            app.query_all_configs();
            app.show_config_query = true;
        }
        KeyCode::Char('w') => {
            app.config_editor.active = true;
        }
        KeyCode::Char('l') | KeyCode::Char('L') => {
            // 循环切换日志等级: Off -> Debug -> Info -> Warn -> Error -> Off
            let current_level = app.local_tx_log_level;
            let next_level = match current_level {
                servo_robot_driver::protocol::log::LogLevel::OFF => servo_robot_driver::protocol::log::LogLevel::Debug,
                servo_robot_driver::protocol::log::LogLevel::Debug => servo_robot_driver::protocol::log::LogLevel::Info,
                servo_robot_driver::protocol::log::LogLevel::Info => servo_robot_driver::protocol::log::LogLevel::Warn,
                servo_robot_driver::protocol::log::LogLevel::Warn => servo_robot_driver::protocol::log::LogLevel::Error,
                servo_robot_driver::protocol::log::LogLevel::Error => servo_robot_driver::protocol::log::LogLevel::OFF,
            };
            app.local_tx_log_level = next_level;
            app.write_config(Config::TxLogLevel(next_level));
            log::info!("[CMD] SetTxLogLevel: {:?} -> {:?}", current_level, next_level);
        }
        KeyCode::Char('f') | KeyCode::Char('F') => {
            // 切换日志组件焦点模式
            app.log_focus = !app.log_focus;
        }
        _ => {}
    }
}
