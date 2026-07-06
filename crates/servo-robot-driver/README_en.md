# servo-robot-driver

English | [简体中文](README.md)

The communication driver between the host computer and ServoRobotBoard is used for bidirectional data transmission between the two through the serial port.

## Features

- Frame protocol parsing (HEAD + TYPE + LEN + PAYLOAD + CRC)
- `DriverCallback` trait callback mechanism
- Synchronous request-response API (auto-wait for ACK)
- Thread-safe state snapshot API
- Auto-reconnection (configurable retries and backoff)
- Mock transport layer (for development and testing)
- Sync/async dual drivers (`Driver` / `AsyncDriver`)
- Board log dispatch via `DriverCallback::on_log`, default output via `log` crate

## Architecture

### Overview

```
┌─────────────────────────────────────────────────────────────────┐
│                         User Code                                │
│  (ROS2 Node / TUI / Tests)                                       │
├─────────────────────────────────────────────────────────────────┤
│                   Driver / AsyncDriver                            │
│  ┌──────────────┐  ┌──────────────┐  ┌──────────────────────┐  │
│  │   EventBus   │  │ DriverState  │  │ Sync Wait (wait_for_*)│  │
│  │ (Dispatch)   │  │ (Snapshot)   │  │ (recv_timeout)       │  │
│  └──────┬───────┘  └──────────────┘  └──────────────────────┘  │
│         │                                                        │
│  ┌──────┴───────┐                                                │
│  │ driver_common │ (Frame encode / decode)                        │
│  └──────┬───────┘                                                │
├─────────┼────────────────────────────────────────────────────────┤
│         ▼                                                        │
│  ┌──────────────────────────────────────────────────────────┐   │
│  │              Transport / AsyncTransport (trait)            │   │
│  │  ┌──────────┐  ┌──────────┐  ┌──────────┐  ┌──────────┐ │   │
│  │  │ Serial   │  │  Mock    │  │ Tokio    │  │  Custom  │ │   │
│  │  └──────────┘  └──────────┘  └──────────┘  └──────────┘ │   │
│  └──────────────────────────────────────────────────────────┘   │
├──────────────────────────────────────────────────────────────────┤
│                       Protocol Layer                              │
│  ┌──────────┐  ┌──────────┐  ┌──────────┐  ┌──────────┐        │
│  │  Frame   │  │  IMU     │  │  Power   │  │ Battery  │        │
│  └──────────┘  └──────────┘  └──────────┘  └──────────┘        │
│  ┌──────────┐  ┌──────────┐  ┌──────────┐  ┌──────────┐        │
│  │ Thermal  │  │  System  │  │  Event   │  │   Log    │        │
│  └──────────┘  └──────────┘  └──────────┘  └──────────┘        │
│  ┌──────────┐                                                    │
│  │  Config  │                                                    │
│  └──────────┘                                                    │
└──────────────────────────────────────────────────────────────────┘
```

### Data Flow

```
STM32 ──serial──→ Transport.read_frame() ──→ decode_and_dispatch()
                                                  │
                                  ┌───────────────┼───────────────┐
                                  ▼               ▼               ▼
                          state.update_*()   return Event    ACK Event
                                  │               │               │
                                  ▼               ▼               ▼
                          DriverState       bounded channel   ACK Channel
                          (Latest Snapshot)  (1024)           (Sync Wait)
                                  │               │
                                  ▼               ▼
                          state.snapshot()   Dispatch Thread → dispatch(callbacks)
                          (TUI/ROS2 Poll)
```

- **High-frequency periodic data** (IMU/Power/Battery, etc.): polled via `state.snapshot()`
- **Low-frequency triggered data** (Log/Event, etc.): processed via `DriverCallback` callbacks

### Core Components

#### 1. Transport Trait

```rust
pub trait Transport: Send + 'static {
    fn read_frame(&mut self) -> Result<Vec<u8>, DriverError>;
    fn write_frame(&mut self, frame: &[u8]) -> Result<(), DriverError>;
    fn close(&mut self) -> Result<(), DriverError>;
}
```

- **SerialTransport**: Serial implementation using `serialport` crate
- **MockTransport**: Simulated data for development and testing
- **AsyncTransport**: Tokio async implementation (feature gated)

#### 2. Driver

```rust
pub struct Driver {
    transport: Arc<Mutex<Option<Box<dyn Transport>>>>,
    bus: Arc<EventBus>,
    state: Arc<DriverState>,
    // ...
}
```

