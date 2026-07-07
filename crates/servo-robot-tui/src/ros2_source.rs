//! # Authors
//! greenhand520
//! # Since
//! version: 0.1.0
//! # Date
//! 2026/7/6 23:05

//! ROS2 数据源实现（channel 架构）
//!
//! ROS2 executor 在独立线程运行，通过 channel 将数据发送给主线程。
//! 服务调用通过 request/response channel 实现。

use rclrs::*;
use sensor_msgs::msg as sensor_msg;
use servo_robot_board_interface::msg as board_msg;
use servo_robot_board_interface::srv as board_srv;

use crate::data_source::{DataSnapshot, DataSource};
use servo_robot_driver::DriverError;
use servo_robot_driver::LogEntry;
use servo_robot_driver::protocol::battery_state::*;
use servo_robot_driver::protocol::config::*;
use servo_robot_driver::protocol::event::*;
use servo_robot_driver::protocol::imu::*;
use servo_robot_driver::protocol::log::LogLevel as ProtocolLogLevel;
use servo_robot_driver::protocol::log::LogMessage;
use servo_robot_driver::protocol::power::*;
use servo_robot_driver::protocol::system::*;
use servo_robot_driver::protocol::thermal::*;
use std::sync::Mutex;
use std::sync::mpsc;
use std::time::{Duration, Instant};

// ==================== 服务请求/响应类型 ====================

/// write_config 请求（通过 switch 或 write_config 服务）
enum WriteConfigRequest {
    /// 通用配置写入
    Write { config_type: u8, value: f32 },
    /// 开关控制
    Switch { switch_type: u8, enable: bool },
}

/// write_config 响应
struct WriteConfigResponse {
    success: bool,
    msg: String,
}

/// query_all_configs 响应
struct QueryAllConfigResponse {
    success: bool,
    msg: String,
    config: Option<BoardConfigSnapshot>,
}

// ==================== 数据事件 ====================

/// ROS2 线程发给主线程的消息
enum RosEvent {
    Imu(Box<ImuData>),
    Power(PowerData),
    Thermal(ThermalData),
    System(SystemInfo),
    Battery(Box<BatteryState>),
    BoardEvent(BoardEvent),
    Config(BoardConfigSnapshot),
    BoardLog(LogEntry),
}

// ==================== ROS2 线程 ====================

/// 解析版本字符串（格式：major.minor.patch）
fn parse_version(s: &str) -> Version {
    let parts: Vec<&str> = s.split('.').collect();
    let major = parts.first().and_then(|v| v.parse().ok()).unwrap_or(0);
    let minor = parts.get(1).and_then(|v| v.parse().ok()).unwrap_or(0);
    let patch = parts.get(2).and_then(|v| v.parse().ok()).unwrap_or(0);
    Version::new(major, minor, patch)
}

/// 在独立线程中运行 ROS2 executor，返回数据接收端
fn spawn_ros2_thread(
    write_resp_tx: mpsc::SyncSender<WriteConfigResponse>,
    query_resp_tx: mpsc::SyncSender<QueryAllConfigResponse>,
) -> Result<
    (
        mpsc::Receiver<RosEvent>,
        mpsc::SyncSender<WriteConfigRequest>,
        mpsc::SyncSender<()>,
    ),
    Box<dyn std::error::Error>,
