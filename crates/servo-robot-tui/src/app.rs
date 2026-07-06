//! App 状态管理

use crate::data_source::{DataSource, DataSnapshot};
use servo_robot_driver::protocol::config::Config;
use servo_robot_driver::protocol::config::ConfigType;

use std::collections::VecDeque;
use std::time::{Duration, Instant};

/// 图表数据点
#[derive(Debug, Clone)]
pub struct ChartData {
    pub data: VecDeque<(f64, f64)>,
    pub max_points: usize,
}

impl ChartData {
    pub fn new(max_points: usize) -> Self {
        ChartData {
            data: VecDeque::with_capacity(max_points),
            max_points,
        }
    }

    pub fn push(&mut self, time: f64, value: f64) {
        if self.data.len() >= self.max_points {
            self.data.pop_front();
        }
        self.data.push_back((time, value));
    }
}

/// 舵机电源图表
#[derive(Debug, Clone)]
pub struct PowerChart {
    pub voltage: ChartData,
    pub current: ChartData,
}

impl PowerChart {
    pub fn new() -> Self {
        PowerChart {
            voltage: ChartData::new(200),
            current: ChartData::new(200),
        }
    }
}

/// 电池图表
#[derive(Debug, Clone)]
pub struct BatteryChart {
    pub voltage: ChartData,
    pub current: ChartData,
    pub power: ChartData,
    pub charge_voltage: Option<ChartData>,
    pub charge_current: Option<ChartData>,
    pub charge_power: Option<ChartData>,
    pub charge_temp: Option<ChartData>,
}

impl BatteryChart {
    pub fn new() -> Self {
        BatteryChart {
            voltage: ChartData::new(200),
            current: ChartData::new(200),
            power: ChartData::new(200),
            charge_voltage: None,
            charge_current: None,
            charge_power: None,
            charge_temp: None,
        }
    }

    pub fn ensure_charge_charts(&mut self) {
        if self.charge_voltage.is_none() {
            self.charge_voltage = Some(ChartData::new(200));
            self.charge_current = Some(ChartData::new(200));
            self.charge_power = Some(ChartData::new(200));
            self.charge_temp = Some(ChartData::new(200));
        }
    }
}

/// 配置编辑器
#[derive(Debug, Clone)]
pub struct ConfigEditor {
    pub active: bool,
    pub selected_index: usize,
    pub scroll_offset: usize,
    pub editing: bool,
    pub edit_buffer: String,
    pub configs: Vec<(ConfigType, f32)>,
}

impl ConfigEditor {
    pub fn new() -> Self {
        ConfigEditor {
            active: false,
            selected_index: 0,
            scroll_offset: 0,
            editing: false,
            edit_buffer: String::new(),
            configs: vec![
                (ConfigType::PowerServoCurrentLimit, 5.0),
                (ConfigType::PowerServoTempLimit, 80.0),
                (ConfigType::Power5vTempLimit, 70.0),
                (ConfigType::ChargeMaxCurrent, 9.0),
                (ConfigType::ChargeTempDerating, 60.0),
                (ConfigType::ChargeTempLimit, 70.0),
                (ConfigType::ChargeStopVoltage, 16.8),
                (ConfigType::ChargeStopSoc, 1.0),
            ],
        }
    }

    pub fn update_from_config(&mut self, config: &servo_robot_driver::protocol::config::BoardConfigSnapshot) {
        self.configs[0].1 = config.servo_current_limit;
        self.configs[1].1 = config.servo_temp_limit;
        self.configs[2].1 = config.temp_5v_limit;
        self.configs[3].1 = config.charge_max_current;
        self.configs[4].1 = config.charge_temp_derating;
        self.configs[5].1 = config.charge_temp_limit;
        self.configs[6].1 = config.charge_stop_voltage;
        self.configs[7].1 = config.charge_stop_percentage;
    }
}