- **Read Thread**: Dedicated thread continuously reads frames, updates state, sends events to channel
- **Dispatch Thread**: Dedicated thread consumes events from channel, triggers callbacks (non-blocking)
- **Synchronous Writes**: Transport writes protected by `Mutex`
- **State Snapshot**: `DriverState` provides thread-safe data access

#### 3. EventBus

```rust
pub struct EventBus {
    tx: Sender<DriverEvent>,       // bounded(1024), main event channel
    ack_tx: Sender<DriverEvent>,   // unbounded, ACK-only channel
    callbacks: Arc<Mutex<Vec<Box<dyn DriverCallback>>>>,
}
```

- Dual-channel design: main event channel (bounded, auto backpressure) + ACK channel (unbounded)
- Callbacks fire on dedicated dispatch thread, non-blocking to read thread

#### 4. Protocol Layer

Each data type implements:
- `ToPayload`: Serialize to bytes
- `FromPayload`: Deserialize from bytes
- `from_bytes()` / `to_bytes()`: Low-level byte operations

### Frame Format

```
┌──────┬──────┬──────┬───────────────┬──────┐
│ HEAD │ TYPE │ LEN  │   PAYLOAD     │ CRC  │
│ 1B   │ 1B   │ 2B   │   0~255B      │ 2B   │
└──────┴──────┴──────┴───────────────┴──────┘

HEAD:    0xAA (fixed header)
TYPE:    Message type
LEN:     Payload length (little-endian uint16)
PAYLOAD: Data content
CRC:     CRC-16/CCITT checksum (from TYPE to end of PAYLOAD)
```

### Frame Types

| Type | Value | Direction | Description |
|------|-------|-----------|-------------|
| Imu | 0x01 | Upstream | IMU data |
| Power | 0x02 | Upstream | Power data |
| Thermal | 0x03 | Upstream | Temperature data |
| Config | 0x04 | Upstream | Config snapshot |
| Battery | 0x05 | Upstream | Battery state |
| System | 0x06 | Upstream | System info |
| Event | 0x07 | Upstream | Event |
| Log | 0x08 | Upstream | Log message |
| CfgWrite | 0x80 | Downstream | Write config |
| CfgQuery | 0x81 | Downstream | Query config |
| CfgQueryAll | 0x82 | Downstream | Query all configs |
| AckCfgWrite | 0xC0 | Response | Write ACK |
| AckCfgQuery | 0xC1 | Response | Config response |
| AckCfgQueryAll | 0xC2 | Response | All configs |

### Threading Model

```
┌─────────────────────────────────────────────────┐
│                   User Thread                    │
│  driver.write_config() / driver.query_config()   │
│         │                                        │
│         ▼                                        │
│  ┌──────────────┐                                │
│  │ Mutex<Transport> │ ◄── Read/Write mutex       │
│  └──────┬───────┘                                │
│         │                                        │
├─────────┼────────────────────────────────────────┤
│         ▼                                        │
│  ┌──────────────┐                                │
│  │  Read Thread │  loop {                        │
│  │              │    transport.read_frame()       │
│  │              │    decode → state.update        │
│  │              │    → channel.send()             │
│  │              │  }                              │
│  └──────┬───────┘                                │
│         │ bounded channel (1024)                  │
│         ▼                                        │
│  ┌──────────────┐                                │
│  │ Dispatch Thr │  loop {                        │
│  │              │    channel.recv()               │
│  │              │    → dispatch(callbacks)        │
│  │              │  }                              │
│  └──────────────┘                                │
│                                                  │
│  ┌──────────────┐                                │
│  │  ACK Channel │  Sync wait recv_timeout()      │
│  │ (unbounded)  │                                │
│  └──────────────┘                                │
└─────────────────────────────────────────────────┘
```

### Reconnection

```
Connection Lost
    │
    ▼
Check Reconnect Config
    │
    ├── max_retries = 0 → No reconnect
    │
    ▼
Wait retry_interval * backoff_multiplier^retry_count
    │
    ▼
Call TransportFactory.create()
    │
    ├── Success → Reset counter, continue
    │
    └── Failure → retry_count++, retry
```

## Quick Start

### Add Dependency

```toml
[dependencies]
servo-robot-driver = { path = "../servo-robot-driver" }

# Enable mock transport (for dev/test)
servo-robot-driver = { path = "../servo-robot-driver", features = ["mock"] }

# Enable async support
servo-robot-driver = { path = "../servo-robot-driver", features = ["async"] }
```

