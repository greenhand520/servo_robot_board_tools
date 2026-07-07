# servo-robot-protocol

[English](README_en.md) | 简体中文

ServoRobotBoard与上位机通信协议定义，支持 `no_std` + `alloc`，可用于 PC 和嵌入式平台。

## 特性

- `#![no_std]` 兼容，支持嵌入式环境
- 帧协议解析（HEAD + TYPE + LEN + PAYLOAD + CRC）
- 完整的数据类型定义（IMU、电源、温度、电池、系统信息、事件、日志、配置）
- CRC-16/CCITT 校验
- 通过 `embedded` feature 切换平台支持

## 快速开始

### 引用方式

```toml
# 方式1: 本地路径
[dependencies]
servo-robot-protocol = { path = "../servo-robot-protocol" }

# 方式2: GitHub 引用
[dependencies]
servo-robot-protocol = { git = "https://github.com/greenhand520/servo_robot_board_tools", branch = "main" }

# 嵌入式模式（no_std）
[dependencies]
servo-robot-protocol = { git = "https://github.com/greenhand520/servo_robot_board_tools", branch = "main", default-features = false, features = ["embedded"] }
```

## 帧格式

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

## 帧类型

| 类型 | 值 | 方向 | 说明 |
|------|-----|------|------|
| Imu | 0x01 | 上行 | IMU 惯性测量数据 |
| Power | 0x02 | 上行 | 电源电气数据 |
| Thermal | 0x03 | 上行 | 温度数据 |
| Config | 0x04 | 上行 | 配置快照 |
| Battery | 0x05 | 上行 | 电池状态 |
| System | 0x06 | 上行 | 系统信息 |
| Event | 0x07 | 上行 | 事件 |
| Log | 0x08 | 上行 | 日志消息 |
| CfgWrite | 0x80 | 下行 | 写入配置 |
| CfgQuery | 0x81 | 下行 | 查询单个配置 |
| CfgQueryAll | 0x82 | 下行 | 查询所有配置 |
| AckCfgWrite | 0xC0 | 应答 | 写入确认 |
| AckCfgQuery | 0xC1 | 应答 | 单个配置响应 |
| AckCfgQueryAll | 0xC2 | 应答 | 所有配置响应 |

## 数据类型

### 上行数据

| 类型 | 字段 | 更新频率 |
|------|------|----------|
| `ImuData` | accel[3], gyro[3], quaternion[4], roll, pitch, yaw | 100Hz |
| `PowerData` | servo_voltage/current, charge_in_voltage/current, bat_voltage/current | 20Hz |
| `ThermalData` | temp_servo_power, temp_5v_power, temp_mcu, temp_charge, temp_battery | 5Hz |
| `BatteryState` | voltage, current, percentage, capacity, design_capacity, cell_voltages, ... | 10Hz |
| `SystemInfo` | device_id, uid, uptime, cpu_usage, heap, stack, frames_sent, pd_voltage/current, version | 1Hz |
| `BoardEvent` | charge_phase, state_change_flags, protection_flags, error_flags | 触发式 |
| `LogMessage` | level, file_name, fun_name, msg | 触发式 |
| `BoardConfigSnapshot` | 所有配置参数 + 开关状态 | 事件触发 |

### BatteryState

电池状态信息，`percentage` 范围为 0.0~1.0。

| 字段 | 类型 | 说明 |
|------|------|------|
| voltage | f32 | 电池总电压 |
| current | f32 | 电池电流 |
| percentage | f32 | 电量百分比 (0.0~1.0) |
| capacity | f32 | 实际容量 (mAh) |
| design_capacity | f32 | 设计容量 (mAh) |
| temperature | f32 | 电池温度 |
| charge_status | BatteryChargeStatus | 充电状态 |
| health | BatteryHealth | 电池健康状态 |
| technology | BatteryTechnology | 电池技术类型 |
| present | bool | 电池是否存在 |
| serial_number | u32 | 序列号 |
| cell_voltages | Vec\<f32\> | 各节电芯电压 |
| cell_temperatures | Vec\<f32\> | 各节电芯温度 |

### SystemInfo

系统信息，包含固件版本。

