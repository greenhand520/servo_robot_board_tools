# servo-robot-protocol

STM32 Robot Communication Protocol Definition, supports `no_std` + `alloc`, compatible with PC and embedded platforms.

**[中文版](README.md)**

## Features

- `#![no_std]` compatible, supports embedded environments
- Frame protocol parsing (HEAD + TYPE + LEN + PAYLOAD + CRC)
- Complete data type definitions (IMU, Power, Thermal, Battery, System Info, Event, Log, Config)
- CRC-16/CCITT checksum
- Platform switching via `embedded` feature

## Quick Start

### Reference Methods

```toml
# Option 1: Local path
[dependencies]
servo-robot-protocol = { path = "../servo-robot-protocol" }

# Option 2: GitHub reference
[dependencies]
servo-robot-protocol = { git = "https://github.com/greenhand520/servo_robot_board_tools", branch = "main" }

# Embedded mode (no_std)
[dependencies]
servo-robot-protocol = { git = "https://github.com/greenhand520/servo_robot_board_tools", branch = "main", default-features = false, features = ["embedded"] }
```

## Frame Format

```
┌──────┬──────┬──────┬───────────────┬──────┐
│ HEAD │ TYPE │ LEN  │   PAYLOAD     │ CRC  │
│ 1B   │ 1B   │ 2B   │   0~255B      │ 2B   │
└──────┴──────┴──────┴───────────────┴──────┘

HEAD:    0xAA (Fixed header)
TYPE:    Message type
LEN:     Payload length (little-endian uint16)
PAYLOAD: Data content
CRC:     CRC-16/CCITT checksum (from TYPE to end of PAYLOAD)
```

## Frame Types

| Type | Value | Direction | Description |
|------|-------|-----------|-------------|
| Imu | 0x01 | Uplink | IMU inertial measurement data |
| Power | 0x02 | Uplink | Power electrical data |
| Thermal | 0x03 | Uplink | Temperature data |
| Config | 0x04 | Uplink | Configuration snapshot |
| Battery | 0x05 | Uplink | Battery status |
| System | 0x06 | Uplink | System information |
| Event | 0x07 | Uplink | Event |
| Log | 0x08 | Uplink | Log message |
| CfgWrite | 0x80 | Downlink | Write configuration |
| CfgQuery | 0x81 | Downlink | Query single config |
| CfgQueryAll | 0x82 | Downlink | Query all configs |
| AckCfgWrite | 0xC0 | Response | Write confirmation |
| AckCfgQuery | 0xC1 | Response | Single config response |
| AckCfgQueryAll | 0xC2 | Response | All configs response |

## Data Types

### Uplink Data

| Type | Fields | Update Rate |
|------|--------|-------------|
| `ImuData` | accel[3], gyro[3], quaternion[4], roll, pitch, yaw | 100Hz |
| `PowerData` | servo_voltage/current, charge_in_voltage/current, bat_voltage/current | 20Hz |
| `ThermalData` | temp_servo_power, temp_5v_power, temp_mcu, temp_charge, temp_battery | 5Hz |
| `BatteryState` | voltage, current, soc, percentage, cell_voltages, ... | 10Hz |
| `SystemInfo` | uptime, cpu_usage, heap, stack, frames_sent, pd_voltage/current | 1Hz |
| `BoardEvent` | charger_connected, fan_enabled, charge_phase, protection_flags | Triggered |
| `LogMessage` | level, file_name, fun_name, msg | Triggered |
| `BoardConfigSnapshot` | All config params + switch states | Event triggered |

### Configuration Types

| Type | Value | Description |
|------|-------|-------------|
| Reset | 0x01 | Reset device |
| Shutdown | 0x02 | Shutdown device |
| SwitchServoPower | 0x10 | Servo power switch |
| Switch5VPower | 0x11 | 5V power switch |
| SwitchCharge | 0x12 | Charge switch |
| SwitchBatExtOut | 0x13 | Battery extra output switch |
| PowerServoCurrentLimit | 0x21 | Servo current limit |
| PowerServoTempLimit | 0x22 | Servo temperature limit |
| Power5vTempLimit | 0x23 | 5V temperature limit |
| ChargeMaxCurrent | 0x24 | Charge max current |
| ChargeTempDerating | 0x25 | Charge temp derating |
| ChargeTempLimit | 0x26 | Charge temp limit |
| ChargeStopVoltage | 0x27 | Charge stop voltage |
| ChargeStopSoc | 0x28 | Charge stop SOC |
| TxLogLevel | 0x29 | STM32 transmit log level |

## Usage Examples

### Parse Frame

```rust
use servo_robot_protocol::frame::{RawFrame, FrameType};
use servo_robot_protocol::imu::ImuData;

// Decode from bytes
let (frame, consumed) = RawFrame::decode(&raw_bytes)?;

// Parse into specific type
match frame.frame_type {
    FrameType::Imu => {
        let imu = ImuData::from_bytes(&frame.payload)?;
        println!("Roll: {:.1}°", imu.roll);
    }
    _ => {}
}
```

### Encode Frame

```rust
use servo_robot_protocol::frame::{RawFrame, FrameType};
use servo_robot_protocol::config::Config;

let config = Config::PowerServoCurrentLimit(5.0);
let frame = RawFrame {
    frame_type: FrameType::CfgWrite,
    payload: config.to_bytes(),
};
let bytes = frame.encode(); // Includes HEAD + TYPE + LEN + PAYLOAD + CRC
```

### CRC Calculation

```rust
use servo_robot_protocol::crc::crc16_ccitt_table;

let data = b"Hello";
let crc = crc16_ccitt_table(data);
```

## Module Structure

```
src/
├── lib.rs              # #![no_std] entry point
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

## Dependencies

- `bitflags` - Bit flags (supports no_std)

## Feature Flags

| Feature | Description |
|---------|-------------|
| `std` (default) | Enable standard library support |
| `embedded` | Embedded mode (no_std) |

## Design Principles

1. **Zero-copy parsing**: Uses `from_le_bytes()` for direct parsing, no `Cursor` needed
2. **Static memory**: Error messages use `&'static str`, no heap allocation
3. **Optional heap**: `Vec<u8>` only used when `alloc` is available
4. **Type safety**: Uses enums and structs to ensure data correctness

## License

GPL-3.0
