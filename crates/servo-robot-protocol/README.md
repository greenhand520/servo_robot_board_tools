# servo-robot-protocol

STM32 机器人通信协议定义，支持 `no_std` + `alloc`，可用于 PC 和嵌入式平台。
[English Version](README_en.md)

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
| `BatteryState` | voltage, current, soc, percentage, cell_voltages, ... | 10Hz |
| `SystemInfo` | uptime, cpu_usage, heap, stack, frames_sent, pd_voltage/current | 1Hz |
| `BoardEvent` | charger_connected, fan_enabled, charge_phase, protection_flags | 触发式 |
| `LogMessage` | level, file_name, fun_name, msg | 触发式 |
| `BoardConfigSnapshot` | 所有配置参数 + 开关状态 | 事件触发 |

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
| ... | ... | ... |

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
├── system.rs           # SystemInfo, ResetReason
├── event.rs            # BoardEvent, ChargePhase, ProtectionFlags
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

## 设计原则

1. **零拷贝解析**：使用 `from_le_bytes()` 直接解析，无需 `Cursor`
2. **静态内存**：错误信息使用 `&'static str`，不分配堆内存
3. **可选堆**：`Vec<u8>` 仅在 `alloc` 可用时使用
4. **类型安全**：使用枚举和结构体确保数据正确性

## License

GPL-3.0
