//! Driver 主结构体

use crate::dispatch::callback::DriverCallback;
use crate::dispatch::{DriverEvent, EventBus};
use crate::error::DriverError;
use crate::protocol::battery_state::BatteryState;
use crate::protocol::config::{BoardConfigSnapshot, Config, ConfigType};
use crate::protocol::event::BoardEvent;
use crate::protocol::frame::{FrameType, RawFrame, TypedFrame};
use crate::protocol::imu::ImuData;
use crate::protocol::power::PowerData;
use crate::protocol::thermal::ThermalData;
use crate::protocol::system::SystemInfo;
use crate::reconnect::ReconnectConfig;
use crate::state::DriverState;
use crate::transport::{Transport, TransportFactory};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use std::thread::JoinHandle;
use std::time::Duration;

/// 默认超时时间
const DEFAULT_TIMEOUT: Duration = Duration::from_millis(1000);

/// 主驱动结构体
pub struct Driver {
    /// 传输层
    transport: Arc<Mutex<Option<Box<dyn Transport>>>>,
    /// 传输层工厂（用于重连）
    transport_factory: Option<Arc<dyn TransportFactory>>,
    /// 重连配置
    reconnect_config: Option<ReconnectConfig>,
    /// 事件总线
    bus: Arc<EventBus>,
    /// 状态快照
    state: Arc<DriverState>,
    /// 读取线程句柄
    read_handle: Option<JoinHandle<()>>,
    /// 运行标志
    running: Arc<AtomicBool>,
}

impl Driver {
    /// 创建新驱动实例（不支持自动重连）
    pub fn new(transport: impl Transport) -> Self {
        Driver {
            transport: Arc::new(Mutex::new(Some(Box::new(transport)))),
            transport_factory: None,
            reconnect_config: None,
            bus: Arc::new(EventBus::new()),
            state: Arc::new(DriverState::new()),
            read_handle: None,
            running: Arc::new(AtomicBool::new(false)),
        }
    }

    /// 创建支持自动重连的驱动实例
    ///
    /// # Arguments
    /// * `factory` - 传输层工厂，每次重连时调用
    /// * `reconnect_config` - 重连配置
    ///
    /// # Example
    /// ```rust,no_run
    /// use servo_robot_driver::{Driver, SerialTransport, FnTransportFactory};
    /// use servo_robot_driver::reconnect::ReconnectConfig;
    ///
    /// let factory = FnTransportFactory::new(|| {
    ///     SerialTransport::open("/dev/ttyUSB0", 115200).map(|t| Box::new(t) as _)
    /// });
    /// let config = ReconnectConfig::new(5);
    /// let driver = Driver::new_with_reconnect(factory, config);
    /// ```
    pub fn new_with_reconnect(
        factory: impl TransportFactory,
        reconnect_config: ReconnectConfig,
    ) -> Self {
        // 创建初始连接
        let initial_transport = factory.create();

        Driver {
            transport: Arc::new(Mutex::new(initial_transport.ok())),
            transport_factory: Some(Arc::new(factory)),
            reconnect_config: Some(reconnect_config),
            bus: Arc::new(EventBus::new()),
            state: Arc::new(DriverState::new()),
            read_handle: None,
            running: Arc::new(AtomicBool::new(false)),
        }
    }

    /// 注册 trait 回调（Pattern A）
    pub fn register_callback(&self, cb: impl DriverCallback) {
        self.bus.register_callback(cb);
    }

    /// 注册闭包回调（Pattern B）
    pub fn on_imu_data(&self, f: impl FnMut(&ImuData) + Send + 'static) {
        self.bus.on_imu_data(f);
    }

    pub fn on_power_data(&self, f: impl FnMut(&PowerData) + Send + 'static) {
        self.bus.on_power_data(f);
    }

    pub fn on_thermal_data(&self, f: impl FnMut(&ThermalData) + Send + 'static) {
        self.bus.on_thermal_data(f);
    }

    pub fn on_battery_state(&self, f: impl FnMut(&BatteryState) + Send + 'static) {
        self.bus.on_battery_state(f);
    }

    pub fn on_config_snapshot(&self, f: impl FnMut(&BoardConfigSnapshot) + Send + 'static) {
        self.bus.on_config_snapshot(f);
    }

    pub fn on_board_event(&self, f: impl FnMut(&BoardEvent) + Send + 'static) {
        self.bus.on_board_event(f);
    }

    pub fn on_system_info(&self, f: impl FnMut(&SystemInfo) + Send + 'static) {
        self.bus.on_system_info(f);
    }

