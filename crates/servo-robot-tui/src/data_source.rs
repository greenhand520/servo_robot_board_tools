//! 数据源抽象

use servo_robot_driver::protocol::battery_state::BatteryState;
use servo_robot_driver::protocol::config::{BoardConfigSnapshot, Config};

use servo_robot_driver::protocol::event::BoardEvent;
use servo_robot_driver::protocol::imu::ImuData;
use servo_robot_driver::protocol::power::PowerData;
use servo_robot_driver::protocol::system::SystemInfo;
use servo_robot_driver::protocol::thermal::ThermalData;
use servo_robot_driver::{Driver, DriverError, DriverState};
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
    pub connected: bool,
    pub frame_count: u64,
    pub frames_parsed: u64,
    pub frames_dropped: u64,
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
            connected: false,
            frame_count: 0,
            frames_parsed: 0,
            frames_dropped: 0,
            last_update: Instant::now(),
        }
    }
}

/// 数据源 trait
pub trait DataSource: Send + 'static {
    /// 获取最新快照
    fn snapshot(&self) -> DataSnapshot;

    /// 发送命令
    

    /// 写入配置
    fn write_config(&self, config: Config) -> Result<(), DriverError>;

    /// 查询所有配置
    fn query_all_configs(&self) -> Result<(), DriverError>;
}

/// 基于 Driver 的数据源
pub struct DriverSource {
    driver: Arc<Mutex<Driver>>,
    state: Arc<DriverState>,
}

impl DriverSource {
    pub fn new(driver: Driver) -> Self {
        let state = driver.state();
        DriverSource {
            driver: Arc::new(Mutex::new(driver)),
            state,
        }
    }

    pub fn start(&self) -> Result<(), DriverError> {
        let mut driver = self.driver.lock().map_err(|_| DriverError::LockPoisoned)?;
        driver.start()
    }
}

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
            connected: snap.connected,
            frame_count: snap.frame_count,
            frames_parsed: snap.frames_parsed,
            frames_dropped: snap.frames_dropped,
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
