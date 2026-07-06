//! servo-robot-driver - STM32 串口通信驱动

pub mod dispatch;
pub mod driver;
pub(crate) mod driver_common;
pub mod error;
pub mod reconnect;
pub mod state;
pub mod transport;

#[cfg(feature = "async")]
pub mod async_driver;

// 重导出协议层
pub use servo_robot_protocol as protocol;

// 便捷重导出
pub use dispatch::callback::DriverCallback;
pub use dispatch::EventBus;
pub use driver::Driver;
pub use error::{DriverError, FrameError};
pub use reconnect::ReconnectConfig;
pub use state::{DriverState, StateSnapshot};
pub use transport::factory::FnTransportFactory;
pub use transport::serial::SerialTransport;
pub use transport::{Transport, TransportFactory};

// Mock 传输层（需要启用 mock feature）
#[cfg(feature = "mock")]
pub use transport::MockTransport;

// 异步传输层（需要启用 async feature）
#[cfg(feature = "async")]
pub use async_driver::AsyncDriver;
#[cfg(feature = "async")]
pub use transport::{AsyncTransport, AsyncTransportFactory, FnAsyncTransportFactory, TokioSerialTransport};
#[cfg(all(feature = "mock", feature = "async"))]
pub use transport::AsyncMockTransport;
