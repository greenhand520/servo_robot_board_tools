//! 事件分发系统

pub mod callback;
pub mod closure;

use crate::error::DriverError;
use crate::protocol::battery_state::BatteryState;
use crate::protocol::config::{BoardConfigSnapshot, Config};
use crate::protocol::event::BoardEvent;
use crate::protocol::imu::ImuData;
use crate::protocol::power::PowerData;
use crate::protocol::thermal::ThermalData;
use crate::protocol::system::SystemInfo;
use callback::DriverCallback;
use closure::ClosureStore;
use flume::{Receiver, Sender};
use std::sync::{Arc, Mutex};

/// 事件类型，内部使用
#[derive(Debug, Clone)]
pub enum DriverEvent {
    // ═══ 上行数据 ═══
    ImuData(ImuData),
    PowerData(PowerData),
    ThermalData(ThermalData),
    BatteryState(BatteryState),
    ConfigSnapshot(BoardConfigSnapshot),
    BoardEvent(BoardEvent),
    SystemInfo(SystemInfo),

    // ═══ 应答事件 ═══
    AckCfgWrite { success: bool },
    AckCfgQuery(Config),
    AckCfgQueryAll(BoardConfigSnapshot),

    // ═══ 错误 ═══
    Error(DriverError),
}

/// 事件总线 — 连接传输层和回调系统
pub struct EventBus {
    /// 事件发送端
    tx: Sender<DriverEvent>,
    /// 事件接收端
    rx: Receiver<DriverEvent>,
    /// ACK 事件发送端（用于同步等待）
    ack_tx: Sender<DriverEvent>,
    /// ACK 事件接收端（用于同步等待）
    ack_rx: Receiver<DriverEvent>,
    /// 回调注册表（Pattern A）
    callbacks: Arc<Mutex<Vec<Box<dyn DriverCallback>>>>,
    /// 闭包注册表（Pattern B）
    closures: Arc<Mutex<ClosureStore>>,
}

impl EventBus {
    pub fn new() -> Self {
        let (tx, rx) = flume::unbounded();
        let (ack_tx, ack_rx) = flume::unbounded();
        EventBus {
            tx,
            rx,
            ack_tx,
            ack_rx,
            callbacks: Arc::new(Mutex::new(Vec::new())),
            closures: Arc::new(Mutex::new(ClosureStore::new())),
        }
    }

    /// Pattern A: 注册 trait 对象
    pub fn register_callback(&self, cb: impl DriverCallback) {
        let mut callbacks = self.callbacks.lock().unwrap();
        callbacks.push(Box::new(cb));
    }

    /// Pattern B: 注册 IMU 数据闭包
    pub fn on_imu_data(&self, f: impl FnMut(&ImuData) + Send + 'static) {
        let mut closures = self.closures.lock().unwrap();
        closures.on_imu_data(f);
    }

    /// Pattern B: 注册电源数据闭包
    pub fn on_power_data(&self, f: impl FnMut(&PowerData) + Send + 'static) {
        let mut closures = self.closures.lock().unwrap();
        closures.on_power_data(f);
    }

    /// Pattern B: 注册温度数据闭包
    pub fn on_thermal_data(&self, f: impl FnMut(&ThermalData) + Send + 'static) {
        let mut closures = self.closures.lock().unwrap();
        closures.on_thermal_data(f);
    }

    /// Pattern B: 注册电池状态闭包
    pub fn on_battery_state(&self, f: impl FnMut(&BatteryState) + Send + 'static) {
        let mut closures = self.closures.lock().unwrap();
        closures.on_battery_state(f);
    }

    /// Pattern B: 注册配置快照闭包
    pub fn on_config_snapshot(&self, f: impl FnMut(&BoardConfigSnapshot) + Send + 'static) {
        let mut closures = self.closures.lock().unwrap();
        closures.on_config_snapshot(f);
    }

    /// Pattern B: 注册事件闭包
    pub fn on_board_event(&self, f: impl FnMut(&BoardEvent) + Send + 'static) {
        let mut closures = self.closures.lock().unwrap();
        closures.on_board_event(f);
    }

    /// Pattern B: 注册系统信息闭包
    pub fn on_system_info(&self, f: impl FnMut(&SystemInfo) + Send + 'static) {
        let mut closures = self.closures.lock().unwrap();
        closures.on_system_info(f);
    }