    pub fn on_error(&self, f: impl FnMut(&DriverError) + Send + 'static) {
        self.bus.on_error(f);
    }

    /// 启动驱动（开启读取线程）
    pub fn start(&mut self) -> Result<(), DriverError> {
        if self.running.load(Ordering::Relaxed) {
            return Err(DriverError::NotRunning);
        }

        // 检查是否有可用的传输层
        {
            let transport = self.transport.lock().map_err(|_| DriverError::LockPoisoned)?;
            if transport.is_none() && self.transport_factory.is_none() {
                return Err(DriverError::TransportClosed);
            }
        }

        self.running.store(true, Ordering::Relaxed);
        self.state.set_connected(true);

        let transport = Arc::clone(&self.transport);
        let transport_factory = self.transport_factory.clone();
        let reconnect_config = self.reconnect_config.clone();
        let bus = Arc::clone(&self.bus);
        let state = Arc::clone(&self.state);
        let running = Arc::clone(&self.running);

        let handle = std::thread::spawn(move || {
            Self::read_loop(transport, transport_factory, bus, state, running, reconnect_config);
        });

        self.read_handle = Some(handle);
        Ok(())
    }

    /// 停止驱动
    pub fn stop(&mut self) -> Result<(), DriverError> {
        self.running.store(false, Ordering::Relaxed);

        if let Some(handle) = self.read_handle.take() {
            handle.join().map_err(|_| DriverError::LockPoisoned)?;
        }

        self.state.set_connected(false);
        Ok(())
    }

    // ═══ 异步发送（不等待应答）═══

    /// 发送配置/命令到 STM32（不等待应答）
    pub fn send_config(&self, config: Config) -> Result<(), DriverError> {
        let frame = RawFrame {
            frame_type: FrameType::CfgWrite,
            payload: config.to_bytes(),
        };
        let encoded = frame.encode();

        let mut transport = self.transport.lock().map_err(|_| DriverError::LockPoisoned)?;
        match transport.as_mut() {
            Some(t) => t.write_frame(&encoded)?,
            None => return Err(DriverError::TransportClosed),
        }
        Ok(())
    }

    /// 查询单个配置（不等待应答）
    pub fn query_config(&self, config_type: ConfigType) -> Result<(), DriverError> {
        let frame = RawFrame {
            frame_type: FrameType::CfgQuery,
            payload: vec![config_type as u8],
        };
        let encoded = frame.encode();

        let mut transport = self.transport.lock().map_err(|_| DriverError::LockPoisoned)?;
        match transport.as_mut() {
            Some(t) => t.write_frame(&encoded)?,
            None => return Err(DriverError::TransportClosed),
        }
        Ok(())
    }

    /// 查询所有配置（不等待应答）
    pub fn query_all_configs(&self) -> Result<(), DriverError> {
        let frame = RawFrame {
            frame_type: FrameType::CfgQueryAll,
            payload: vec![],
        };
        let encoded = frame.encode();

        let mut transport = self.transport.lock().map_err(|_| DriverError::LockPoisoned)?;
        match transport.as_mut() {
            Some(t) => t.write_frame(&encoded)?,
            None => return Err(DriverError::TransportClosed),
        }
        Ok(())
    }

    /// 写入配置（不等待应答）
    pub fn write_config(&self, config: Config) -> Result<(), DriverError> {
        let frame = RawFrame {
            frame_type: FrameType::CfgWrite,
            payload: config.to_bytes(),
        };
        let encoded = frame.encode();

        let mut transport = self.transport.lock().map_err(|_| DriverError::LockPoisoned)?;
        match transport.as_mut() {
            Some(t) => t.write_frame(&encoded)?,
            None => return Err(DriverError::TransportClosed),
        }
        Ok(())
    }

    // ═══ 同步发送（等待应答）═══

    /// 发送命令并等待确认

    /// 查询单个配置并等待响应
    pub fn query_config_sync(&self, config_type: ConfigType) -> Result<Config, DriverError> {
        self.query_config(config_type)?;
        self.wait_for_ack_cfg_query(DEFAULT_TIMEOUT)
    }

    /// 查询所有配置并等待响应
    pub fn query_all_configs_sync(&self) -> Result<BoardConfigSnapshot, DriverError> {
        self.query_all_configs()?;
        self.wait_for_ack_cfg_query_all(DEFAULT_TIMEOUT)
    }

    /// 写入配置并等待确认
    pub fn write_config_sync(&self, config: Config) -> Result<bool, DriverError> {
        self.write_config(config)?;
        self.wait_for_ack_cfg_write(DEFAULT_TIMEOUT)
    }