> {
    let (tx, rx) = mpsc::channel();
    let (ready_tx, ready_rx) = mpsc::channel::<Result<(), String>>();

    // 服务调用通道（bounded，避免无限缓冲）
    let (write_req_tx, write_req_rx) = mpsc::sync_channel::<WriteConfigRequest>(4);
    let (query_req_tx, query_req_rx) = mpsc::sync_channel::<()>(4);

    std::thread::Builder::new()
        .name("ros2-spin".into())
        .spawn(move || {
            let result = (|| -> Result<(), Box<dyn std::error::Error>> {
                let context = Context::default();
                let mut executor = context.create_basic_executor();
                let node = executor.create_node("servo_robot_board_tui")?;
                let logger = node.logger().clone();
                let _ = logger.set_level(LogSeverity::Error);

                log::info!("[ROS2] Creating subscriptions...");

                // ==================== 订阅 ====================

                // IMU
                let tx_imu = tx.clone();
                let _imu_sub = node.create_subscription::<sensor_msg::Imu, _>(
                    "/robot/board/imu",
                    move |msg: sensor_msg::Imu| {
                        let data = ImuData {
                            accel: [msg.linear_acceleration.x as f32, msg.linear_acceleration.y as f32, msg.linear_acceleration.z as f32],
                            gyro: [msg.angular_velocity.x as f32, msg.angular_velocity.y as f32, msg.angular_velocity.z as f32],
                            quaternion: [msg.orientation.w as f32, msg.orientation.x as f32, msg.orientation.y as f32, msg.orientation.z as f32],
                            timestamp_ms: 0, roll: 0.0, pitch: 0.0, yaw: 0.0,
                        };
                        let _ = tx_imu.send(RosEvent::Imu(Box::new(data)));
                    },
                )?;

                // Power
                let tx_power = tx.clone();
                let _power_sub = node.create_subscription::<board_msg::BoardPower, _>(
                    "/robot/board/power",
                    move |msg: board_msg::BoardPower| {
                        let _ = tx_power.send(RosEvent::Power(PowerData {
                            servo_voltage: msg.servo_voltage, servo_current: msg.servo_current,
                            charge_in_voltage: msg.charge_in_voltage, charge_in_current: msg.charge_in_current,
                            bat_voltage: msg.bat_voltage, bat_current: msg.bat_current,
                        }));
                    },
                )?;

                // Thermal
                let tx_thermal = tx.clone();
                let _thermal_sub = node.create_subscription::<board_msg::BoardThermal, _>(
                    "/robot/board/thermal",
                    move |msg: board_msg::BoardThermal| {
                        let _ = tx_thermal.send(RosEvent::Thermal(ThermalData {
                            temp_servo_power: msg.temp_servo_power, temp_5v_power: msg.temp_5v_power,
                            temp_mcu: msg.temp_mcu, temp_charge: msg.temp_charge, temp_battery: msg.temp_battery,
                            reserved: 0.0,
                        }));
                    },
                )?;

                // System
                let tx_system = tx.clone();
                let _system_sub = node.create_subscription::<board_msg::BoardSystem, _>(
                    "/robot/board/system",
                    move |msg: board_msg::BoardSystem| {
                        let version = parse_version(&msg.firmware_version);
                        let _ = tx_system.send(RosEvent::System(SystemInfo {
                            device_id: msg.device_id, uid: msg.uid, imu_id: msg.imu_id,
                            uptime_s: msg.uptime_s, cpu_usage_percent: msg.cpu_usage_percent,
                            free_heap_kb: msg.free_heap_kb, stack_watermark_min_kb: msg.stack_watermark_min_kb,
                            i2c_error_count: msg.i2c_error_count, spi_error_count: msg.spi_error_count,
                            uart_error_count: msg.uart_error_count, usb_error_count: msg.usb_error_count,
                            frames_sent_total: msg.frames_sent_total,
                            pd_request_voltage: msg.pd_request_voltage, pd_request_current: msg.pd_request_current,
                            firmware_version: version,
                        }));
                    },
                )?;

                // Battery
                let tx_bat = tx.clone();
                let _battery_sub = node.create_subscription::<sensor_msg::BatteryState, _>(
                    "/robot/board/battery",
                    move |msg: sensor_msg::BatteryState| {
                        let _ = tx_bat.send(RosEvent::Battery(Box::new(BatteryState {
                            voltage: msg.voltage, current: msg.current,
                            capacity: msg.capacity, design_capacity: msg.design_capacity,
                            percentage: msg.percentage, temperature: msg.temperature,
                            charge_status: BatteryChargeStatus::from_u8(msg.power_supply_status),
                            health: BatteryHealth::from_u8(msg.power_supply_health),
                            technology: BatteryTechnology::from_u8(msg.power_supply_technology),
                            present: msg.present,
                            serial_number: msg.serial_number.parse().unwrap_or(0),
                            cell_voltages: msg.cell_voltage.iter().map(|v| v.clamp(0.0, 4.4)).collect(),
                            cell_temperatures: msg.cell_temperature.clone(),
                        })));
                    },
                )?;

                // Event
                let tx_event = tx.clone();
                let _event_sub = node.create_subscription::<board_msg::BoardEvent, _>(
                    "/robot/board/event",
                    move |msg: board_msg::BoardEvent| {
                        use servo_robot_driver::protocol::event::StateChangeFlags;
                        let state_change_flags = StateChangeFlags::empty();
                        let _ = tx_event.send(RosEvent::BoardEvent(BoardEvent {
                            charge_phase: ChargePhase::from_u8(msg.charge_phase),
                            state_change_flags,
                            protection_flags: ProtectionFlags::from_bits(msg.protection_flags)
                                .unwrap_or(ProtectionFlags::empty()),
                            error_flags: ErrorFlags::from_bits(msg.error_flags)
                                .unwrap_or(ErrorFlags::empty()),
                        }));
                    },
                )?;

                // Config
                let tx_cfg = tx.clone();
                let _config_sub = node.create_subscription::<board_msg::BoardConfig, _>(
                    "/robot/board/config",
                    move |msg: board_msg::BoardConfig| {
                        let _ = tx_cfg.send(RosEvent::Config(convert_board_config_to_snapshot(&msg)));
                    },
                )?;

                // Board Log（从 /robot/board/log 订阅）
                let tx_log = tx.clone();
                let _log_sub = node.create_subscription::<rcl_interfaces::msg::Log, _>(
                    "/robot/board/log",
                    move |msg: rcl_interfaces::msg::Log| {
                        // 只处理 servo_robot_board 的日志
                        if msg.name != "servo_robot_board" {
                            return;
                        }
                        let level = match msg.level {
                            rcl_interfaces::msg::Log::ERROR => ProtocolLogLevel::Error,
                            rcl_interfaces::msg::Log::WARN => ProtocolLogLevel::Warn,
                            rcl_interfaces::msg::Log::INFO => ProtocolLogLevel::Info,
                            rcl_interfaces::msg::Log::DEBUG => ProtocolLogLevel::Debug,
                            _ => ProtocolLogLevel::Info,
                        };
                        let ts = msg.stamp.sec as u64 * 1000 + msg.stamp.nanosec as u64 / 1_000_000;
                        let entry = LogEntry {
                            ts,
                            msg: LogMessage {
                                level,
                                file_name: msg.file,
                                fun_name: msg.function,
                                msg: msg.msg,
                            },
                        };
                        let _ = tx_log.send(RosEvent::BoardLog(entry));
                    },
                )?;

                log::info!("[ROS2] Creating service clients...");

                // ==================== 服务客户端 ====================

                let write_client = node.create_client::<board_srv::BoardWriteConfig>(
                    "/robot/board/write_config",
                )?;
                let switch_client = node.create_client::<board_srv::BoardSwitch>(
                    "/robot/board/switch",
                )?;
                let query_all_client = node.create_client::<board_srv::BoardQueryAllConfig>(
                    "/robot/board/query_all_config",
                )?;

                log::info!("[ROS2] All subscriptions and service clients created, starting spin...");

                // 通知主线程初始化完成
                let _ = ready_tx.send(Ok(()));

                // ==================== spin 循环 ====================
                // 每次 spin 后检查服务请求通道

                loop {
                    executor.spin(SpinOptions::new().timeout(Duration::from_millis(50)));

                    // 处理 write_config 请求
                    while let Ok(req) = write_req_rx.try_recv() {
                        match req {
                            WriteConfigRequest::Write { config_type, value } => {
                                let request = board_srv::BoardWriteConfig_Request { config_type, value };
                                let resp_tx = write_resp_tx.clone();
                                match write_client.call_then(request, move |resp: board_srv::BoardWriteConfig_Response| {
                                    let _ = resp_tx.send(WriteConfigResponse {
                                        success: resp.success,
                                        msg: resp.msg,
                                    });
                                }) {
                                    Ok(_) => {}
                                    Err(e) => {
                                        log::error!("[ROS2] write_config call failed: {}", e);
                                        let _ = write_resp_tx.send(WriteConfigResponse {
                                            success: false,
                                            msg: format!("ROS2 call failed: {}", e),
                                        });
                                    }
                                }
                            }
                            WriteConfigRequest::Switch { switch_type, enable } => {
                                let request = board_srv::BoardSwitch_Request { switch_type, enable };
                                let resp_tx = write_resp_tx.clone();
                                match switch_client.call_then(request, move |resp: board_srv::BoardSwitch_Response| {
                                    let _ = resp_tx.send(WriteConfigResponse {
                                        success: resp.success,
                                        msg: resp.msg,
                                    });
                                }) {
                                    Ok(_) => {}
                                    Err(e) => {
                                        log::error!("[ROS2] switch call failed: {}", e);
                                        let _ = write_resp_tx.send(WriteConfigResponse {
                                            success: false,
                                            msg: format!("ROS2 call failed: {}", e),
                                        });
                                    }
                                }
                            }
                        }
                    }

                    // 处理 query_all_configs 请求
                    while let Ok(()) = query_req_rx.try_recv() {
                        let request = board_srv::BoardQueryAllConfig_Request {
                            structure_needs_at_least_one_member: 0,
                        };
                        let resp_tx = query_resp_tx.clone();
                        match query_all_client.call_then(request, move |resp: board_srv::BoardQueryAllConfig_Response| {
                            let config_snapshot = if resp.success {
                                Some(convert_board_config_to_snapshot(&resp.config))
                            } else {
                                None
                            };
                            let _ = resp_tx.send(QueryAllConfigResponse {
                                success: resp.success,
                                msg: resp.msg,
                                config: config_snapshot,
                            });
                        }) {
                            Ok(_) => {}
                            Err(e) => {
                                log::error!("[ROS2] query_all_config call failed: {}", e);
                                let _ = query_resp_tx.send(QueryAllConfigResponse {
                                    success: false,
                                    msg: format!("ROS2 call failed: {}", e),
                                    config: None,
                                });
                            }
                        }
                    }
                }
            })();

            if let Err(e) = result {
                let _ = ready_tx.send(Err(format!("{}", e)));
            }
        })?;

    // 等待 spin 线程初始化完成
    match ready_rx.recv() {
        Ok(Ok(())) => {
            log::info!("[ROS2] ROS2 thread ready");
            Ok((rx, write_req_tx, query_req_tx))
        }
        Ok(Err(e)) => Err(e.into()),
        Err(e) => Err(format!("ROS2 thread init failed: {}", e).into()),
    }
}