    /// Pattern B: 注册错误闭包
    pub fn on_error(&self, f: impl FnMut(&DriverError) + Send + 'static) {
        let mut closures = self.closures.lock().unwrap();
        closures.on_error(f);
    }

    /// 获取事件发送端（给 transport 层用）
    pub fn sender(&self) -> Sender<DriverEvent> {
        self.tx.clone()
    }

    /// 获取 ACK 事件发送端（用于同步等待）
    pub fn ack_sender(&self) -> Sender<DriverEvent> {
        self.ack_tx.clone()
    }

    /// 尝试接收 ACK 事件（非阻塞）
    pub fn try_recv_ack(&self) -> Result<Option<DriverEvent>, DriverError> {
        match self.ack_rx.try_recv() {
            Ok(event) => Ok(Some(event)),
            Err(flume::TryRecvError::Empty) => Ok(None),
            Err(flume::TryRecvError::Disconnected) => Err(DriverError::TransportClosed),
        }
    }

    /// 接收一个事件（阻塞）
    pub fn recv(&self) -> Result<DriverEvent, DriverError> {
        self.rx.recv().map_err(|_| DriverError::TransportClosed)
    }

    /// 尝试接收一个事件（非阻塞）
    pub fn try_recv(&self) -> Result<Option<DriverEvent>, DriverError> {
        match self.rx.try_recv() {
            Ok(event) => Ok(Some(event)),
            Err(flume::TryRecvError::Empty) => Ok(None),
            Err(flume::TryRecvError::Disconnected) => Err(DriverError::TransportClosed),
        }
    }

    /// 分发事件给所有注册的回调
    pub fn dispatch(&self, event: DriverEvent) {
        // 1. Pattern A: trait 对象
        let mut callbacks = self.callbacks.lock().unwrap();
        for cb in callbacks.iter_mut() {
            match &event {
                DriverEvent::ImuData(d) => cb.on_imu_data(d),
                DriverEvent::PowerData(d) => cb.on_power_data(d),
                DriverEvent::ThermalData(d) => cb.on_thermal_data(d),
                DriverEvent::BatteryState(d) => cb.on_battery_state(d),
                DriverEvent::ConfigSnapshot(d) => cb.on_config_snapshot(d),
                DriverEvent::BoardEvent(d) => cb.on_board_event(d),
                DriverEvent::SystemInfo(d) => cb.on_system_info(d),
                DriverEvent::AckCfgWrite { success } => cb.on_ack_cfg_write(*success),
                DriverEvent::AckCfgQuery(config) => cb.on_ack_cfg_query(config),
                DriverEvent::AckCfgQueryAll(config) => cb.on_ack_cfg_query_all(config),
                                DriverEvent::Error(e) => cb.on_error(e),
            }
        }

        // 2. Pattern B: 闭包
        let mut closures = self.closures.lock().unwrap();
        match &event {
            DriverEvent::ImuData(d) => closures.call_imu(d),
            DriverEvent::PowerData(d) => closures.call_power(d),
            DriverEvent::ThermalData(d) => closures.call_thermal(d),
            DriverEvent::BatteryState(d) => closures.call_battery(d),
            DriverEvent::ConfigSnapshot(d) => closures.call_config_snapshot(d),
            DriverEvent::BoardEvent(d) => closures.call_board_event(d),
            DriverEvent::SystemInfo(d) => closures.call_system_info(d),
            DriverEvent::Error(e) => closures.call_error(e),
            _ => {}
        }
    }

    /// 接收并分发事件（阻塞）
    pub fn recv_and_dispatch(&self) -> Result<(), DriverError> {
        let event = self.recv()?;
        self.dispatch(event);
        Ok(())
    }

    /// 尝试接收并分发事件（非阻塞）
    pub fn try_recv_and_dispatch(&self) -> Result<bool, DriverError> {
        match self.try_recv()? {
            Some(event) => {
                self.dispatch(event);
                Ok(true)
            }
            None => Ok(false),
        }
    }

    /// 异步接收 ACK 事件
    #[cfg(feature = "async")]
    pub async fn recv_ack_async(&self) -> Result<DriverEvent, DriverError> {
        self.ack_rx
            .recv_async()
            .await
            .map_err(|_| DriverError::TransportClosed)
    }
}
