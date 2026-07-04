//! 异步模拟传输层实现

use crate::error::DriverError;
use crate::protocol::config::{BoardConfigSnapshot, Config, ConfigType};
use crate::protocol::frame::{FrameType, RawFrame};
use crate::transport::async_trait::AsyncTransport;
use crate::transport::mock_data::*;
use std::collections::VecDeque;
use std::future::Future;
use std::pin::Pin;
use std::time::{Duration, Instant};

/// 异步模拟传输层
///
/// 复用 mock_data.rs 中的数据生成器，使用 tokio 异步 I/O
#[cfg(all(feature = "mock", feature = "async"))]
pub struct AsyncMockTransport {
    rx_queue: VecDeque<Vec<u8>>,
    priority_queue: VecDeque<Vec<u8>>,
    imu: ImuSimulator,
    power: PowerSimulator,
    thermal: ThermalSimulator,
    battery: BatterySimulator,
    system: SystemSimulator,
    config: BoardConfigSnapshot,
    last_imu: Instant,
    last_power: Instant,
    last_thermal: Instant,
    last_battery: Instant,
    last_system: Instant,
    written_frames: Vec<Vec<u8>>,
    connected: bool,
    auto_disconnect_frames: Option<u64>,
    frame_count: u64,
}

#[cfg(all(feature = "mock", feature = "async"))]
impl AsyncMockTransport {
    pub fn new() -> Self {
        let now = Instant::now();
        AsyncMockTransport {
            rx_queue: VecDeque::new(),
            priority_queue: VecDeque::new(),
            imu: ImuSimulator::new(),
            power: PowerSimulator::new(),
            thermal: ThermalSimulator::new(),
            battery: BatterySimulator::new(),
            system: SystemSimulator::new(),
            config: BoardConfigSnapshot::default(),
            last_imu: now,
            last_power: now,
            last_thermal: now,
            last_battery: now,
            last_system: now,
            written_frames: Vec::new(),
            connected: true,
            auto_disconnect_frames: None,
            frame_count: 0,
        }
    }

    pub fn set_battery_percentage(&mut self, percentage: f32) {
        self.battery.percentage = percentage.clamp(0.0, 100.0);
    }

    pub fn set_initial_attitude(&mut self, roll_deg: f32, pitch_deg: f32, yaw_deg: f32) {
        self.imu.set_attitude(roll_deg, pitch_deg, yaw_deg);
    }

    pub fn set_charging(&mut self, charging: bool) {
        self.power.charging = charging;
        self.battery.charging = charging;
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

    fn generate_frames(&mut self) {
        let now = Instant::now();

        if now.duration_since(self.last_imu) >= Duration::from_millis(10) {
            let data = self.imu.generate();
            let frame = RawFrame {
                frame_type: FrameType::Imu,
                payload: data.to_bytes(),
            };
            self.rx_queue.push_back(frame.encode());
            self.last_imu = now;
        }

        if now.duration_since(self.last_power) >= Duration::from_millis(50) {
            let data = self.power.generate();
            let frame = RawFrame {
                frame_type: FrameType::Power,
                payload: data.to_bytes(),
            };
            self.rx_queue.push_back(frame.encode());
            self.last_power = now;
        }

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

        if now.duration_since(self.last_battery) >= Duration::from_millis(100) {
            let data = self.battery.generate();
            let frame = RawFrame {
                frame_type: FrameType::Battery,
                payload: data.to_bytes(),
            };
            self.rx_queue.push_back(frame.encode());
            self.last_battery = now;
        }

        if now.duration_since(self.last_system) >= Duration::from_millis(1000) {
            let data = self.system.generate();
            let frame = RawFrame {
                frame_type: FrameType::System,
                payload: data.to_bytes(),
            };
            self.rx_queue.push_back(frame.encode());
            self.last_system = now;
        }
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
                        }
                    }
                }
                FrameType::CfgQueryAll => {
                    let ack = RawFrame {
                        frame_type: FrameType::AckCfgQueryAll,
                        payload: self.config.to_bytes(),
                    };
                    self.priority_queue.push_back(ack.encode());
                }
                FrameType::CfgWrite => {
                    if let Ok(config) = Config::from_bytes(&raw.payload) {
                        self.update_config(config);
                        let ack = RawFrame {
                            frame_type: FrameType::AckCfgWrite,
                            payload: vec![1],
                        };
                        self.priority_queue.push_back(ack.encode());
                    }
                }
                FrameType::Cmd => {
                    let ack = RawFrame {
                        frame_type: FrameType::AckCfgWrite,
                        payload: vec![1],
                    };
                    self.priority_queue.push_back(ack.encode());
                }
                _ => {}
            }
        }
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
            ConfigType::ChargeStopCapacity => self.config.charge_stop_percentage,
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
            Config::ChargeStopCapacity(v) => self.config.charge_stop_percentage = v,
        }
    }
}

#[cfg(all(feature = "mock", feature = "async"))]
impl AsyncTransport for AsyncMockTransport {
    fn read_frame(&mut self) -> Pin<Box<dyn Future<Output = Result<Vec<u8>, DriverError>> + Send + '_>> {
        Box::pin(async move {
            if !self.connected {
                return Err(DriverError::TransportClosed);
            }

            if let Some(threshold) = self.auto_disconnect_frames {
                if self.frame_count >= threshold {
                    self.connected = false;
                    return Err(DriverError::TransportClosed);
                }
            }

            // 优先返回 ACK 响应
            if let Some(frame) = self.priority_queue.pop_front() {
                self.frame_count += 1;
                return Ok(frame);
            }

            // 生成新数据帧
            self.generate_frames();

            // 从队列取出一帧
            match self.rx_queue.pop_front() {
                Some(frame) => {
                    self.frame_count += 1;
                    tokio::time::sleep(Duration::from_millis(1)).await;
                    Ok(frame)
                }
                None => {
                    tokio::time::sleep(Duration::from_millis(5)).await;
                    Err(DriverError::Io("timed out".to_string()))
                }
            }
        })
    }

    fn write_frame(&mut self, frame: &[u8]) -> Pin<Box<dyn Future<Output = Result<(), DriverError>> + Send + '_>> {
        let frame = frame.to_vec();
        Box::pin(async move {
            if !self.connected {
                return Err(DriverError::TransportClosed);
            }

            self.handle_write(&frame);
            self.written_frames.push(frame);
            Ok(())
        })
    }

    fn close(&mut self) -> Pin<Box<dyn Future<Output = Result<(), DriverError>> + Send + '_>> {
        Box::pin(async {
            self.connected = false;
            Ok(())
        })
    }
}
