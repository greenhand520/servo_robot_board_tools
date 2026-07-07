//! # Authors
//! greenhand520
//! # Since
//! version: 0.1.0
//! # Date
//! 2026/7/6 21:50

//! Mock 传输层共享内核
//!
//! 包含所有模拟状态和逻辑，供 MockTransport 和 AsyncMockTransport 复用。

use super::mock_data::*;
use crate::protocol::config::{BoardConfigSnapshot, Config, ConfigType};
use crate::protocol::frame::{FrameType, RawFrame};
use crate::protocol::log::{LogLevel, LogMessage};
use std::collections::VecDeque;
use std::time::{Duration, Instant};

/// Mock 共享内核 —— 所有模拟状态和业务逻辑
pub(crate) struct MockCore {
    rx_queue: VecDeque<Vec<u8>>,
    priority_queue: VecDeque<Vec<u8>>,
    imu: ImuSimulator,
    power: PowerSimulator,
    thermal: ThermalSimulator,
    battery: BatterySimulator,
    system: SystemSimulator,
    event: EventSimulator,
    config: BoardConfigSnapshot,
    last_imu: Instant,
    last_power: Instant,
    last_thermal: Instant,
    last_battery: Instant,
    last_system: Instant,
    last_event: Instant,
    last_log: Instant,
    pub(crate) written_frames: Vec<Vec<u8>>,
    pub(crate) connected: bool,
    auto_disconnect_frames: Option<u64>,
    pub(crate) frame_count: u64,
}

impl MockCore {
    pub fn new() -> Self {
        let now = Instant::now();
        MockCore {
            rx_queue: VecDeque::new(),
            priority_queue: VecDeque::new(),
            imu: ImuSimulator::new(),
            power: PowerSimulator::new(),
            thermal: ThermalSimulator::new(),
            battery: BatterySimulator::new(),
            system: SystemSimulator::new(),
            event: EventSimulator::new(),
            config: BoardConfigSnapshot::default(),
            last_imu: now,
            last_power: now,
            last_thermal: now,
            last_battery: now,
            last_system: now,
            last_event: now,
            last_log: now,
            written_frames: Vec::new(),
            connected: true,
            auto_disconnect_frames: None,
            frame_count: 0,
        }
    }

    // ═══ 配置 setter ═══

    pub fn set_battery_soc(&mut self, percentage: f32) {
        self.battery.percentage = percentage.clamp(0.0, 1.0);
    }

    pub fn set_initial_attitude(&mut self, roll_deg: f32, pitch_deg: f32, yaw_deg: f32) {
        self.imu.set_attitude(roll_deg, pitch_deg, yaw_deg);
    }

    pub fn set_charging(&mut self, charging: bool) {
        self.power.charging = charging;
        self.battery.charging = charging;
        self.system.charging = charging;
        self.event.charging = charging;
    }

    pub fn set_charging_probability(&mut self, charging_probability: f64) {
        let charging = rand::RngExt::random_bool(&mut rand::rng(), charging_probability);
        self.set_charging(charging);
    }

    pub fn set_auto_disconnect(&mut self, after_frames: u64) {
        self.auto_disconnect_frames = Some(after_frames);
    }

    pub fn disconnect(&mut self) {
        self.connected = false;
    }

    pub fn reconnect(&mut self) {
        self.connected = true;
    }

    pub fn written_frames(&self) -> &[Vec<u8>] {
        &self.written_frames
    }

    // ═══ 连接状态检查 ═══

    /// 检查是否需要自动断开，返回 true 表示已断开
    pub fn check_disconnect(&mut self) -> bool {
        if !self.connected {
            return true;
        }
        if let Some(threshold) = self.auto_disconnect_frames {
            if self.frame_count >= threshold {
                self.connected = false;
                return true;
            }
        }
        false
    }

    // ═══ 帧读取 ═══

    /// 尝试从队列取一帧（优先 ACK，再普通帧）
    pub fn try_read_frame(&mut self) -> Option<Vec<u8>> {
        // 优先返回 ACK 响应
        if let Some(frame) = self.priority_queue.pop_front() {
            self.frame_count += 1;
            return Some(frame);
        }

        // 从队列取出一帧
        if let Some(frame) = self.rx_queue.pop_front() {
            self.frame_count += 1;
            return Some(frame);
        }

        None
    }