    // ═══ 等待应答 ═══

    /// 等待命令确认
    fn wait_for_ack_cmd(&self, timeout: Duration) -> Result<bool, DriverError> {
        let deadline = std::time::Instant::now() + timeout;

        while std::time::Instant::now() < deadline {
            match self.bus.try_recv_ack()? {
                Some(event) => {
                    if let DriverEvent::AckCfgWrite { success } = event {
                        return Ok(success);
                    }
                }
                None => {
                    std::thread::sleep(Duration::from_millis(10));
                }
            }
        }

        Err(DriverError::Timeout)
    }

    /// 等待配置查询响应
    fn wait_for_ack_cfg_query(&self, timeout: Duration) -> Result<Config, DriverError> {
        let deadline = std::time::Instant::now() + timeout;

        while std::time::Instant::now() < deadline {
            match self.bus.try_recv_ack()? {
                Some(event) => {
                    if let DriverEvent::AckCfgQuery(config) = event {
                        return Ok(config);
                    }
                }
                None => {
                    std::thread::sleep(Duration::from_millis(10));
                }
            }
        }

        Err(DriverError::Timeout)
    }

    /// 等待所有配置查询响应
    fn wait_for_ack_cfg_query_all(
        &self,
        timeout: Duration,
    ) -> Result<BoardConfigSnapshot, DriverError> {
        let deadline = std::time::Instant::now() + timeout;

        while std::time::Instant::now() < deadline {
            match self.bus.try_recv_ack()? {
                Some(event) => {
                    if let DriverEvent::AckCfgQueryAll(config) = event {
                        return Ok(config);
                    }
                }
                None => {
                    std::thread::sleep(Duration::from_millis(10));
                }
            }
        }

        Err(DriverError::Timeout)
    }

    /// 等待配置写入确认
    fn wait_for_ack_cfg_write(&self, timeout: Duration) -> Result<bool, DriverError> {
        let deadline = std::time::Instant::now() + timeout;

        while std::time::Instant::now() < deadline {
            match self.bus.try_recv_ack()? {
                Some(event) => {
                    if let DriverEvent::AckCfgWrite { success } = event {
                        return Ok(success);
                    }
                }
                None => {
                    std::thread::sleep(Duration::from_millis(10));
                }
            }
        }

        Err(DriverError::Timeout)
    }

    /// 获取状态快照（给 TUI 用）
    pub fn state(&self) -> Arc<DriverState> {
        Arc::clone(&self.state)
    }

