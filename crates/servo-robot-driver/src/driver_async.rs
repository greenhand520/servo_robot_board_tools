//! 异步 Driver 主结构体

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
use crate::transport::async_trait::{AsyncTransport, AsyncTransportFactory};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::Mutex;
use tokio::task::JoinHandle;

/// 默认超时时间
const DEFAULT_TIMEOUT: Duration = Duration::from_millis(1000);

/// 异步驱动结构体
#[cfg(feature = "async")]
pub struct AsyncDriver {
    /// 传输层
    transport: Arc<Mutex<Option<Box<dyn AsyncTransport>>>>,
    /// 传输层工厂（用于重连）
    transport_factory: Option<Arc<dyn AsyncTransportFactory>>,
    /// 重连配置
    reconnect_config: Option<ReconnectConfig>,
    /// 事件总线
    bus: Arc<EventBus>,
    /// 状态快照
    state: Arc<DriverState>,
    /// 读取任务句柄
    handle: Option<JoinHandle<()>>,
    /// 运行标志
    running: Arc<AtomicBool>,
}

#[cfg(feature = "async")]
impl AsyncDriver {
    /// 创建新的异步驱动实例
    pub fn new(transport: impl AsyncTransport) -> Self {
        AsyncDriver {
            transport: Arc::new(Mutex::new(Some(Box::new(transport)))),
            transport_factory: None,
            reconnect_config: None,
            bus: Arc::new(EventBus::new()),
            state: Arc::new(DriverState::new()),
            handle: None,
            running: Arc::new(AtomicBool::new(false)),
        }
    }

