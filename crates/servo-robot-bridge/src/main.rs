//! ROS2 桥接节点

mod conversion;
mod services;

use rclrs::*;
use servo_robot_driver::{Driver, DriverCallback};
use servo_robot_driver::protocol::log::{LogLevel, LogMessage};
use std::sync::{Arc, Mutex};
use std::time::Duration;

#[cfg(feature = "mock")]
use servo_robot_driver::MockTransport;

#[cfg(not(feature = "mock"))]
use servo_robot_driver::SerialTransport;

/// ROS2 桥接回调，将板级日志转发到 ROS2 日志系统
struct BridgeCallback {
    logger: Logger,
}

impl DriverCallback for BridgeCallback {
    fn on_log(&mut self, ts: u64, log_msg: &LogMessage) {
        let total_s = ts / 1000;
        let ms = ts % 1000;
        let h = (total_s % 86400) / 3600;
        let m = (total_s % 3600) / 60;
        let s = total_s % 60;
        let msg = format!("[{:02}:{:02}:{:02}.{:03}] {}::{}: {}",
            h, m, s, ms, log_msg.file_name, log_msg.fun_name, log_msg.msg);
        match log_msg.level {
            LogLevel::Error => log_error!(&self.logger, "[servo_robot_board] {}", msg),
            LogLevel::Warn => log_warn!(&self.logger, "[servo_robot_board] {}", msg),
            LogLevel::Info | LogLevel::OFF => log_info!(&self.logger, "[servo_robot_board] {}", msg),
            LogLevel::Debug => log_debug!(&self.logger, "[servo_robot_board] {}", msg),
        }
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let context = Context::default();
    let executor = context.create_basic_executor();
    let node = executor.create_node("servo_robot_board_bridge")?;

    // 声明 ROS2 参数（可通过 --ros-args -p port:=... -p baud_rate:=... 或 YAML 参数文件设置）
    #[cfg(not(feature = "mock"))]
    let param_port: MandatoryParameter<Arc<str>> = node
        .declare_parameter("port")
        .default(Arc::from("/dev/ttyUSB0"))
        .mandatory()?;

    #[cfg(not(feature = "mock"))]
    let param_baud_rate: MandatoryParameter<i64> = node
        .declare_parameter("baud_rate")
        .default(115200)
        .mandatory()?;

    #[cfg(feature = "mock")]
    {
        let mut mock = MockTransport::new();
        mock.set_charging(true);
        mock.set_battery_soc(75.0);
        log_info!(node.logger(), "Starting with MockTransport");
        let mut driver = Driver::new(mock);
        driver.register_callback(BridgeCallback { logger: node.logger().clone() });
        driver.start()?;
        run_bridge(executor, node, driver)?;
    }

    #[cfg(not(feature = "mock"))]
    {
        let port: Arc<str> = param_port.get();
        let baud_rate = param_baud_rate.get() as u32;
        log_info!(node.logger(), "Connecting to {} at {} baud...", port, baud_rate);
        let transport = SerialTransport::open(&port, baud_rate)?;
        let mut driver = Driver::new(transport);
        driver.register_callback(BridgeCallback { logger: node.logger().clone() });
        driver.start()?;
        run_bridge(executor, node, driver)?;
    }

    Ok(())
}

/// 发布所有数据话题
fn publish_data(
    state: &servo_robot_driver::DriverState,
    logger: &Logger,
    imu_pub: &Publisher<sensor_msgs::msg::Imu>,
    power_pub: &Publisher<servo_robot_board_interface::msg::BoardPower>,
    thermal_pub: &Publisher<servo_robot_board_interface::msg::BoardThermal>,
    system_pub: &Publisher<servo_robot_board_interface::msg::BoardSystem>,
    event_pub: &Publisher<servo_robot_board_interface::msg::BoardEvent>,
    config_pub: &Publisher<servo_robot_board_interface::msg::BoardConfig>,
    battery_pub: &Publisher<sensor_msgs::msg::BatteryState>,
) {
    let snap = state.snapshot();

    if let Some(imu) = &snap.imu {
        if let Err(e) = imu_pub.publish(conversion::convert_imu(imu)) {
            log_error!(logger, "Publish IMU failed: {}", e);
        } else {
            log_debug!(logger.throttle(Duration::from_secs(1)), "IMU: {}", imu);
        }
    }
    if let Some(power) = &snap.power {
        if let Err(e) = power_pub.publish(conversion::convert_power(power)) {
            log_error!(logger, "Publish Power failed: {}", e);
        } else {
            log_debug!(logger.throttle(Duration::from_secs(1)), "Power: {}", power);
        }
    }
    if let Some(battery) = &snap.battery {
        if let Err(e) = battery_pub.publish(conversion::convert_battery(battery)) {
            log_error!(logger, "Publish Battery failed: {}", e);
        } else {
            log_debug!(logger.throttle(Duration::from_secs(1)), "Battery: {}", battery);
        }
    }
    if let Some(thermal) = &snap.thermal {
        if let Err(e) = thermal_pub.publish(conversion::convert_thermal(thermal)) {
            log_error!(logger, "Publish Thermal failed: {}", e);
        } else {
            log_debug!(logger.throttle(Duration::from_secs(1)), "Thermal: {}", thermal);
        }
    }
    if let Some(system) = &snap.system {
        if let Err(e) = system_pub.publish(conversion::convert_system(system)) {
            log_error!(logger, "Publish System failed: {}", e);
        } else {
            log_debug!(logger, "System: {}", system);
        }
    }
    if let Some(event) = &snap.event {
        if let Err(e) = event_pub.publish(conversion::convert_event(event)) {
            log_error!(logger, "Publish Event failed: {}", e);
        } else {
            log_debug!(logger, "Event: {}", event);
        }
    }
    if let Some(config) = &snap.config {
        if let Err(e) = config_pub.publish(conversion::convert_config(config)) {
            log_error!(logger, "Publish Config failed: {}", e);
        } else {
            log_debug!(logger, "Config: {}", config);
        }
    }
}

fn run_bridge(
    mut executor: Executor,
    node: Arc<NodeState>,
    driver: Driver,
) -> Result<(), Box<dyn std::error::Error>> {
    let state = driver.state();
    let driver = Arc::new(Mutex::new(driver));
    let logger = node.logger().clone();

    // 创建发布者
    let imu_pub = node.create_publisher::<sensor_msgs::msg::Imu>("/robot/board/imu")?;
    let power_pub = node
        .create_publisher::<servo_robot_board_interface::msg::BoardPower>("/robot/board/power")?;
    let thermal_pub = node.create_publisher::<servo_robot_board_interface::msg::BoardThermal>(
        "/robot/board/thermal",
    )?;
    let system_pub = node
        .create_publisher::<servo_robot_board_interface::msg::BoardSystem>("/robot/board/system")?;
    let event_pub = node
        .create_publisher::<servo_robot_board_interface::msg::BoardEvent>("/robot/board/event")?;
    let config_pub = node
        .create_publisher::<servo_robot_board_interface::msg::BoardConfig>("/robot/board/config")?;
    let battery_pub =
        node.create_publisher::<sensor_msgs::msg::BatteryState>("/robot/board/battery")?;

    log_info!(&logger, "Publishers created");

    // 创建服务
    let driver_clone = driver.clone();
    let logger_clone = logger.clone();
    let _query_srv = node.create_service::<servo_robot_board_interface::srv::BoardQueryConfig, _>(
        "/robot/board/query_config",
        move |req: servo_robot_board_interface::srv::BoardQueryConfig_Request| {
            log_info!(&logger_clone, "Service query_config: type=0x{:02X}", req.config_type);
            let resp = services::handle_query_config(&driver_clone, req);
            if resp.success {
                log_info!(&logger_clone, "Service query_config: ok, value={:.2}", resp.value);
            } else {
                log_error!(&logger_clone, "Service query_config: failed, {}", resp.msg);
            }
            resp
        },
    )?;

    let driver_clone = driver.clone();
    let logger_clone = logger.clone();
    let _query_all_srv = node
        .create_service::<servo_robot_board_interface::srv::BoardQueryAllConfig, _>(
            "/robot/board/query_all_config",
            move |_req: servo_robot_board_interface::srv::BoardQueryAllConfig_Request| {
                log_info!(&logger_clone, "Service query_all_config");
                let resp = services::handle_query_all_config(&driver_clone);
                if resp.success {
                    log_info!(&logger_clone, "Service query_all_config: ok");
                } else {
                    log_error!(&logger_clone, "Service query_all_config: failed, {}", resp.msg);
                }
                resp
            },
        )?;

    let driver_clone = driver.clone();
    let logger_clone = logger.clone();
    let _write_srv = node.create_service::<servo_robot_board_interface::srv::BoardWriteConfig, _>(
        "/robot/board/write_config",
        move |req: servo_robot_board_interface::srv::BoardWriteConfig_Request| {
            log_info!(&logger_clone, "Service write_config: type=0x{:02X}, value={:.2}",
                req.config_type, req.value);
            let resp = services::handle_write_config(&driver_clone, req);
            if resp.success {
                log_info!(&logger_clone, "Service write_config: ok");
            } else {
                log_error!(&logger_clone, "Service write_config: failed, {}", resp.msg);
            }
            resp
        },
    )?;

    let driver_clone = driver.clone();
    let logger_clone = logger.clone();
    let _switch_srv = node.create_service::<servo_robot_board_interface::srv::BoardSwitch, _>(
        "/robot/board/switch",
        move |req: servo_robot_board_interface::srv::BoardSwitch_Request| {
            log_info!(&logger_clone, "Service switch: type=0x{:02X}, enable={}",
                req.switch_type, req.enable);
            let resp = services::handle_switch(&driver_clone, req);
            if resp.success {
                log_info!(&logger_clone, "Service switch: ok");
            } else {
                log_error!(&logger_clone, "Service switch: failed, {}", resp.msg);
            }
            resp
        },
    )?;

    log_info!(&logger, "Services created, bridge ready!");

    // 主循环：轮询状态 + 发布数据 + 处理服务回调
    loop {
        publish_data(
            &state,
            &logger,
            &imu_pub,
            &power_pub,
            &thermal_pub,
            &system_pub,
            &event_pub,
            &config_pub,
            &battery_pub,
        );

        executor.spin(SpinOptions::spin_once());
        std::thread::sleep(Duration::from_millis(50));
    }
}
