# servo-robot-driver

STM32 串口通信驱动，用于 PC 与 STM32 之间的双向数据传输。

## 特性

- 帧协议解析（HEAD + TYPE + LEN + PAYLOAD + CRC）
- 两种回调模式：`DriverCallback` trait 和闭包注册
- 同步请求-响应 API（自动等待应答）
- 线程安全的状态快照 API
- 自动重连（可配置重试次数和退避策略）
- 模拟传输层（用于开发和测试）
- 传输层抽象，支持未来扩展到异步 I/O

## 架构设计

### 整体架构

```
┌─────────────────────────────────────────────────────────────────┐
│                         用户代码                                  │
│  (ROS2 Node / TUI / 测试)                                        │
├─────────────────────────────────────────────────────────────────┤
│                         Driver                                    │
│  ┌──────────────┐  ┌──────────────┐  ┌──────────────────────┐  │
│  │   EventBus   │  │ DriverState  │  │ 同步等待 (wait_for_*) │  │
│  │ (事件分发)    │  │ (状态快照)    │  │ (超时机制)            │  │
│  └──────┬───────┘  └──────────────┘  └──────────────────────┘  │
│         │                                                        │
├─────────┼────────────────────────────────────────────────────────┤
│         ▼                                                        │
│  ┌──────────────────────────────────────────────────────────┐   │
│  │                    Transport (trait)                       │   │
│  │  ┌──────────┐  ┌──────────┐  ┌──────────┐  ┌──────────┐ │   │
│  │  │ Serial   │  │  Mock    │  │  Async   │  │  Custom  │ │   │
│  │  │ (串口)   │  │ (模拟)   │  │ (tokio)  │  │ (扩展)   │ │   │
│  │  └──────────┘  └──────────┘  └──────────┘  └──────────┘ │   │
│  └──────────────────────────────────────────────────────────┘   │
│                                                                  │
├──────────────────────────────────────────────────────────────────┤
│                       Protocol 层                                 │
│  ┌──────────┐  ┌──────────┐  ┌──────────┐  ┌──────────┐        │
│  │  Frame   │  │  IMU     │  │  Power   │  │ Battery  │        │
│  │ (帧解析) │  │ (惯性)   │  │ (电源)   │  │ (电池)   │        │
│  └──────────┘  └──────────┘  └──────────┘  └──────────┘        │
│  ┌──────────┐  ┌──────────┐  ┌──────────┐  ┌──────────┐        │
│  │ Thermal  │  │  System  │  │  Event   │  │  Config  │        │
│  │ (温度)   │  │ (系统)   │  │ (事件)   │  │ (配置)   │        │
│  └──────────┘  └──────────┘  └──────────┘  └──────────┘        │
└──────────────────────────────────────────────────────────────────┘
```

### 数据流

```
STM32 ──串口──→ Transport.read_frame() ──→ RawFrame.decode()
                                              │
                                              ▼
                                        TypedFrame.parse_typed()
                                              │
                                              ▼
                                      DriverState.update_*()
                                              │
                                              ▼
                                      EventBus.dispatch()
                                         │         │
                                         ▼         ▼
                                    Callbacks   Closures
                                    (Pattern A) (Pattern B)
```

### 核心组件

#### 1. Transport Trait

```rust
pub trait Transport: Send + 'static {
    fn read_frame(&mut self) -> Result<Vec<u8>, DriverError>;
    fn write_frame(&mut self, frame: &[u8]) -> Result<(), DriverError>;
    fn close(&mut self) -> Result<(), DriverError>;
}
```

- **SerialTransport**: 使用 `serialport` crate 的串口实现
- **MockTransport**: 模拟真实数据，用于开发和测试
- **AsyncTransport**: tokio 异步实现（feature gated）

#### 2. Driver

```rust
pub struct Driver {
    transport: Arc<Mutex<Option<Box<dyn Transport>>>>,
    bus: Arc<EventBus>,
    state: Arc<DriverState>,
    // ...
}
```

- **读取线程**: 独立线程持续读取帧并分发事件
- **同步写入**: 通过 `Mutex` 保护的传输层写入
- **状态快照**: `DriverState` 提供线程安全的数据访问

#### 3. EventBus