/// 确认对话框状态
#[derive(Debug, Clone)]
pub struct ConfirmDialog {
    pub active: bool,
    pub message: String,
    pub pending_action: Option<PendingAction>,
}

#[derive(Debug, Clone)]
pub enum PendingAction {
    Reset,
    Shutdown,
    SwitchServoPower(bool),
    Switch5VPower(bool),
    SwitchCharge(bool),
    SwitchBatExtOut(bool),
}

/// App 状态
pub struct App {
    pub data_source: Box<dyn DataSource>,
    pub current_data: DataSnapshot,
    pub power_chart: PowerChart,
    pub battery_chart: BatteryChart,
    pub config: Option<servo_robot_driver::protocol::config::BoardConfigSnapshot>,
    pub config_received: bool,
    pub last_config_query: Instant,
    pub config_editor: ConfigEditor,
    pub confirm_dialog: ConfirmDialog,
    pub show_config_query: bool,
    pub should_quit: bool,
    pub start_time: Instant,
    pub log_state: tui_logger::TuiWidgetState,
    pub log_focus: bool,
    pub event_log: VecDeque<String>,
    pub protection_event_count: u64,
    pub local_tx_log_level: servo_robot_driver::protocol::log::LogLevel,
}

impl App {
    pub fn new(data_source: Box<dyn DataSource>) -> Self {
        App {
            data_source,
            current_data: DataSnapshot::default(),
            power_chart: PowerChart::new(),
            battery_chart: BatteryChart::new(),
            config: None,
            config_received: false,
            last_config_query: Instant::now(),
            config_editor: ConfigEditor::new(),
            confirm_dialog: ConfirmDialog {
                active: false,
                message: String::new(),
                pending_action: None,
            },
            show_config_query: false,
            should_quit: false,
            start_time: Instant::now(),
            log_state: tui_logger::TuiWidgetState::new(),
            log_focus: false,
            event_log: VecDeque::new(),
            protection_event_count: 0,
            local_tx_log_level: servo_robot_driver::protocol::log::LogLevel::Info,
        }
    }

    /// 显示确认对话框
    pub fn show_confirm(&mut self, action: PendingAction) {
        self.confirm_dialog.active = true;
        self.confirm_dialog.message = match &action {
            PendingAction::Reset => "确认要复位设备吗？".to_string(),
            PendingAction::Shutdown => "确认要关机吗？".to_string(),
            PendingAction::SwitchServoPower(on) => format!("确认要{}舵机电源吗？", if *on { "开启" } else { "关闭" }),
            PendingAction::Switch5VPower(on) => format!("确认要{}5V电源吗？", if *on { "开启" } else { "关闭" }),
            PendingAction::SwitchCharge(on) => format!("确认要{}充电吗？", if *on { "开启" } else { "关闭" }),
            PendingAction::SwitchBatExtOut(on) => format!("确认要{}电池额外输出吗？", if *on { "开启" } else { "关闭" }),
        };
        self.confirm_dialog.pending_action = Some(action);
    }

    /// 确认操作
    pub fn confirm_action(&mut self) {
        if let Some(action) = self.confirm_dialog.pending_action.take() {
            match action {
                PendingAction::Reset => self.write_config(Config::Reset),
                PendingAction::Shutdown => self.write_config(Config::Shutdown),
                PendingAction::SwitchServoPower(on) => self.write_config(Config::SwitchServoPower(on)),
                PendingAction::Switch5VPower(on) => self.write_config(Config::Switch5VPower(on)),
                PendingAction::SwitchCharge(on) => self.write_config(Config::SwitchCharge(on)),
                PendingAction::SwitchBatExtOut(on) => self.write_config(Config::SwitchBatExtOut(on)),
            }
        }
        self.confirm_dialog.active = false;
    }

    /// 取消操作
    pub fn cancel_action(&mut self) {
        self.confirm_dialog.active = false;
        self.confirm_dialog.pending_action = None;
    }