| 字段 | 类型 | 说明 |
|------|------|------|
| device_id | u16 | STM32 设备 ID |
| uid | u32 | STM32 全球唯一 ID |
| imu_id | u8 | IMU 芯片 ID |
| uptime_s | u32 | 运行时间 (秒) |
| cpu_usage_percent | u8 | CPU 使用率 |
| free_heap_kb | u16 | 空闲堆内存 (KB) |
| stack_watermark_min_kb | u16 | 栈最小水位 (KB) |
| i2c_error_count | u16 | I2C 错误计数 |
| spi_error_count | u16 | SPI 错误计数 |
| uart_error_count | u16 | UART 错误计数 |
| usb_error_count | u16 | USB 错误计数 |
| frames_sent_total | u32 | 发送帧总数 |
| pd_request_voltage | u16 | PD 协议电压 (mV) |
| pd_request_current | u16 | PD 协议电流 (mA) |
| firmware_version | Version | 固件版本 |

#### Version 结构

```rust
pub struct Version {
    pub major: u8,
    pub minor: u8,
    pub patch: u8,
}
// 显示格式: "major.minor.patch"，如 "0.1.0"
```

### BoardEvent

事件数据，使用位域标志位。

| 字段 | 类型 | 说明 |
|------|------|------|
| charge_phase | ChargePhase | 充电阶段 |
| state_change_flags | StateChangeFlags | 状态变化标志位 |
| protection_flags | ProtectionFlags | 保护标志位 |
| error_flags | ErrorFlags | 错误标志位 |

#### StateChangeFlags

```rust
bitflags! {
    pub struct StateChangeFlags: u16 {
        const CHARGER_CONNECTED = 0x0001;
        const FAN_ENABLED       = 0x0002;
        const SERVO_POWER_ON    = 0x0004;
        const POWER_5V_ON       = 0x0008;
        const BAT_EXT_OUT_ON    = 0x0010;
    }
}
```

#### ProtectionFlags

```rust
bitflags! {
    pub struct ProtectionFlags: u16 {
        const SERVO_OVERCURRENT  = 0x0001;
        const SERVO_OVERTEMP     = 0x0002;
        const V5V_OVERTEMP       = 0x0004;
        const CHARGE_OVERCURRENT = 0x0008;
        const CHARGE_OVERTEMP    = 0x0010;
        const BAT_LOW_VOLTAGE    = 0x0020;
    }
}
```

#### ErrorFlags

```rust
bitflags! {
    pub struct ErrorFlags: u16 {
        const UNKNOWN_ERROR = 0x0001;
        const UART1_ERROR   = 0x0002;
        const UART2_ERROR   = 0x0004;
        const I2C1_ERROR    = 0x0008;
        const I2C3_ERROR    = 0x0010;
        const SPI1_ERROR    = 0x0020;
        const USB_ERROR     = 0x0040;
        const DMA_ERROR     = 0x0080;
    }
}
```

### 事件日志类型

用于 TUI 事件显示的类型定义：

```rust
/// 事件日志条目
pub struct EventLog {
    pub ts: u64,           // 时间戳 (ms)
    pub kind: EventKind,   // 事件类型
}

/// 事件类型枚举 (24 种)
pub enum EventKind {
    // 充电相关
    ChargerConnected, ChargerDisconnected,
    ChargePhaseCc, ChargePhaseCv, ChargePhaseFull, ChargePhaseDone,
    // 状态变化
    ServoPowerOn, ServoPowerOff,
    Power5vOn, Power5vOff,
    FanEnabled, FanDisabled,
    BatExtOutOn, BatExtOutOff,
    // 保护事件
    ServoOvercurrent, ServoOvertemp,
    V5vOvertemp,
    ChargeOvercurrent, ChargeOvertemp,
    BatLowVoltage,
    // 错误事件
    UnknownError, UartError, I2cError, SpiError,
}

/// 事件分类 (4 种)
pub enum EventCategory {
    Charger,
    StateChange,
    Protection,
    Error,
}
```

### 配置类型