```rust
pub struct EventBus {
    tx: Sender<DriverEvent>,
    ack_tx: Sender<DriverEvent>,
    callbacks: Arc<Mutex<Vec<Box<dyn DriverCallback>>>>,
    closures: Arc<Mutex<ClosureStore>>,
}
```

- 双通道设计：主事件通道 + ACK 专用通道
- 支持两种回调模式

#### 4. Protocol 层

每个数据类型实现：
- `ToPayload`: 序列化为字节
- `FromPayload`: 从字节反序列化
- `from_bytes()` / `to_bytes()`: 底层字节操作

### 帧格式

```
┌──────┬──────┬──────┬───────────────┬──────┐
│ HEAD │ TYPE │ LEN  │   PAYLOAD     │ CRC  │
│ 1B   │ 1B   │ 2B   │   0~255B      │ 2B   │
└──────┴──────┴──────┴───────────────┴──────┘

HEAD:    0xAA (固定帧头)
TYPE:    消息类型
LEN:     payload 长度 (小端 uint16)
PAYLOAD: 数据内容
CRC:     CRC-16/CCITT 校验 (从 TYPE 到 PAYLOAD 末尾)
```

### 帧类型

| 类型 | 值 | 方向 | 说明 |
|------|-----|------|------|
| Imu | 0x01 | 上行 | IMU 数据 |
| Power | 0x02 | 上行 | 电源数据 |
| Thermal | 0x03 | 上行 | 温度数据 |
| Config | 0x04 | 上行 | 配置快照 |
| Battery | 0x05 | 上行 | 电池状态 |
| System | 0x06 | 上行 | 系统信息 |
| Event | 0x07 | 上行 | 事件 |
| CfgWrite | 0x80 | 下行 | 写入配置 |
| CfgQuery | 0x81 | 下行 | 查询配置 |
| CfgQueryAll | 0x82 | 下行 | 查询所有配置 |
| AckCfgWrite | 0xC0 | 应答 | 写入确认 |
| AckCfgQuery | 0xC1 | 应答 | 配置响应 |
| AckCfgQueryAll | 0xC2 | 应答 | 所有配置 |

### 线程模型

```
┌─────────────────────────────────────────────────┐
│                   用户线程                        │
│  driver.send_command() / driver.query_config()   │
│         │                                        │
│         ▼                                        │
│  ┌──────────────┐                                │
│  │ Mutex<Transport> │ ◄── 读写互斥               │
│  └──────┬───────┘                                │
│         │                                        │
├─────────┼────────────────────────────────────────┤
│         ▼                                        │
│  ┌──────────────┐                                │
│  │  读取线程     │  loop {                        │
│  │              │    transport.read_frame()       │
│  │              │    decode → parse → dispatch    │
│  │              │  }                              │
│  └──────┬───────┘                                │
│         │                                        │
│         ▼                                        │
│  ┌──────────────┐                                │
│  │  EventBus    │ ──→ Callbacks / Closures       │
│  └──────────────┘                                │
└─────────────────────────────────────────────────┘
```

### 重连机制

```
连接断开
    │
    ▼
检查重连配置
    │
    ├── max_retries = 0 → 不重连
    │
    ▼
等待 retry_interval * backoff_multiplier^retry_count
    │
    ▼
调用 TransportFactory.create()
    │
    ├── 成功 → 重置计数器，继续
    │
    └── 失败 → retry_count++，重试
```

## 快速开始

### 添加依赖

```toml
[dependencies]
servo-robot-driver = { path = "../servo-robot-driver" }

# 启用模拟传输层（用于开发和测试）
servo-robot-driver = { path = "../servo-robot-driver", features = ["mock"] }

# 启用异步支持
servo-robot-driver = { path = "../servo-robot-driver", features = ["async"] }
```

### 使用模拟传输层（无需硬件）

```rust
use servo_robot_driver::{Driver, MockTransport};

let mock = MockTransport::new();
let mut driver = Driver::new(mock);

driver.on_imu_data(|data| {
    println!("IMU: roll={:.1}° pitch={:.1}° yaw={:.1}°",
        data.roll, data.pitch, data.yaw);
});

driver.on_battery_state(|state| {
    println!("Battery: {:.1}%", state.percentage);
});

driver.start()?;
```

### 使用真实串口

