//! # Authors
//! greenhand520
//! # Since
//! version: 0.1.0
//! # Date
//! 2026/7/4 10:18

//! ROS2 bridge node

mod conversion;
mod services;

use std::sync::{Arc, Mutex};
use std::time::Duration;

use rclrs::*;
use servo_robot_board_interface::msg::*;
use servo_robot_board_interface::srv::*;
use servo_robot_driver::protocol::log::{LogLevel, LogMessage};
use servo_robot_driver::protocol::servo::ServoCmdWrapper;
use servo_robot_driver::{Driver, DriverCallback};

#[cfg(feature = "mock")]
use servo_robot_driver::MockTransport;

#[cfg(not(feature = "mock"))]
use servo_robot_driver::SerialTransport;

/// ROS2 bridges callbacks to forward board-level logs to the ROS2 log system + /robot/board/log topic
struct BridgeCallback {
    logger: Logger,
    log_pub: Arc<Publisher<rcl_interfaces::msg::Log>>,
}

impl DriverCallback for BridgeCallback {
    fn on_log(&mut self, ts: u64, log_msg: &LogMessage) {
        // 格式化日志消息
        let total_s = ts / 1000;
        let ms = ts % 1000;
        let h = (total_s % 86400) / 3600;
        let m = (total_s % 3600) / 60;
        let s = total_s % 60;
        let formatted = format!(
            "[{:02}:{:02}:{:02}.{:03}] {}::{}: {}",
            h, m, s, ms, log_msg.file_name, log_msg.fun_name, log_msg.msg
        );

        // Publish to /robot/board/log
        let ros_level = match log_msg.level {
            LogLevel::Error => rcl_interfaces::msg::Log::ERROR,
            LogLevel::Warn => rcl_interfaces::msg::Log::WARN,
            LogLevel::Info | LogLevel::OFF => rcl_interfaces::msg::Log::INFO,
            LogLevel::Debug => rcl_interfaces::msg::Log::DEBUG,
        };
        let log_msg_ros = rcl_interfaces::msg::Log {
            stamp: builtin_interfaces::msg::Time {
                sec: total_s as i32,
                nanosec: (ms * 1_000_000) as u32,
            },
            level: ros_level,
            name: "servo_robot_board".to_string(),
            msg: formatted.clone(),
            file: log_msg.file_name.clone(),
            function: log_msg.fun_name.clone(),
            line: 0,
        };
        if let Err(e) = self.log_pub.publish(log_msg_ros) {
            log_error!(&self.logger, "Publish board log failed: {}", e);
        }

        // 同时输出到 ROS2 日志系统
        match log_msg.level {
            LogLevel::Error => log_error!(&self.logger, "[servo_robot_board] {}", formatted),
            LogLevel::Warn => log_warn!(&self.logger, "[servo_robot_board] {}", formatted),
            LogLevel::Info | LogLevel::OFF => {
                log_info!(&self.logger, "[servo_robot_board] {}", formatted)
            }
            LogLevel::Debug => log_debug!(&self.logger, "[servo_robot_board] {}", formatted),
        }
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let context = Context::default();
    let executor = context.create_basic_executor();
    let node = executor.create_node("servo_robot_board_bridge")?;

    // Declare ROS2 parameters
    // can be done via --ros-args -p port:=... -p baud_rate:=... or YAML parameter file settings
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

    // Create board-level log publishers
    let log_pub = Arc::new(node.create_publisher::<rcl_interfaces::msg::Log>("/robot/board/log")?);

    #[cfg(feature = "mock")]
    {
        let mut mock = MockTransport::new();
        mock.set_charging(true);
        mock.set_battery_soc(0.75);
        log_info!(node.logger(), "Starting with MockTransport");
        let mut driver = Driver::new(mock);
        driver.register_callback(BridgeCallback {
            logger: node.logger().clone(),
            log_pub: Arc::clone(&log_pub),
        });
        driver.start()?;
        run_bridge(executor, node, driver)?;
    }

    #[cfg(not(feature = "mock"))]
    {
        let port: Arc<str> = param_port.get();
        let baud_rate = param_baud_rate.get() as u32;
        log_info!(
            node.logger(),
            "Connecting to {} at {} baud...",
            port,
            baud_rate
        );
        let transport = SerialTransport::open(&port, baud_rate)?;
        let mut driver = Driver::new(transport);
        driver.register_callback(BridgeCallback {
            logger: node.logger().clone(),
            log_pub: Arc::clone(&log_pub),
        });
        driver.start()?;
        run_bridge(executor, node, driver)?;
    }

    Ok(())
}

/// Publish all data topics
fn publish_data(
    state: &servo_robot_driver::DriverState,
    logger: &Logger,
    imu_pub: &Publisher<sensor_msgs::msg::Imu>,
    power_pub: &Publisher<BoardPower>,
    system_pub: &Publisher<BoardSystem>,
    event_pub: &Publisher<BoardEvent>,
    config_pub: &Publisher<BoardConfig>,
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
            log_debug!(
                logger.throttle(Duration::from_secs(1)),
                "Battery: {}",
                battery
            );
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

    let imu_pub = node.create_publisher::<sensor_msgs::msg::Imu>("/robot/board/imu")?;
    let power_pub = node
        .create_publisher::<BoardPower>("/robot/board/power")?;
    let system_pub = node
        .create_publisher::<BoardSystem>("/robot/board/system")?;
    let event_pub = node
        .create_publisher::<BoardEvent>("/robot/board/event")?;
    let config_pub = node
        .create_publisher::<BoardConfig>("/robot/board/config")?;
    let battery_pub =
        node.create_publisher::<sensor_msgs::msg::BatteryState>("/robot/board/battery")?;

    log_info!(&logger, "Publishers created");

    let driver_clone = driver.clone();
    let logger_clone = logger.clone();
    let _query_srv = node.create_service::<BoardQueryConfig, _>(
        "/robot/board/query_config",
        move |req: BoardQueryConfig_Request| {
            log_info!(
                &logger_clone,
                "Service query_config: type=0x{:02X}",
                req.config_type
            );
            let resp = services::handle_query_config(&driver_clone, req);
            if resp.success {
                log_info!(
                    &logger_clone,
                    "Service query_config: ok, value={:.2}",
                    resp.value
                );
            } else {
                log_error!(&logger_clone, "Service query_config: failed, {}", resp.msg);
            }
            resp
        },
    )?;

    let driver_clone = driver.clone();
    let logger_clone = logger.clone();
    let _query_all_srv = node
        .create_service::<BoardQueryAllConfig, _>(
            "/robot/board/query_all_config",
            move |_req: BoardQueryAllConfig_Request| {
                log_info!(&logger_clone, "Service query_all_config");
                let resp = services::handle_query_all_config(&driver_clone);
                if resp.success {
                    log_info!(&logger_clone, "Service query_all_config: ok");
                } else {
                    log_error!(
                        &logger_clone,
                        "Service query_all_config: failed, {}",
                        resp.msg
                    );
                }
                resp
            },
        )?;

    let driver_clone = driver.clone();
    let logger_clone = logger.clone();
    let _write_srv = node.create_service::<BoardWriteConfig, _>(
        "/robot/board/write_config",
        move |req: BoardWriteConfig_Request| {
            log_info!(
                &logger_clone,
                "Service write_config: type=0x{:02X}, value={:.2}",
                req.config_type,
                req.value
            );
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
    let _switch_srv = node.create_service::<BoardSwitch, _>(
        "/robot/board/switch",
        move |req: BoardSwitch_Request| {
            log_info!(
                &logger_clone,
                "Service switch: type=0x{:02X}, enable={}",
                req.switch_type,
                req.enable
            );
            let resp = services::handle_switch(&driver_clone, req);
            if resp.success {
                log_info!(&logger_clone, "Service switch: ok");
            } else {
                log_error!(&logger_clone, "Service switch: failed, {}", resp.msg);
            }
            resp
        },
    )?;

    let driver_clone = driver.clone();
    let logger_clone = logger.clone();
    let _servo_forward_srv = node.create_service::<ServoForward, _>(
        "/robot/board/servo/forward",
        move |req: ServoForward_Request| {
            log_info!(
                &logger_clone,
                "Service servo_forward: {} bytes",
                req.command.len()
            );
            let resp = services::handle_servo_forward(&driver_clone, req);
            if resp.success {
                log_info!(
                    &logger_clone,
                    "Service servo_forward: ok, response {} bytes",
                    resp.response.len()
                );
            } else {
                log_error!(&logger_clone, "Service servo_forward: failed, {}", resp.msg);
            }
            resp
        },
    )?;

    let driver_clone = driver.clone();
    let logger_clone = logger.clone();
    let _servo_target_sub = node.create_subscription::<ServoTarget, _>(
        "/robot/board/servo/target",
        move |msg: ServoTarget| {
            log_info!(
                &logger_clone,
                "Subscription servo_target: {} bytes",
                msg.data.len()
            );
            let cmd = ServoCmdWrapper::new(msg.data.clone());
            if let Err(e) = driver_clone.lock().unwrap().forward_servo(&cmd) {
                log_error!(&logger_clone, "Forward servo command failed: {}", e);
            }
        },
    )?;

    log_info!(&logger, "Services created, bridge ready!");

    // Main loop: Polling state + publishing data + processing service callbacks
    loop {
        publish_data(
            &state,
            &logger,
            &imu_pub,
            &power_pub,
            &system_pub,
            &event_pub,
            &config_pub,
            &battery_pub,
        );

        // spin_once timeouts to prevent blockages that could prevent publish_data from executing
        executor.spin(SpinOptions::spin_once().timeout(Duration::from_millis(100)));
        std::thread::sleep(Duration::from_millis(40));
    }
}