### Using DriverCallback (Recommended)

```rust
use servo_robot_driver::{Driver, MockTransport, DriverCallback};
use servo_robot_driver::protocol::imu::ImuData;
use servo_robot_driver::protocol::battery_state::BatteryState;
use servo_robot_driver::protocol::log::LogMessage;

struct MyCallback;

impl DriverCallback for MyCallback {
    fn on_imu_data(&mut self, data: &ImuData) {
        println!("IMU: roll={:.1}° pitch={:.1}° yaw={:.1}°",
            data.roll, data.pitch, data.yaw);
    }

    fn on_battery_state(&mut self, state: &BatteryState) {
        println!("Battery: {:.1}%", state.percentage);
    }

    // Override default log handling
    fn on_log(&mut self, ts: u64, log_msg: &LogMessage) {
        println!("[Board][{}] {}: {}", ts, log_msg.fun_name, log_msg.msg);
    }
}

let mock = MockTransport::new();
let mut driver = Driver::new(mock);
driver.register_callback(MyCallback);
driver.start()?;
```

### Using State Snapshot (TUI Scenario)

```rust
use servo_robot_driver::{Driver, MockTransport};

let mock = MockTransport::new();
let mut driver = Driver::new(mock);
driver.start()?;

let state = driver.state();
let snap = state.snapshot();
if let Some(imu) = &snap.imu {
    println!("Roll: {:.1}°", imu.roll);
}
```

### Using Real Serial Port

```rust
use servo_robot_driver::{Driver, SerialTransport};

let transport = SerialTransport::open("/dev/ttyUSB0", 115200)?;
let mut driver = Driver::new(transport);
driver.start()?;
```

### Auto-Reconnect

```rust
use servo_robot_driver::{Driver, SerialTransport, FnTransportFactory};
use servo_robot_driver::reconnect::ReconnectConfig;
use std::time::Duration;

let factory = FnTransportFactory::new(|| {
    SerialTransport::open("/dev/ttyUSB0", 115200).map(|t| Box::new(t) as _)
});

let config = ReconnectConfig::new(5)
    .with_retry_interval(Duration::from_secs(1))
    .with_backoff_multiplier(2.0)
    .with_max_retry_interval(Duration::from_secs(30));

let mut driver = Driver::new_with_reconnect(factory, config);
driver.start()?;
```

## MockTransport API

```rust
use servo_robot_driver::MockTransport;

let mut mock = MockTransport::new();

// Set initial state
mock.set_battery_soc(80.0);
mock.set_initial_attitude(10.0, 5.0, 0.0);
mock.set_charging(true);

// Simulate disconnect (for reconnection testing)
mock.set_auto_disconnect(1000);

// Manual connection control
mock.disconnect();
mock.reconnect();

// Get written frames (for command verification)
let written = mock.written_frames();
```

### Simulated Data

| Data Type | Rate | Simulation |
|-----------|------|------------|
| IMU | 100Hz | Attitude drift, sensor noise, gravity |
| Power | 20Hz | Battery voltage variation, current fluctuation |
| Thermal | 5Hz | Temperature rise trend, random noise |
| Battery | 10Hz | SOC drain, charge state |
| System | 1Hz | Uptime, CPU usage |
| Event | 1Hz | Charge state, fan state, protection flags |

## Callback Mechanism

### DriverCallback Trait

```rust
use servo_robot_driver::{Driver, DriverCallback};
use servo_robot_driver::protocol::imu::ImuData;
use servo_robot_driver::protocol::battery_state::BatteryState;
use servo_robot_driver::protocol::config::{BoardConfigSnapshot, Config};
use servo_robot_driver::protocol::log::LogMessage;

struct MyCallback {
    imu_count: u64,
}

impl DriverCallback for MyCallback {
    fn on_imu_data(&mut self, data: &ImuData) {
        self.imu_count += 1;
        println!("IMU #{}: roll={:.1}", self.imu_count, data.roll);
    }

    fn on_battery_state(&mut self, state: &BatteryState) {
        println!("Battery: {:.1}%", state.percentage);
    }

    fn on_ack_cfg_write(&mut self, success: bool) {
        println!("Config write ack: {}", success);
    }

    fn on_ack_cfg_query(&mut self, config: &Config) {
        println!("Config: {}", config);
    }

    // Board log callback (default: output via log crate)
    fn on_log(&mut self, ts: u64, log_msg: &LogMessage) {
        println!("[Board][{}] {}: {}", ts, log_msg.fun_name, log_msg.msg);
    }
}

driver.register_callback(MyCallback { imu_count: 0 });
```