    pub fn update(&mut self) {
        let snapshot = self.data_source.snapshot();

        // 更新图表数据
        let time = self.start_time.elapsed().as_secs_f64();

        if let Some(power) = &snapshot.power {
            self.power_chart.voltage.push(time, power.servo_voltage as f64);
            self.power_chart.current.push(time, power.servo_current as f64);
        }

        if let Some(battery) = &snapshot.battery {
            self.battery_chart.voltage.push(time, battery.voltage as f64);
            self.battery_chart.current.push(time, battery.current as f64);
            self.battery_chart.power.push(time, (battery.voltage * battery.current) as f64);
        }

        // 更新充电数据（只需要 power 数据）
        if let Some(power) = &snapshot.power {
            if power.charge_in_voltage > 0.5 {
                self.battery_chart.ensure_charge_charts();
                if let Some(ref mut cv) = self.battery_chart.charge_voltage {
                    cv.push(time, power.charge_in_voltage as f64);
                }
                if let Some(ref mut cc) = self.battery_chart.charge_current {
                    cc.push(time, power.charge_in_current as f64);
                }
                if let Some(ref mut cp) = self.battery_chart.charge_power {
                    cp.push(time, (power.charge_in_voltage * power.charge_in_current) as f64);
                }
                // 充电温度从 thermal 获取
                if let Some(thermal) = &snapshot.thermal {
                    if let Some(ref mut ct) = self.battery_chart.charge_temp {
                        ct.push(time, thermal.temp_charge as f64);
                    }
                }
            }
        }

        // 更新配置
        if let Some(config) = &snapshot.config {
            self.config = Some(config.clone());
            self.config_received = true;
            self.config_editor.update_from_config(config);
        }

        // 更新事件日志
        if let Some(evt) = &snapshot.event {
            let flags = evt.protection_flags;

            // 统计保护事件数量
            if !flags.is_empty() {
                self.protection_event_count += 1;

                // 添加保护事件到日志
                if flags.contains(servo_robot_driver::protocol::event::ProtectionFlags::SERVO_OVERCURRENT) {
                    self.push_event("SERVO_OVERCURRENT".to_string());
                }
                if flags.contains(servo_robot_driver::protocol::event::ProtectionFlags::SERVO_THERMAL) {
                    self.push_event("SERVO_THERMAL".to_string());
                }
                if flags.contains(servo_robot_driver::protocol::event::ProtectionFlags::DCDC_5V_THERMAL) {
                    self.push_event("DCDC_5V_THERMAL".to_string());
                }
                if flags.contains(servo_robot_driver::protocol::event::ProtectionFlags::CHARGE_DERATING) {
                    self.push_event("CHARGE_DERATING".to_string());
                }
                if flags.contains(servo_robot_driver::protocol::event::ProtectionFlags::CHARGE_THERMAL) {
                    self.push_event("CHARGE_THERMAL".to_string());
                }
                if flags.contains(servo_robot_driver::protocol::event::ProtectionFlags::BATTERY_LOW) {
                    self.push_event("BATTERY_LOW".to_string());
                }
            }

            // 充电状态变化
            if evt.charger_connected {
                self.push_event("CHARGER_CONNECTED".to_string());
            }
        }

        // 自动查询配置
        if snapshot.connected && !self.config_received {
            let now = Instant::now();
            if now.duration_since(self.last_config_query) > Duration::from_secs(5) {
                self.data_source.query_all_configs().ok();
                self.last_config_query = now;
            }
        }

        self.current_data = snapshot;
    }

    /// 添加事件到日志队列，保持最大3条（底部区域5行，边框2行，可见3行）
    fn push_event(&mut self, event: String) {
        if self.event_log.len() >= 3 {
            self.event_log.pop_front();
        }
        self.event_log.push_back(event);
    }

    

    pub fn write_config(&self, config: Config) {
        self.data_source.write_config(config).ok();
    }

    pub fn query_all_configs(&self) {
        self.data_source.query_all_configs().ok();
    }
}