/// 将 ROS2 BoardConfig 消息转换为协议 BoardConfigSnapshot
fn convert_board_config_to_snapshot(
    msg: &board_msg::BoardConfig,
) -> BoardConfigSnapshot {
    BoardConfigSnapshot {
        servo_current_limit: msg.servo_current_limit,
        servo_temp_limit: msg.servo_temp_limit,
        temp_5v_limit: msg.temp_5v_limit,
        charge_max_current: msg.charge_max_current,
        charge_temp_derating: msg.charge_temp_derating,
        charge_temp_limit: msg.charge_temp_limit,
        charge_stop_voltage: msg.charge_stop_voltage,
        charge_stop_percentage: msg.charge_stop_percentage,
        charge_enable: msg.charge_enable,
        power_servo_on: msg.power_servo_on,
        power_5v_on: msg.power_5v_on,
        charge_on: msg.charge_on,
        bat_ext_out_on: msg.bat_ext_out_on,
        tx_log_level: servo_robot_driver::protocol::log::LogLevel::from_u8(msg.tx_log_level),
    }
}

// ==================== ROS2 数据源 ====================

/// ROS2 数据源（channel 架构）
pub struct Ros2Source {
    rx: mpsc::Receiver<RosEvent>,
    snapshot: Mutex<DataSnapshot>,
    /// 服务请求通道
    write_req_tx: mpsc::SyncSender<WriteConfigRequest>,
    query_req_tx: mpsc::SyncSender<()>,
    /// 服务响应通道
    write_resp_rx: Mutex<mpsc::Receiver<WriteConfigResponse>>,
    query_resp_rx: Mutex<mpsc::Receiver<QueryAllConfigResponse>>,
}

