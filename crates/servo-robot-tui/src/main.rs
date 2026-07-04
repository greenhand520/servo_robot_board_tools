//! servo-robot-tui - STM32 监控界面

mod app;
mod data_source;
mod ui;

use app::{App, PendingAction};
use data_source::DriverSource;
use rand::RngExt;

use servo_robot_driver::protocol::config::Config;
use servo_robot_driver::{Driver, MockTransport};

use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode, KeyEventKind},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{
    backend::CrosstermBackend,
    Terminal,
};
use std::io;
use std::time::Duration;

fn main() -> anyhow::Result<()> {
    // 创建 MockTransport，随机决定是否充电
    let mut mock = MockTransport::new();
    let charging = rand::rng().random_bool(0.5);
    mock.set_charging(charging);
    let mut driver = Driver::new(mock);
    driver.start()?;

    let source = DriverSource::new(driver);
    let mut app = App::new(Box::new(source));

    // 设置终端
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // 主循环
    loop {
        // 处理输入
        if event::poll(Duration::from_millis(50))? {
            if let Event::Key(key) = event::read()? {
                if key.kind == KeyEventKind::Press {
                    // 确认对话框模式
                    if app.confirm_dialog.active {
                        match key.code {
                            KeyCode::Char('y') | KeyCode::Char('Y') => {
                                app.confirm_action();
                            }
                            KeyCode::Char('n') | KeyCode::Char('N') | KeyCode::Esc => {
                                app.cancel_action();
                            }
                            _ => {}
                        }
                    }
                    // 配置编辑器模式
                    else if app.config_editor.active {
                        match key.code {
                            KeyCode::Esc => {
                                app.config_editor.active = false;
                                app.config_editor.editing = false;
                                app.config_editor.edit_buffer.clear();
                            }
                            KeyCode::Up => {
                                if !app.config_editor.editing && app.config_editor.selected_index > 0 {
                                    app.config_editor.selected_index -= 1;
                                }
                            }
                            KeyCode::Down => {
                                if !app.config_editor.editing && app.config_editor.selected_index < app.config_editor.configs.len() - 1 {
                                    app.config_editor.selected_index += 1;
                                }
                            }
                            KeyCode::Enter => {
                                if app.config_editor.editing {
                                    if let Ok(value) = app.config_editor.edit_buffer.parse::<f32>() {
                                        let idx = app.config_editor.selected_index;
                                        app.config_editor.configs[idx].1 = value;
                                        let config_type = app.config_editor.configs[idx].0;
                                        let config = Config::from_type_value(config_type, value);
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
                            KeyCode::Char(c) => {
                                if app.config_editor.editing {
                                    app.config_editor.edit_buffer.push(c);
                                }
                            }
                            KeyCode::Backspace => {
                                if app.config_editor.editing {
                                    app.config_editor.edit_buffer.pop();
                                }
                            }
                            _ => {}
                        }
                    } else {
                        // 普通模式
                        match key.code {
                            KeyCode::Esc => {
                                if app.show_config_query {
                                    app.show_config_query = false;
                                } else {
                                    app.should_quit = true;
                                }
                            }
                            KeyCode::Char('r') => {
                                app.show_confirm(PendingAction::Reset);
                            }
                            KeyCode::Char('s') => {
                                app.show_confirm(PendingAction::Shutdown);
                            }
                            KeyCode::Char('1') => {
                                let current = app.current_data.power.as_ref()
                                    .map(|p| p.servo_voltage > 1.0)
                                    .unwrap_or(false);
                                app.show_confirm(PendingAction::SwitchServoPower(!current));
                            }
                            KeyCode::Char('2') => {
                                let current = app.current_data.power.as_ref()
                                    .map(|p| p.charge_in_voltage > 0.5)
                                    .unwrap_or(false);
                                app.show_confirm(PendingAction::Switch5VPower(!current));
                            }
                            KeyCode::Char('3') => {
                                let charging = app.current_data.battery.as_ref()
                                    .map(|b| b.charge_status == servo_robot_driver::protocol::battery_state::BatteryChargeStatus::Charging)
                                    .unwrap_or(false);
                                app.show_confirm(PendingAction::SwitchCharge(!charging));
                            }
                            KeyCode::Char('4') => {
                                app.show_confirm(PendingAction::SwitchBatExtOut(true));
                            }
                            KeyCode::Char('q') => {
                                // 查询所有配置并显示
                                app.query_all_configs();
                                app.show_config_query = true;
                            }
                            KeyCode::Char('w') => {
                                app.config_editor.active = true;
                            }
                            _ => {}
                        }
                    }
                }
            }
        }

        // 更新数据
        app.update();

        // 渲染 UI
        terminal.draw(|f| ui::render(f, &app))?;

        if app.should_quit {
            break;
        }
    }

    // 恢复终端
    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;

    Ok(())
}
