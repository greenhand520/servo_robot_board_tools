//! 模拟传输层实现

use crate::error::DriverError;
use crate::protocol::config::{BoardConfigSnapshot, Config, ConfigType};
use crate::protocol::frame::{FrameType, RawFrame};
use crate::transport::mock_data::*;
use crate::transport::Transport;
use std::collections::VecDeque;
use std::time::{Duration, Instant};

/// 模拟传输层
pub struct MockTransport {
    /// 帧队列
    rx_queue: VecDeque<Vec<u8>>,
    /// 高优先级帧队列（ACK 响应）
    priority_queue: VecDeque<Vec<u8>>,
    /// IMU 模拟器 (100Hz)
    imu: ImuSimulator,
    /// 电源模拟器 (20Hz)
    power: PowerSimulator,
    /// 温度模拟器 (5Hz)
    thermal: ThermalSimulator,
    /// 电池模拟器 (10Hz)
    battery: BatterySimulator,
    /// 系统信息模拟器 (1Hz)
    system: SystemSimulator,
    /// 事件模拟器 (1Hz)
    event: EventSimulator,
    /// 板级配置
    config: BoardConfigSnapshot,
    /// 上次各类数据生成时间
    last_imu: Instant,
    last_power: Instant,
    last_thermal: Instant,
    last_battery: Instant,
    last_system: Instant,
    last_event: Instant,
    /// 写入的帧
    written_frames: Vec<Vec<u8>>,
    /// 连接状态
    connected: bool,
    /// 自动断开配置
    auto_disconnect_frames: Option<u64>,
    frame_count: u64,
}

impl MockTransport {
    pub fn new() -> Self {
        let now = Instant::now();
        MockTransport {
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
        self.system.charging = charging;
        self.event.charging = charging;
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
                            // ACK 优先
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
            Config::ChargeStopCapacity(v) => self.config.charge_stop_percentage = v,
            _ => {}
        }
    }
}

impl Transport for MockTransport {
    fn read_frame(&mut self) -> Result<Vec<u8>, DriverError> {
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
                std::thread::sleep(Duration::from_millis(1));
                Ok(frame)
            }
            None => {
                std::thread::sleep(Duration::from_millis(5));
                Err(DriverError::Io("timed out".to_string()))
            }
        }
    }

    fn write_frame(&mut self, frame: &[u8]) -> Result<(), DriverError> {
        if !self.connected {
            return Err(DriverError::TransportClosed);
        }

        self.written_frames.push(frame.to_vec());
        self.handle_write(frame);
        Ok(())
    }

    fn close(&mut self) -> Result<(), DriverError> {
        self.connected = false;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mock_transport_imu() {
        let mut mock = MockTransport::new();
        mock.set_initial_attitude(10.0, 5.0, 0.0);

        std::thread::sleep(Duration::from_millis(15));

        for _ in 0..20 {
            match mock.read_frame() {
                Ok(frame) => {
                    let (raw, _) = RawFrame::decode(&frame).unwrap();
                    if raw.frame_type == FrameType::Imu {
                        let imu = crate::protocol::imu::ImuData::from_bytes(&raw.payload).unwrap();
                        assert!(imu.accel[2] > 8.0 && imu.accel[2] < 12.0);
                        assert!(imu.quaternion[0].abs() <= 1.0);
                        return;
                    }
                }
                Err(_) => {
                    std::thread::sleep(Duration::from_millis(5));
                }
            }
        }
        panic!("No IMU frame received");
    }

    #[test]
    fn test_mock_transport_config_query() {
        let mut mock = MockTransport::new();

        let query = RawFrame {
            frame_type: FrameType::CfgQuery,
            payload: vec![ConfigType::PowerServoCurrentLimit as u8],
        };
        mock.write_frame(&query.encode()).unwrap();

        // ACK 应该立即可用（高优先级）
        let response = mock.read_frame().unwrap();
        let (raw, _) = RawFrame::decode(&response).unwrap();
        assert_eq!(raw.frame_type, FrameType::AckCfgQuery);

        let config = Config::from_bytes(&raw.payload).unwrap();
        assert_eq!(config.value(), 5.0);
    }

    #[test]
    fn test_mock_transport_disconnect() {
        let mut mock = MockTransport::new();
        mock.disconnect();

        let result = mock.read_frame();
        assert!(matches!(result, Err(DriverError::TransportClosed)));
    }
}