impl Ros2Source {
    pub fn new() -> Result<Self, Box<dyn std::error::Error>> {
        // 响应通道（bounded=1，每次只处理一个请求）
        let (write_resp_tx, write_resp_rx) = mpsc::sync_channel(1);
        let (query_resp_tx, query_resp_rx) = mpsc::sync_channel(1);

        let (rx, write_req_tx, query_req_tx) = spawn_ros2_thread(write_resp_tx, query_resp_tx)?;

        Ok(Ros2Source {
            rx,
            snapshot: Mutex::new(DataSnapshot::default()),
            write_req_tx,
            query_req_tx,
            write_resp_rx: Mutex::new(write_resp_rx),
            query_resp_rx: Mutex::new(query_resp_rx),
        })
    }

    /// 将 TUI Config 转换为 ROS2 服务请求
    fn config_to_request(config: &Config) -> WriteConfigRequest {
        match config {
            Config::SwitchPowerServo(on) => WriteConfigRequest::Switch {
                switch_type: 0x10,
                enable: *on,
            },
            Config::SwitchPower5V(on) => WriteConfigRequest::Switch {
                switch_type: 0x11,
                enable: *on,
            },
            Config::SwitchCharge(on) => WriteConfigRequest::Switch {
                switch_type: 0x12,
                enable: *on,
            },
            Config::SwitchBatExtOut(on) => WriteConfigRequest::Switch {
                switch_type: 0x13,
                enable: *on,
            },
            _ => WriteConfigRequest::Write {
                config_type: config.config_type() as u8,
                value: config.value(),
            },
        }
    }
}