    /// 生成模拟数据帧入队
    pub fn generate_frames(&mut self) {
        let now = Instant::now();

        // IMU 100Hz (10ms)
        if now.duration_since(self.last_imu) >= Duration::from_millis(10) {
            let data = self.imu.generate();
            let frame = RawFrame {
                frame_type: FrameType::Imu,
                payload: data.to_bytes(),
            };
            self.rx_queue.push_back(frame.encode());
            self.last_imu = now;
        }

        // Power 20Hz (50ms)
        if now.duration_since(self.last_power) >= Duration::from_millis(50) {
            let data = self.power.generate();
            let frame = RawFrame {
                frame_type: FrameType::Power,
                payload: data.to_bytes(),
            };
            self.rx_queue.push_back(frame.encode());
            self.last_power = now;
        }

        // Thermal 5Hz (200ms)
        if now.duration_since(self.last_thermal) >= Duration::from_millis(200) {
            let dt = now.duration_since(self.last_thermal).as_secs_f32();
            let data = self.thermal.generate(dt);
            let frame = RawFrame {
                frame_type: FrameType::Thermal,
                payload: data.to_bytes(),
            };
            self.rx_queue.push_back(frame.encode());
            self.last_thermal = now;
        }

        // Battery 10Hz (100ms)
        if now.duration_since(self.last_battery) >= Duration::from_millis(100) {
            let data = self.battery.generate();
            let frame = RawFrame {
                frame_type: FrameType::Battery,
                payload: data.to_bytes(),
            };
            self.rx_queue.push_back(frame.encode());
            self.last_battery = now;
        }

        // System 1Hz (1000ms)
        if now.duration_since(self.last_system) >= Duration::from_millis(1000) {
            let data = self.system.generate();
            let frame = RawFrame {
                frame_type: FrameType::System,
                payload: data.to_bytes(),
            };
            self.rx_queue.push_back(frame.encode());
            self.last_system = now;
        }

        // Event 1Hz (1000ms)
        if now.duration_since(self.last_event) >= Duration::from_millis(1000) {
            let data = self.event.generate();
            let frame = RawFrame {
                frame_type: FrameType::Event,
                payload: data.to_bytes(),
            };
            self.rx_queue.push_back(frame.encode());
            self.last_event = now;
        }

        // Log ~0.05Hz (20000ms)，随机日志等级，受 tx_log_level 过滤
        if now.duration_since(self.last_log) >= Duration::from_millis(20000) {
            let level = self.random_log_level();
            if self.should_emit_log(level) {
                self.push_log(
                    level,
                    "mock.rs",
                    "simulate",
                    &format!("random {} log #{}", level, self.frame_count),
                );
            }
            self.last_log = now;
        }
    }

    /// 生成随机日志等级
    fn random_log_level(&self) -> LogLevel {
        match rand::RngExt::random_range(&mut rand::rng(), 0u8..4) {
            0 => LogLevel::Debug,
            1 => LogLevel::Info,
            2 => LogLevel::Warn,
            _ => LogLevel::Error,
        }
    }

    /// 根据配置的 tx_log_level 判断是否应该发送该等级的日志
    fn should_emit_log(&self, level: LogLevel) -> bool {
        (level as u8) >= (self.config.tx_log_level as u8)
    }

    // ═══ 帧写入 ═══

    /// 记录写入帧并生成 ACK 响应
    pub fn prepare_write(&mut self, frame: &[u8]) {
        self.written_frames.push(frame.to_vec());
        self.handle_write(frame);
    }

