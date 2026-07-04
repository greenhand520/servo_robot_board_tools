//! Pattern A: DriverCallback trait 定义

use crate::error::DriverError;
use crate::protocol::battery_state::BatteryState;
use crate::protocol::config::{BoardConfigSnapshot, Config};
use crate::protocol::event::BoardEvent;

use crate::protocol::imu::ImuData;
use crate::protocol::power::PowerData;
use crate::protocol::thermal::ThermalData;
use crate::protocol::system::SystemInfo;

/// 回调 trait — 实现感兴趣的回调，其余用默认空实现
///
/// # Example
///
/// ```rust
/// use servo_robot_driver::dispatch::callback::DriverCallback;
/// use servo_robot_driver::protocol::imu::ImuData;
///
/// struct MyCallback {
///     imu_count: u64,
/// }
///
/// impl DriverCallback for MyCallback {
///     fn on_imu_data(&mut self, data: &ImuData) {
///         self.imu_count += 1;
///         println!("IMU #{}: roll={:.1}", self.imu_count, data.roll);
///     }
/// }
/// ```
pub trait DriverCallback: Send + 'static {
    fn on_imu_data(&mut self, _data: &ImuData) {}
    fn on_power_data(&mut self, _data: &PowerData) {}
    fn on_thermal_data(&mut self, _data: &ThermalData) {}
    fn on_battery_state(&mut self, _state: &BatteryState) {}
    fn on_config_snapshot(&mut self, _config: &BoardConfigSnapshot) {}
    fn on_board_event(&mut self, _event: &BoardEvent) {}
    fn on_system_info(&mut self, _info: &SystemInfo) {}

    /// 命令确认回调
    fn on_ack_cmd(&mut self, _success: bool) {}

    /// 配置写入确认回调
    fn on_ack_cfg_write(&mut self, _success: bool) {}

    /// 单个配置查询响应回调
    fn on_ack_cfg_query(&mut self, _config: &Config) {}

    /// 所有配置查询响应回调
    fn on_ack_cfg_query_all(&mut self, _config: &BoardConfigSnapshot) {}

    fn on_error(&mut self, _error: &DriverError) {}
}
