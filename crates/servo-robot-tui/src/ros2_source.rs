//! ROS2 数据源实现

#[cfg(feature = "ros2")]
use rclrs::*;
#[cfg(feature = "ros2")]
use servo_robot_board_interface::{msg as board_msg, srv as board_srv};
#[cfg(feature = "ros2")]
use sensor_msgs::msg as sensor_msg;

use crate::data_source::{DataSource, DataSnapshot};
use servo_robot_driver::protocol::config::{Config, ConfigType};
use servo_robot_driver::DriverError;
use std::sync::{Arc, Mutex};
use std::time::Instant;

/// 写日志到文件
#[cfg(feature = "ros2")]
fn log_to_file(msg: &str) {
    use std::io::Write;
    if let Ok(mut f) = std::fs::OpenOptions::new()
        .create(true).append(true).open("tui_debug.log")
    {
        let _ = writeln!(f, "[{:?}] {}", std::time::SystemTime::now(), msg);
    }
}

/// ROS2 数据源
pub struct Ros2Source {
    #[cfg(feature = "ros2")]
    _node: Arc<NodeState>,
    snapshot: Arc<Mutex<DataSnapshot>>,
}

#[cfg(feature = "ros2")]
impl Ros2Source {
    pub fn new() -> Result<Self, Box<dyn std::error::Error>> {
        log_to_file("Creating ROS2 context...");
        let context = Context::default();
        let mut executor = context.create_basic_executor();
        let node = executor.create_node("servo_robot_tui")?;

        let snapshot = Arc::new(Mutex::new(DataSnapshot::default()));

        // 订阅 IMU
        let snap = snapshot.clone();
        node.create_subscription::<sensor_msg::Imu, _>(
            "/robot/board/imu",
            move |msg: sensor_msg::Imu| {
                log_to_file(">>> Received IMU!");
                let mut s = snap.lock().unwrap();
                s.imu = Some(servo_robot_driver::protocol::imu::ImuData {
                    accel: [msg.linear_acceleration.x as f32, msg.linear_acceleration.y as f32, msg.linear_acceleration.z as f32],
                    gyro: [msg.angular_velocity.x as f32, msg.angular_velocity.y as f32, msg.angular_velocity.z as f32],
                    quaternion: [msg.orientation.w as f32, msg.orientation.x as f32, msg.orientation.y as f32, msg.orientation.z as f32],
                    timestamp_ms: 0, roll: 0.0, pitch: 0.0, yaw: 0.0,
                });
                s.connected = true;
                s.last_update = Instant::now();
            },
        )?;

        // 订阅电源
        let snap = snapshot.clone();
        node.create_subscription::<board_msg::BoardPower, _>(
            "/robot/board/power",
            move |msg: board_msg::BoardPower| {
                let mut s = snap.lock().unwrap();
                s.power = Some(servo_robot_driver::protocol::power::PowerData {
                    servo_voltage: msg.servo_voltage, servo_current: msg.servo_current,
                    charge_in_voltage: msg.charge_in_voltage, charge_in_current: msg.charge_in_current,
                    bat_voltage: msg.bat_voltage, bat_current: msg.bat_current,
                });
                s.last_update = Instant::now();
            },
        )?;

        // 订阅温度
        let snap = snapshot.clone();
        node.create_subscription::<board_msg::BoardThermal, _>(
            "/robot/board/thermal",
            move |msg: board_msg::BoardThermal| {
                let mut s = snap.lock().unwrap();
                s.thermal = Some(servo_robot_driver::protocol::thermal::ThermalData {
                    temp_servo_power: msg.temp_servo_power, temp_5v_power: msg.temp_5v_power,
                    temp_mcu: msg.temp_mcu, temp_charge: msg.temp_charge,
                    temp_battery: msg.temp_battery, reserved: 0.0,
                });
                s.last_update = Instant::now();
            },
        )?;

        // 订阅系统信息
        let snap = snapshot.clone();
        node.create_subscription::<board_msg::BoardSystem, _>(
            "/robot/board/system",
            move |msg: board_msg::BoardSystem| {
                let mut s = snap.lock().unwrap();
                s.system = Some(servo_robot_driver::protocol::system::SystemInfo {
                    device_id: msg.device_id, uid: msg.uid, imu_id: msg.imu_id,
                    uptime_s: msg.uptime_s,
                    cpu_usage_percent: msg.cpu_usage_percent,
                    free_heap_kb: msg.free_heap_kb, stack_watermark_min_kb: msg.stack_watermark_min_kb,
                    i2c_error_count: msg.i2c_error_count, spi_error_count: 0,
                    uart_error_count: msg.uart_error_count, usb_error_count: 0,
                    frames_sent_total: msg.frames_sent_total,
                    pd_request_voltage: msg.pd_request_voltage, pd_request_current: msg.pd_request_current,
                });
                s.last_update = Instant::now();
            },
        )?;

        // 订阅电池
        let snap = snapshot.clone();
        node.create_subscription::<sensor_msg::BatteryState, _>(
            "/robot/board/battery",
            move |msg: sensor_msg::BatteryState| {
                let mut s = snap.lock().unwrap();
                s.battery = Some(servo_robot_driver::protocol::battery_state::BatteryState {
                    voltage: msg.voltage, current: msg.current, soc: msg.charge,
                    capacity: msg.capacity, design_capacity: msg.design_capacity,
                    percentage: msg.percentage * 100.0, temperature: msg.temperature,
                    charge_status: servo_robot_driver::protocol::battery_state::BatteryChargeStatus::from_u8(msg.power_supply_status),
                    health: servo_robot_driver::protocol::battery_state::BatteryHealth::from_u8(msg.power_supply_health),
                    technology: servo_robot_driver::protocol::battery_state::BatteryTechnology::from_u8(msg.power_supply_technology),
                    present: msg.present, serial_number: msg.serial_number.parse().unwrap_or(0),
                    cell_voltages: msg.cell_voltage.iter().map(|v| v.clamp(0.0, 4.4)).collect(),
                    cell_temperatures: msg.cell_temperature.clone(),
                });
                s.last_update = Instant::now();
            },
        )?;

        // 订阅事件
        let snap = snapshot.clone();
        node.create_subscription::<board_msg::BoardEvent, _>(
            "/robot/board/event",
            move |msg: board_msg::BoardEvent| {
                let mut s = snap.lock().unwrap();
                s.event = Some(servo_robot_driver::protocol::event::BoardEvent {
                    charger_connected: msg.charger_connected, fan_enabled: msg.fan_enabled,
                    charge_phase: servo_robot_driver::protocol::event::ChargePhase::from_u8(msg.charge_phase),
                    protection_flags: servo_robot_driver::protocol::event::ProtectionFlags::from_bits(msg.protection_flags)
                        .unwrap_or(servo_robot_driver::protocol::event::ProtectionFlags::empty()),
                    error_flags: servo_robot_driver::protocol::event::ErrorFlags::empty(),
                });
                s.last_update = Instant::now();
            },
        )?;

        // 订阅配置
        let snap = snapshot.clone();
        node.create_subscription::<board_msg::BoardConfig, _>(
            "/robot/board/config",
            move |msg: board_msg::BoardConfig| {
                let mut s = snap.lock().unwrap();
                s.config = Some(servo_robot_driver::protocol::config::BoardConfigSnapshot {
                    servo_current_limit: msg.servo_current_limit, servo_temp_limit: msg.servo_temp_limit,
                    temp_5v_limit: msg.temp_5v_limit, charge_max_current: msg.charge_max_current,
                    charge_temp_derating: msg.charge_temp_derating, charge_temp_limit: msg.charge_temp_limit,
                    charge_stop_voltage: msg.charge_stop_voltage, charge_stop_percentage: msg.charge_stop_percentage,
                    pd_negotiated_mv: 0, pd_negotiated_ma: 0,
                    charge_enable: msg.charge_enable, servo_power_on: msg.servo_power_on,
                    power_5v_on: msg.power_5v_on, charge_on: msg.charge_on, bat_ext_out_on: msg.bat_ext_out_on,
                    tx_log_level: servo_robot_driver::protocol::log::LogLevel::Info,
                });
                s.last_update = Instant::now();
            },
        )?;

        log_to_file("All subscriptions created, starting spin thread...");

        // 在后台线程持续 spin executor
        let snapshot_for_thread = snapshot.clone();
        std::thread::spawn(move || {
            log_to_file("Spin thread started");
            loop {
                executor.spin(SpinOptions::new().timeout(std::time::Duration::from_millis(100)));
                std::thread::sleep(std::time::Duration::from_millis(10));
            }
        });

        log_to_file("Ros2Source created successfully");

        Ok(Ros2Source { snapshot: snapshot_for_thread, _node: node })
    }
}

#[cfg(feature = "ros2")]
impl DataSource for Ros2Source {
    fn snapshot(&self) -> DataSnapshot {
        self.snapshot.lock().unwrap().clone()
    }

    fn write_config(&self, config: Config) -> Result<(), DriverError> {
        log_to_file(&format!("write_config: {:?}", config.config_type()));
        // 服务调用需要 executor，但 executor 在后台线程
        // 简化实现：只记录日志
        Ok(())
    }

    fn query_all_configs(&self) -> Result<(), DriverError> {
        log_to_file("query_all_configs called");
        Ok(())
    }
}