    fn handle_write(&mut self, frame: &[u8]) {
        if let Ok((raw, _)) = RawFrame::decode(frame) {
            match raw.frame_type {
                FrameType::CfgQuery => {
                    if !raw.payload.is_empty() {
                        let config_type = ConfigType::from_u8(raw.payload[0]);
                        if let Some(ct) = config_type {
                            let config = Config::from_type_value(ct, self.get_config_value(ct));
                            let ack = RawFrame {
                                frame_type: FrameType::AckCfgQuery,
                                payload: config.to_bytes(),
                            };
                            self.priority_queue.push_back(ack.encode());
                            self.push_log(
                                LogLevel::Info,
                                "config.rs",
                                "handle_query",
                                &format!("queried {:?}", ct),
                            );
                        }
                    }
                }
                FrameType::CfgQueryAll => {
                    let ack = RawFrame {
                        frame_type: FrameType::AckCfgQueryAll,
                        payload: self.config.to_bytes(),
                    };
                    self.priority_queue.push_back(ack.encode());
                    self.push_log(
                        LogLevel::Info,
                        "config.rs",
                        "handle_query",
                        "queried all configs",
                    );
                }
                FrameType::CfgWrite => {
                    if let Ok(config) = Config::from_bytes(&raw.payload) {
                        self.update_config(config);
                        let ack = RawFrame {
                            frame_type: FrameType::AckCfgWrite,
                            payload: vec![1],
                        };
                        self.priority_queue.push_back(ack.encode());
                        // 发布更新后的全量配置（模拟 STM32 行为）
                        self.publish_config();
                        self.push_log(
                            LogLevel::Info,
                            "config.rs",
                            "handle_write",
                            &format!("config updated: {:?}", config),
                        );
                    }
                }
                _ => {}
            }
        }
    }

    /// 生成一条模拟板级日志并入队
    fn push_log(&mut self, level: LogLevel, file_name: &str, fun_name: &str, msg: &str) {
        let log_msg = LogMessage {
            level,
            file_name: file_name.into(),
            fun_name: fun_name.into(),
            msg: msg.into(),
        };
        let frame = RawFrame {
            frame_type: FrameType::Log,
            payload: log_msg.to_bytes(),
        };
        self.rx_queue.push_back(frame.encode());
    }

    // ═══ 配置管理 ═══

    /// 发布当前全量配置帧（模拟 STM32 在配置变更后主动上报）
    fn publish_config(&mut self) {
        let frame = RawFrame {
            frame_type: FrameType::Config,
            payload: self.config.to_bytes(),
        };
        self.rx_queue.push_back(frame.encode());
    }

    fn get_config_value(&self, ct: ConfigType) -> f32 {
        match ct {
            ConfigType::PowerServoCurrentLimit => self.config.servo_current_limit,
            ConfigType::PowerServoTempLimit => self.config.servo_temp_limit,
            ConfigType::Power5vTempLimit => self.config.temp_5v_limit,
            ConfigType::ChargeMaxCurrent => self.config.charge_max_current,
            ConfigType::ChargeTempDerating => self.config.charge_temp_derating,
            ConfigType::ChargeTempLimit => self.config.charge_temp_limit,
            ConfigType::ChargeStopVoltage => self.config.charge_stop_voltage,
            ConfigType::ChargeStopSoc => self.config.charge_stop_percentage,
            ConfigType::TxLogLevel => self.config.tx_log_level as u8 as f32,
            ConfigType::SwitchServoPower => self.config.power_servo_on as u8 as f32,
            ConfigType::Switch5VPower => self.config.power_5v_on as u8 as f32,
            ConfigType::SwitchCharge => self.config.charge_on as u8 as f32,
            ConfigType::SwitchBatExtOut => self.config.bat_ext_out_on as u8 as f32,
            _ => 0.0,
        }
    }

    fn update_config(&mut self, config: Config) {
        match config {
            Config::PowerServoCurrentLimit(v) => self.config.servo_current_limit = v,
            Config::PowerServoTempLimit(v) => self.config.servo_temp_limit = v,
            Config::Power5vTempLimit(v) => self.config.temp_5v_limit = v,
            Config::ChargeMaxCurrent(v) => self.config.charge_max_current = v,
            Config::ChargeTempDerating(v) => self.config.charge_temp_derating = v,
            Config::ChargeTempLimit(v) => self.config.charge_temp_limit = v,
            Config::ChargeStopVoltage(v) => self.config.charge_stop_voltage = v,
            Config::ChargeStopSoc(v) => self.config.charge_stop_percentage = v,
            Config::TxLogLevel(level) => self.config.tx_log_level = level,
            Config::SwitchPowerServo(on) => self.config.power_servo_on = on,
            Config::SwitchPower5V(on) => self.config.power_5v_on = on,
            Config::SwitchCharge(on) => self.config.charge_on = on,
            Config::SwitchBatExtOut(on) => self.config.bat_ext_out_on = on,
            Config::Reset | Config::Shutdown => {}
        }
    }
}