> **Note**: Callbacks fire on a dedicated dispatch thread, non-blocking to the read thread.

### Callback Methods

| Method | Trigger | Default Behavior |
|--------|---------|-----------------|
| `on_imu_data` | IMU frame (100Hz) | Empty |
| `on_power_data` | Power frame (20Hz) | Empty |
| `on_thermal_data` | Thermal frame (5Hz) | Empty |
| `on_battery_state` | Battery frame (10Hz) | Empty |
| `on_system_info` | System frame (1Hz) | Empty |
| `on_config_snapshot` | Config snapshot | Empty |
| `on_board_event` | Event frame (1Hz) | Empty |
| `on_log(ts, log_msg)` | Board log frame | Output via `log` crate with `[ServoRobotBoard]` prefix |
| `on_ack_cfg_write` | Config write ACK | Empty |
| `on_ack_cfg_query` | Single config query response | Empty |
| `on_ack_cfg_query_all` | All configs query response | Empty |
| `on_error` | Driver error | Empty |

## Log System

### Board Logs

Upstream `Log` frames from STM32 are dispatched via `DriverCallback::on_log`. Default implementation outputs via `log` crate with `[ServoRobotBoard]` prefix and timestamp:

```
[ServoRobotBoard] [14:30:05] config.c::handle_query: queried PowerServoCurrentLimit
[ServoRobotBoard] [14:30:06] imu.c::read_gyro: sensor timeout
[ServoRobotBoard] [14:30:07] main.c::app_init: boot complete
```

Log level mapping: `LogLevel::Error` → `log::error!`, `Warn` → `log::warn!`, `Info` → `log::info!`, `Debug` → `log::debug!`

### Custom Log Handling

Override `on_log` for custom log behavior (e.g., display in TUI panel, publish to ROS2 topic):

```rust
impl DriverCallback for MyCallback {
    fn on_log(&mut self, ts: u64, log_msg: &LogMessage) {
        // Custom handling, no longer output to log
        self.log_buffer.push(format!("[{}] {}::{}: {}",
            ts, log_msg.file_name, log_msg.fun_name, log_msg.msg));
    }
}
```

### Using log Backend

Connect any `log` backend to see default board log output:

```rust
env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("debug")).init();
```

## Commands and Config

```rust
use servo_robot_driver::protocol::config::{Config, ConfigType};

// Async (no wait for ACK)
driver.query_config(ConfigType::PowerServoCurrentLimit)?;
driver.write_config(Config::PowerServoCurrentLimit(5.0))?;

// Sync (wait for ACK)
let config = driver.query_config_sync(ConfigType::PowerServoCurrentLimit)?;
let all_config = driver.query_all_configs_sync()?;
let success = driver.write_config_sync(Config::PowerServoCurrentLimit(5.0))?;
```

## Feature Flags

| Feature | Dependency | Description |
|---------|-----------|-------------|
| `mock` | `rand` | Enable MockTransport |
| `async` | `tokio` | Enable AsyncDriver and async transport |
| `async,mock` | `tokio`, `rand` | Enable AsyncMockTransport |

## Module Structure

```
src/
├── lib.rs                  # Public API exports
├── error.rs                # Error types
├── driver.rs               # Driver (sync)
├── async_driver.rs         # AsyncDriver [async]
├── driver_common.rs        # Shared logic (frame encode/decode)
├── reconnect.rs            # Reconnect config
├── state.rs                # State snapshot
├── transport/
│   ├── mod.rs              # Transport / AsyncTransport trait
│   ├── factory.rs          # TransportFactory
│   ├── serial.rs           # Serial impl (with read_frame_from_reader)
│   ├── mock/               # Mock transport [mock]
│   │   ├── mod.rs
│   │   ├── mock_core.rs    # Mock shared core
│   │   ├── mock_data.rs    # Simulated data generation
│   │   └── async_mock.rs   # Async mock [async,mock]
│   ├── frame_codec.rs      # Frame codec
│   ├── async_trait.rs      # AsyncTransport [async]
│   └── async_serial.rs     # Async serial [async]
└── dispatch/
    ├── mod.rs              # EventBus
    └── callback.rs         # DriverCallback
```

## License

GPL-3.0
