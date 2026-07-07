//! # Authors
//! greenhand520
//! # Since
//! version: 0.1.0
//! # Date
//! 2026/7/4 16:16

//! 数据源抽象

use servo_robot_driver::protocol::battery_state::BatteryState;
use servo_robot_driver::protocol::config::{BoardConfigSnapshot, Config};
use servo_robot_driver::LogEntry;

#[cfg(not(feature = "ros2"))]
use crate::{DataSourceMode, data_source};
use servo_robot_driver::DriverError;
use servo_robot_driver::protocol::event::BoardEvent;
use servo_robot_driver::protocol::imu::ImuData;
use servo_robot_driver::protocol::power::PowerData;
use servo_robot_driver::protocol::system::SystemInfo;
use servo_robot_driver::protocol::thermal::ThermalData;
#[cfg(not(feature = "ros2"))]
use servo_robot_driver::{Driver, DriverCallback, DriverState};
#[cfg(not(feature = "ros2"))]
use std::sync::{Arc, Mutex};
use std::time::Instant;

/// 数据快照
#[derive(Debug, Clone)]
pub struct DataSnapshot {
    pub power: Option<PowerData>,
    pub battery: Option<BatteryState>,
    pub imu: Option<ImuData>,
    pub system: Option<SystemInfo>,
    pub thermal: Option<ThermalData>,
    pub config: Option<BoardConfigSnapshot>,
    pub event: Option<BoardEvent>,
    pub logs: Vec<LogEntry>,
    pub connected: bool,
    pub frame_count: u64,
    pub frames_parsed: u64,

    pub last_update: Instant,
}

impl Default for DataSnapshot {
    fn default() -> Self {
        DataSnapshot {
            power: None,
            battery: None,
            imu: None,
            system: None,
            thermal: None,
            config: None,
            event: None,
            logs: Vec::new(),
            connected: false,
            frame_count: 0,
            frames_parsed: 0,
            last_update: Instant::now(),
        }
    }
}

/// 数据源 trait
pub trait DataSource: Send + 'static {
    /// 获取最新快照
    fn snapshot(&self) -> DataSnapshot;

    /// 写入配置
    fn write_config(&self, config: Config) -> Result<(), DriverError>;

    /// 查询所有配置
    fn query_all_configs(&self) -> Result<(), DriverError>;
}

/// 基于 Driver 的数据源
#[cfg(not(feature = "ros2"))]
pub struct DriverSource {
    driver: Arc<Mutex<Driver>>,
    state: Arc<DriverState>,
}

#[cfg(not(feature = "ros2"))]
impl DriverSource {
    pub fn new(driver: Driver) -> Self {
        let state = driver.state();
        DriverSource {
            driver: Arc::new(Mutex::new(driver)),
            state,
        }
    }
}

#[cfg(not(feature = "ros2"))]
impl DataSource for DriverSource {
    fn snapshot(&self) -> DataSnapshot {
        let snap = self.state.snapshot();
        DataSnapshot {
            power: snap.power,
            battery: snap.battery,
            imu: snap.imu,
            system: snap.system,
            thermal: snap.thermal,
            config: snap.config,
            event: snap.event,
            logs: snap.logs.into(),
            connected: snap.connected,
            frame_count: snap.frame_count,
            frames_parsed: snap.frames_parsed,
            last_update: Instant::now(),
        }
    }

    fn write_config(&self, config: Config) -> Result<(), DriverError> {
        let driver = self.driver.lock().map_err(|_| DriverError::LockPoisoned)?;
        driver.write_config(config)
    }

    fn query_all_configs(&self) -> Result<(), DriverError> {
        let driver = self.driver.lock().map_err(|_| DriverError::LockPoisoned)?;
        driver.query_all_configs()
    }
}

/// Board log 回调 — 将板级日志转发到 tui-logger（通过 `log` 宏）
#[cfg(not(feature = "ros2"))]
struct TuiBoardLogCallback;

#[cfg(not(feature = "ros2"))]
impl DriverCallback for TuiBoardLogCallback {}

/// Mock 数据源（无需硬件）
#[cfg(all(feature = "mock", not(feature = "ros2")))]
pub(crate) fn create_mock_source()
-> Result<(Box<dyn data_source::DataSource>, DataSourceMode), Box<dyn std::error::Error>> {
    use crate::data_source::DriverSource;
    use servo_robot_driver::{Driver, MockTransport};

    let mut mock = MockTransport::new();
    mock.set_charging_probability(0.5);
    mock.set_battery_soc(0.75);
    log::info!("Data source: MockTransport");
    let mut driver = Driver::new(mock);
    driver.register_callback(TuiBoardLogCallback);
    driver.start()?;
    Ok((Box::new(DriverSource::new(driver)), DataSourceMode::Mock))
}

/// 串口数据源（从命令行参数读取串口号和波特率）
/// Usage: servo-robot-tui [PORT] [BAUD_RATE]
#[cfg(not(any(feature = "ros2", feature = "mock")))]
pub(crate) fn create_serial_source()
-> Result<(Box<dyn data_source::DataSource>, DataSourceMode), Box<dyn std::error::Error>> {
    use crate::data_source::DriverSource;
    use servo_robot_driver::{Driver, SerialTransport};

    let args: Vec<String> = std::env::args().collect();
    let port = args
        .get(1)
        .cloned()
        .unwrap_or_else(|| "/dev/ttyUSB0".to_string());
    let baud_rate = args
        .get(2)
        .and_then(|s| s.parse().ok())
        .unwrap_or(115200u32);

    log::info!("Data source: Serial {} @ {} baud", port, baud_rate);
    let transport = SerialTransport::open(&port, baud_rate)?;
    let mut driver = Driver::new(transport);
    driver.register_callback(TuiBoardLogCallback);
    driver.start()?;
    Ok((Box::new(DriverSource::new(driver)), DataSourceMode::Serial))
}