impl DataSource for Ros2Source {
    fn snapshot(&self) -> DataSnapshot {
        let mut snap = self.snapshot.lock().unwrap();
        // 非阻塞地接收所有待处理事件
        while let Ok(event) = self.rx.try_recv() {
            match event {
                RosEvent::Imu(d) => {
                    snap.imu = Some(*d);
                    snap.connected = true;
                }
                RosEvent::Power(d) => {
                    snap.power = Some(d);
                }
                RosEvent::Thermal(d) => {
                    snap.thermal = Some(d);
                }
                RosEvent::System(d) => {
                    snap.system = Some(d);
                }
                RosEvent::Battery(d) => {
                    snap.battery = Some(*d);
                }
                RosEvent::BoardEvent(d) => {
                    snap.event = Some(d);
                }
                RosEvent::Config(d) => {
                    snap.config = Some(d);
                }
                RosEvent::BoardLog(entry) => {
                    snap.logs.push(entry);
                    // 限制日志缓冲大小
                    if snap.logs.len() > 500 {
                        snap.logs.remove(0);
                    }
                }
            }
            snap.last_update = Instant::now();
        }
        snap.clone()
    }

    fn write_config(&self, config: Config) -> Result<(), DriverError> {
        log::info!("[ROS2] write_config: {:?}", config.config_type());

        let request = Self::config_to_request(&config);

        // 发送请求到 ROS2 线程
        self.write_req_tx
            .send(request)
            .map_err(|e| DriverError::Io(format!("channel send failed: {}", e)))?;

        // 等待响应（最多 3 秒）
        let resp = self
            .write_resp_rx
            .lock()
            .unwrap()
            .recv_timeout(Duration::from_secs(3))
            .map_err(|_| DriverError::Timeout)?;

        if resp.success {
            log::info!("[ROS2] write_config: ok");
            Ok(())
        } else {
            log::error!("[ROS2] write_config failed: {}", resp.msg);
            Err(DriverError::Io(resp.msg))
        }
    }

    fn query_all_configs(&self) -> Result<(), DriverError> {
        log::info!("[ROS2] query_all_configs");

        // 发送请求到 ROS2 线程
        self.query_req_tx
            .send(())
            .map_err(|e| DriverError::Io(format!("channel send failed: {}", e)))?;

        // 等待响应（最多 3 秒）
        let resp = self
            .query_resp_rx
            .lock()
            .unwrap()
            .recv_timeout(Duration::from_secs(3))
            .map_err(|_| DriverError::Timeout)?;

        if resp.success {
            log::info!("[ROS2] query_all_configs: ok");
            // 将配置更新到 snapshot
            if let Some(config) = resp.config {
                let mut snap = self.snapshot.lock().unwrap();
                snap.config = Some(config);
            }
            Ok(())
        } else {
            log::error!("[ROS2] query_all_configs failed: {}", resp.msg);
            Err(DriverError::Io(resp.msg))
        }
    }
}

/// 创建 ROS2 数据源
pub(crate) fn create_ros2_source()
-> Result<(Box<dyn DataSource>, crate::DataSourceMode), Box<dyn std::error::Error>> {
    let ros2_source = Ros2Source::new()?;
    Ok((Box::new(ros2_source), crate::DataSourceMode::Ros2))
}