| 类型 | 值 | 说明 |
|------|-----|------|
| Reset | 0x01 | 复位 |
| Shutdown | 0x02 | 关机 |
| SwitchServoPower | 0x10 | 舵机电源开关 |
| Switch5VPower | 0x11 | 5V 电源开关 |
| SwitchCharge | 0x12 | 充电开关 |
| SwitchBatExtOut | 0x13 | 电池额外输出开关 |
| PowerServoCurrentLimit | 0x21 | 舵机电流限制 |
| PowerServoTempLimit | 0x22 | 舵机温度限制 |
| Power5vTempLimit | 0x23 | 5V 温度限制 |
| ChargeMaxCurrent | 0x24 | 充电最大电流 |
| ChargeTempDerating | 0x25 | 充电降流温度 |
| ChargeTempLimit | 0x26 | 充电停止温度 |
| ChargeStopVoltage | 0x27 | 充电停止电压 |
| ChargeStopSoc | 0x28 | 充电电量限制 |
| TxLogLevel | 0x29 | STM32 发送日志等级 |

### BoardConfigSnapshot

板级配置快照，用于查询和显示当前配置状态。

| 字段 | 类型 | 默认值 | 说明 |
|------|------|--------|------|
| servo_current_limit | f32 | 5.0 | 舵机电流限制 (A) |
| servo_temp_limit | f32 | 80.0 | 舵机温度限制 (°C) |
| temp_5v_limit | f32 | 70.0 | 5V 温度限制 (°C) |
| charge_max_current | f32 | 9.0 | 充电最大电流 (A) |
| charge_temp_derating | f32 | 60.0 | 充电降流温度 (°C) |
| charge_temp_limit | f32 | 70.0 | 充电停止温度 (°C) |
| charge_stop_voltage | f32 | 16.8 | 充电停止电压 (V) |
| charge_stop_percentage | f32 | 1.0 | 充电电量限制 (0~1) |
| charge_enable | bool | true | 充电使能 |
| power_servo_on | bool | true | 舵机电源开关 |
| power_5v_on | bool | true | 5V 电源开关 |
| charge_on | bool | true | 充电开关 |
| bat_ext_out_on | bool | true | 电池额外输出开关 |
| tx_log_level | LogLevel | Info | STM32 发送日志等级 |

## 使用示例

### 解析帧

```rust
use servo_robot_protocol::frame::{RawFrame, FrameType};
use servo_robot_protocol::imu::ImuData;

// 从字节解码
let (frame, consumed) = RawFrame::decode(&raw_bytes)?;

// 解析为具体类型
match frame.frame_type {
    FrameType::Imu => {
        let imu = ImuData::from_bytes(&frame.payload)?;
        println!("Roll: {:.1}°", imu.roll);
    }
    _ => {}
}
```

### 编码帧

```rust
use servo_robot_protocol::frame::{RawFrame, FrameType};
use servo_robot_protocol::config::Config;

let config = Config::PowerServoCurrentLimit(5.0);
let frame = RawFrame {
    frame_type: FrameType::CfgWrite,
    payload: config.to_bytes(),
};
let bytes = frame.encode(); // 包含 HEAD + TYPE + LEN + PAYLOAD + CRC
```

### 事件处理

```rust
use servo_robot_protocol::event::{BoardEvent, EventKind, EventCategory};

// 与上一次状态对比，获取新增事件
let prev_event = BoardEvent::default();
let new_events = current_event.diff_events(&prev_event);

for kind in new_events {
    let category = kind.category();
    match category {
        EventCategory::Charger => println!("充电事件: {:?}", kind),
        EventCategory::Protection => println!("保护事件: {:?}", kind),
        EventCategory::Error => println!("错误事件: {:?}", kind),
        _ => {}
    }
}
```

### CRC 计算

```rust
use servo_robot_protocol::crc::crc16_ccitt_table;

let data = b"Hello";
let crc = crc16_ccitt_table(data);
```

## 模块结构

```
src/
├── lib.rs              # #![no_std] 入口
├── crc.rs              # CRC-16/CCITT
├── error.rs            # FrameError
├── frame.rs            # RawFrame, TypedFrame, FrameType
├── imu.rs              # ImuData
├── power.rs            # PowerData
├── thermal.rs          # ThermalData
├── battery_state.rs    # BatteryState
├── system.rs           # SystemInfo, Version
├── event.rs            # BoardEvent, EventLog, EventKind, EventCategory
├── log.rs              # LogMessage, LogLevel
└── config.rs           # ConfigType, Config, BoardConfigSnapshot
```

## 依赖

- `bitflags` - 位标志（支持 no_std）

## Feature Flags

| Feature | 说明 |
|---------|------|
| `std` (默认) | 启用标准库支持 |
| `embedded` | 嵌入式模式（no_std） |

## License

GPL-3.0