```rust
use servo_robot_driver::{Driver, SerialTransport};

let transport = SerialTransport::open("/dev/ttyUSB0", 115200)?;
let mut driver = Driver::new(transport);
driver.start()?;
```

### 自动重连

```rust
use servo_robot_driver::{Driver, SerialTransport, FnTransportFactory};
use servo_robot_driver::reconnect::ReconnectConfig;

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

// 设置初始状态
mock.set_battery_percentage(80.0);
mock.set_initial_attitude(10.0, 5.0, 0.0);
mock.set_charging(true);

// 模拟断开连接（用于测试重连）
mock.set_auto_disconnect(1000);

// 手动控制连接
mock.disconnect();
mock.reconnect();

// 获取写入的帧（用于验证命令）
let written = mock.written_frames();
```

### 模拟数据特性

| 数据类型 | 更新频率 | 模拟特性 |
|---------|---------|---------|
| IMU | 100Hz | 姿态漂移、传感器噪声、重力分量 |
| Power | 20Hz | 电池电压变化、电流波动 |
| Thermal | 5Hz | 温度上升趋势、随机波动 |
| Battery | 10Hz | 电量消耗、充电状态 |
| System | 1Hz | 运行时间、CPU 使用率 |
| Event | 1Hz | 充电状态、风扇状态、保护标志 |

## 回调模式

### Pattern A: DriverCallback trait（推荐用于 ROS2）

```rust
use servo_robot_driver::{Driver, DriverCallback};
use servo_robot_driver::protocol::imu::ImuData;
use servo_robot_driver::protocol::battery_state::BatteryState;
use servo_robot_driver::protocol::config::{BoardConfigSnapshot, Config};

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
}

driver.register_callback(MyCallback { imu_count: 0 });
```

### Pattern B: 闭包注册

```rust
driver.on_imu_data(|data| {
    println!("IMU: roll={:.1}°", data.roll);
});

driver.on_battery_state(|state| {
    println!("Battery: {:.1}%", state.percentage);
});
```

## 命令和配置

```rust
use servo_robot_driver::protocol::config::{Config, ConfigType};

// 异步（不等待应答）
driver.send_config(Config::Reset)?;
driver.query_config(ConfigType::PowerServoCurrentLimit)?;
driver.write_config(Config::PowerServoCurrentLimit(5.0))?;

// 同步（等待应答）
let config = driver.query_config_sync(ConfigType::PowerServoCurrentLimit)?;
let all_config = driver.query_all_configs_sync()?;
let success = driver.write_config_sync(Config::PowerServoCurrentLimit(5.0))?;
```

## Feature Flags

| Feature | 依赖 | 说明 |
|---------|------|------|
| `mock` | `rand` | 启用 MockTransport 模拟传输层 |
| `async` | `tokio` | 启用 AsyncDriver 和异步传输层 |
| `async,mock` | `tokio`, `rand` | 启用 AsyncMockTransport |

## 模块结构

```
src/
├── lib.rs                  # 公共 API 导出
├── crc.rs                  # CRC-16/CCITT
├── error.rs                # 错误类型
├── driver.rs               # Driver 主结构
├── driver_async.rs         # AsyncDriver [async]
├── reconnect.rs            # 重连配置
├── state.rs                # 状态快照
├── transport/
│   ├── mod.rs              # Transport trait
│   ├── factory.rs          # TransportFactory
│   ├── serial.rs           # 串口实现
│   ├── mock.rs             # 模拟传输层 [mock]
│   ├── mock_data.rs        # 模拟数据生成 [mock]
│   ├── async_trait.rs      # AsyncTransport [async]
│   ├── async_serial.rs     # 异步串口 [async]
│   └── async_mock.rs       # 异步模拟 [async,mock]
├── dispatch/
│   ├── mod.rs              # EventBus
│   ├── callback.rs         # DriverCallback
│   └── closure.rs          # 闭包存储
└── protocol/
    ├── mod.rs              # 模块声明
    ├── frame.rs            # 帧类型和解析
    ├── imu.rs              # IMU 数据
    ├── power.rs            # 电源数据
    ├── thermal.rs          # 温度数据
    ├── battery_state.rs    # 电池状态
    ├── config.rs           # 配置
    ├── command.rs          # 命令
    ├── event.rs            # 事件
    └── system.rs           # 系统信息
```

## License

GPL-3.0