    /// 内部读取循环
    fn read_loop(
        transport: Arc<Mutex<Option<Box<dyn Transport>>>>,
        transport_factory: Option<Arc<dyn TransportFactory>>,
        bus: Arc<EventBus>,
        state: Arc<DriverState>,
        running: Arc<AtomicBool>,
        reconnect_config: Option<ReconnectConfig>,
    ) {
        let mut retry_count = 0;

        while running.load(Ordering::Relaxed) {
            // 读取一帧
            let frame_data = {
                let mut transport_guard = match transport.lock() {
                    Ok(t) => t,
                    Err(_) => {
                        state.set_error(DriverError::LockPoisoned);
                        break;
                    }
                };

                let current_transport = match transport_guard.as_mut() {
                    Some(t) => t,
                    None => {
                        // 传输层不可用，尝试重连
                        drop(transport_guard);
                        if let Some(ref factory) = transport_factory {
                            Self::attempt_reconnect(
                                &transport,
                                factory.as_ref(),
                                &state,
                                &bus,
                                reconnect_config.as_ref(),
                                &mut retry_count,
                            );
                        } else {
                            state.set_error(DriverError::TransportClosed);
                            bus.sender()
                                .send(DriverEvent::Error(DriverError::TransportClosed))
                                .ok();
                            break;
                        }
                        continue;
                    }
                };

                match current_transport.read_frame() {
                    Ok(data) => {
                        retry_count = 0; // 成功读取，重置重试计数
                        data
                    }
                    Err(DriverError::Io(ref e)) if e.contains("timed out") => {
                        continue;
                    }
                    Err(DriverError::TransportClosed) => {
                        // 连接断开
                        log::warn!("Connection lost");
                        state.set_connected(false);
                        *transport_guard = None;

                        // 尝试重连
                        if let Some(ref factory) = transport_factory {
                            drop(transport_guard);
                            Self::attempt_reconnect(
                                &transport,
                                factory.as_ref(),
                                &state,
                                &bus,
                                reconnect_config.as_ref(),
                                &mut retry_count,
                            );
                        } else {
                            bus.sender()
                                .send(DriverEvent::Error(DriverError::TransportClosed))
                                .ok();
                            break;
                        }
                        continue;
                    }
                    Err(e) => {
                        state.set_error(e.clone());
                        bus.sender().send(DriverEvent::Error(e)).ok();
                        continue;
                    }
                }
            };

            // 解码帧
            let raw_frame = match RawFrame::decode(&frame_data) {
                Ok((frame, _)) => frame,
                Err(e) => {
                    log::warn!("Frame decode error: {}", e);
                    state.increment_frames_dropped();
                    continue;
                }
            };

            // 解析为类型化帧
            let typed_frame = match raw_frame.parse_typed() {
                Ok(frame) => frame,
                Err(e) => {
                    log::warn!("Frame parse error: {}", e);
                    state.increment_frames_dropped();
                    continue;
                }
            };

            // 成功解析，计数+1
            state.increment_frames_parsed();

            // 分发事件
            let event = match typed_frame {
                // 上行数据
                TypedFrame::Imu(data) => {
                    state.update_imu(data.clone());
                    DriverEvent::ImuData(data)
                }
                TypedFrame::Power(data) => {
                    state.update_power(data.clone());
                    DriverEvent::PowerData(data)
                }
                TypedFrame::Thermal(data) => {
                    state.update_thermal(data.clone());
                    DriverEvent::ThermalData(data)
                }
                TypedFrame::Battery(bat) => {
                    state.update_battery(bat.clone());
                    DriverEvent::BatteryState(bat)
                }
                TypedFrame::Config(config) => {
                    state.update_config(config.clone());
                    DriverEvent::ConfigSnapshot(config)
                }
                TypedFrame::Event(event) => {
                    state.update_event(event.clone());
                    DriverEvent::BoardEvent(event)
                }
                TypedFrame::System(info) => {
                    state.update_system(info.clone());
                    DriverEvent::SystemInfo(info)
                }

                // 应答
                TypedFrame::AckCfgWrite { success } => {
                    DriverEvent::AckCfgWrite { success }
                }
                TypedFrame::AckCfgQuery(config) => {
                    DriverEvent::AckCfgQuery(config)
                }
                TypedFrame::AckCfgQueryAll(config_snapshot) => {
                    state.update_config(config_snapshot.clone());
                    DriverEvent::AckCfgQueryAll(config_snapshot)
                }
                // 下行帧不应该被接收
                _ => continue,
            };

            // 检查是否是 ACK 事件
            let is_ack = matches!(
                event,
                DriverEvent::AckCfgQuery(_)
                    | DriverEvent::AckCfgQueryAll(_)
                    | DriverEvent::AckCfgWrite { .. }
            );

            // 通过事件总线分发
            bus.sender().send(event.clone()).ok();

            // ACK 事件同时发送到 ACK 通道（供同步等待使用）
            if is_ack {
                bus.ack_sender().send(event).ok();
            }

            // 立即处理事件（触发回调）
            if let Err(e) = bus.try_recv_and_dispatch() {
                log::warn!("Event dispatch error: {}", e);
            }
        }

        log::info!("Read loop exited");
    }

    /// 尝试重连
    fn attempt_reconnect(
        transport: &Arc<Mutex<Option<Box<dyn Transport>>>>,
        factory: &dyn TransportFactory,
        state: &Arc<DriverState>,
        bus: &Arc<EventBus>,
        config: Option<&ReconnectConfig>,
        retry_count: &mut u32,
    ) {
        let config = match config {
            Some(c) => c,
            None => return,
        };

        if *retry_count >= config.max_retries {
            log::error!("Max retries ({}) reached", config.max_retries);
            state.set_error(DriverError::TransportClosed);
            bus.sender()
                .send(DriverEvent::Error(DriverError::TransportClosed))
                .ok();
            return;
        }

        let delay = config.delay_for_retry(*retry_count);
        log::info!(
            "Reconnecting in {:?} (attempt {}/{})",
            delay,
            *retry_count + 1,
            config.max_retries
        );

        std::thread::sleep(delay);

        match factory.create() {
            Ok(new_transport) => {
                match transport.lock() {
                    Ok(mut guard) => {
                        *guard = Some(new_transport);
                        state.set_connected(true);
                        log::info!("Reconnected successfully");
                    }
                    Err(e) => {
                        log::error!("Failed to acquire transport lock: {}", e);
                    }
                }
            }
            Err(e) => {
                log::warn!("Reconnect failed: {}", e);
                *retry_count += 1;
            }
        }
    }
}

impl Drop for Driver {
    fn drop(&mut self) {
        let _ = self.stop();
    }
}