    /// 创建支持自动重连的异步驱动实例
    pub fn new_with_reconnect(
        factory: impl AsyncTransportFactory,
        reconnect_config: ReconnectConfig,
    ) -> Self {
        AsyncDriver {
            transport: Arc::new(Mutex::new(None)),
            transport_factory: Some(Arc::new(factory)),
            reconnect_config: Some(reconnect_config),
            bus: Arc::new(EventBus::new()),
            state: Arc::new(DriverState::new()),
            handle: None,
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

    /// 启动异步驱动
    pub async fn start(&mut self) -> Result<(), DriverError> {
        if self.running.load(Ordering::Relaxed) {
            return Err(DriverError::NotRunning);
        }

        // 如果有工厂，创建初始连接
        if let Some(ref factory) = self.transport_factory {
            let transport = factory.create().await?;
            let mut guard = self.transport.lock().await;
            *guard = Some(transport);
        }

        self.running.store(true, Ordering::Relaxed);
        self.state.set_connected(true);

        let transport = self.transport.clone();
        let transport_factory = self.transport_factory.clone();
        let reconnect_config = self.reconnect_config.clone();
        let bus = self.bus.clone();
        let state = self.state.clone();
        let running = self.running.clone();

        let handle = tokio::spawn(Self::read_loop(
            transport,
            transport_factory,
            bus,
            state,
            running,
            reconnect_config,
        ));

        self.handle = Some(handle);
        Ok(())
    }

    /// 停止异步驱动
    pub async fn stop(&mut self) -> Result<(), DriverError> {
        self.running.store(false, Ordering::Relaxed);

        if let Some(handle) = self.handle.take() {
            handle.await.map_err(|_| DriverError::LockPoisoned)?;
        }

        self.state.set_connected(false);
        Ok(())
    }

    // ═══ 异步发送（不等待应答）═══

    pub async fn send_config_cmd(&self, config: Config) -> Result<(), DriverError> {
        let frame = RawFrame {
            frame_type: FrameType::Cmd,
            payload: cmd.to_bytes(),
        };
        let encoded = frame.encode();

        let mut transport = self.transport.lock().await;
        match transport.as_mut() {
            Some(t) => t.write_frame(&encoded).await?,
            None => return Err(DriverError::TransportClosed),
        }
        Ok(())
    }

    pub async fn query_config(&self, config_type: ConfigType) -> Result<(), DriverError> {
        let frame = RawFrame {
            frame_type: FrameType::CfgQuery,
            payload: vec![config_type as u8],
        };
        let encoded = frame.encode();

        let mut transport = self.transport.lock().await;
        match transport.as_mut() {
            Some(t) => t.write_frame(&encoded).await?,
            None => return Err(DriverError::TransportClosed),
        }
        Ok(())
    }

    pub async fn query_all_configs(&self) -> Result<(), DriverError> {
        let frame = RawFrame {
            frame_type: FrameType::CfgQueryAll,
            payload: vec![],
        };
        let encoded = frame.encode();

        let mut transport = self.transport.lock().await;
        match transport.as_mut() {
            Some(t) => t.write_frame(&encoded).await?,
            None => return Err(DriverError::TransportClosed),
        }
        Ok(())
    }

    pub async fn write_config(&self, config: Config) -> Result<(), DriverError> {
        let frame = RawFrame {
            frame_type: FrameType::CfgWrite,
            payload: config.to_bytes(),
        };
        let encoded = frame.encode();

        let mut transport = self.transport.lock().await;
        match transport.as_mut() {
            Some(t) => t.write_frame(&encoded).await?,
            None => return Err(DriverError::TransportClosed),
        }
        Ok(())
    }

    // ═══ 同步发送（等待应答）═══


    pub async fn query_config_sync(&self, config_type: ConfigType) -> Result<Config, DriverError> {
        self.query_config(config_type).await?;
        self.wait_for_ack_cfg_query(DEFAULT_TIMEOUT).await
    }

    pub async fn query_all_configs_sync(&self) -> Result<BoardConfigSnapshot, DriverError> {
        self.query_all_configs().await?;
        self.wait_for_ack_cfg_query_all(DEFAULT_TIMEOUT).await
    }

    pub async fn write_config_sync(&self, config: Config) -> Result<bool, DriverError> {
        self.write_config(config).await?;
        self.wait_for_ack_cfg_write(DEFAULT_TIMEOUT).await
    }

    // ═══ 等待应答 ═══

    async fn wait_for_ack_cmd(&self, timeout: Duration) -> Result<bool, DriverError> {
        match tokio::time::timeout(timeout, self.bus.recv_ack_async()).await {
            Ok(Ok(event)) => {
                if let DriverEvent::AckCfgWrite { success } = event {
                    Ok(success)
                } else {
                    Err(DriverError::Timeout)
                }
            }
            Ok(Err(e)) => Err(e),
            Err(_) => Err(DriverError::Timeout),
        }
    }

    async fn wait_for_ack_cfg_query(&self, timeout: Duration) -> Result<Config, DriverError> {
        match tokio::time::timeout(timeout, self.bus.recv_ack_async()).await {
            Ok(Ok(event)) => {
                if let DriverEvent::AckCfgQuery(config) = event {
                    Ok(config)
                } else {
                    Err(DriverError::Timeout)
                }
            }
            Ok(Err(e)) => Err(e),
            Err(_) => Err(DriverError::Timeout),
        }
    }

    async fn wait_for_ack_cfg_query_all(
        &self,
        timeout: Duration,
    ) -> Result<BoardConfigSnapshot, DriverError> {
        match tokio::time::timeout(timeout, self.bus.recv_ack_async()).await {
            Ok(Ok(event)) => {
                if let DriverEvent::AckCfgQueryAll(config) = event {
                    Ok(config)
                } else {
                    Err(DriverError::Timeout)
                }
            }
            Ok(Err(e)) => Err(e),
            Err(_) => Err(DriverError::Timeout),
        }
    }

    async fn wait_for_ack_cfg_write(&self, timeout: Duration) -> Result<bool, DriverError> {
        match tokio::time::timeout(timeout, self.bus.recv_ack_async()).await {
            Ok(Ok(event)) => {
                if let DriverEvent::AckCfgWrite { success } = event {
                    Ok(success)
                } else {
                    Err(DriverError::Timeout)
                }
            }
            Ok(Err(e)) => Err(e),
            Err(_) => Err(DriverError::Timeout),
        }
    }

    /// 获取状态快照
    pub fn state(&self) -> Arc<DriverState> {
        self.state.clone()
    }

    /// 异步读取循环
    async fn read_loop(
        transport: Arc<Mutex<Option<Box<dyn AsyncTransport>>>>,
        _transport_factory: Option<Arc<dyn AsyncTransportFactory>>,
        bus: Arc<EventBus>,
        state: Arc<DriverState>,
        running: Arc<AtomicBool>,
        _reconnect_config: Option<ReconnectConfig>,
    ) {
        while running.load(Ordering::Relaxed) {
            // 读取一帧
            let frame_data = {
                let mut transport_guard = transport.lock().await;

                let current_transport = match transport_guard.as_mut() {
                    Some(t) => t,
                    None => {
                        state.set_error(DriverError::TransportClosed);
                        bus.sender()
                            .send(DriverEvent::Error(DriverError::TransportClosed))
                            .ok();
                        break;
                    }
                };

                match current_transport.read_frame().await {
                    Ok(data) => data,
                    Err(DriverError::TransportClosed) => {
                        state.set_connected(false);
                        *transport_guard = None;
                        bus.sender()
                            .send(DriverEvent::Error(DriverError::TransportClosed))
                            .ok();
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
                TypedFrame::AckCfgWrite { success } => DriverEvent::AckCfgWrite { success },
                TypedFrame::AckCfgQuery(config) => DriverEvent::AckCfgQuery(config),
                TypedFrame::AckCfgQueryAll(config_snapshot) => {
                    state.update_config(config_snapshot.clone());
                    DriverEvent::AckCfgQueryAll(config_snapshot)
                }
                
                _ => continue,
            };

            // 检查是否是 ACK 事件
            let is_ack = matches!(
                event,
                                    | DriverEvent::AckCfgQuery(_)
                    | DriverEvent::AckCfgQueryAll(_)
                    | DriverEvent::AckCfgWrite { .. }
            );

            // 通过事件总线分发
            bus.sender().send(event.clone()).ok();

            // ACK 事件同时发送到 ACK 通道
            if is_ack {
                bus.ack_sender().send(event).ok();
            }

            // 立即处理事件（触发回调）
            if let Err(e) = bus.try_recv_and_dispatch() {
                log::warn!("Event dispatch error: {}", e);
            }
        }

        log::info!("Async read loop exited");
    }
}

#[cfg(feature = "async")]
impl Drop for AsyncDriver {
    fn drop(&mut self) {
        // 注意：在 async 上下文中无法直接调用 async stop()
        // 用户应该在 drop 前显式调用 stop().await
    }
}
