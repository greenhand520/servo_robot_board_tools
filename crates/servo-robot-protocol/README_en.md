# servo-robot-protocol

English | [简体中文](README.md)

Definition of communication protocol between ServoRobotBoard and host computer, supports `no_std` + `alloc`, compatible with PC and embedded platforms.

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
| `BatteryState` | voltage, current, percentage, capacity, design_capacity, cell_voltages, ... | 10Hz |
| `SystemInfo` | device_id, uid, uptime, cpu_usage, heap, stack, frames_sent, pd_voltage/current, version | 1Hz |
| `BoardEvent` | charge_phase, state_change_flags, protection_flags, error_flags | Triggered |
| `LogMessage` | level, file_name, fun_name, msg | Triggered |
| `BoardConfigSnapshot` | All config params + switch states | Event triggered |

### BatteryState

Battery status information. `percentage` range is 0.0~1.0.

| Field | Type | Description |
|-------|------|-------------|
| voltage | f32 | Total battery voltage |
| current | f32 | Battery current |
| percentage | f32 | Battery percentage (0.0~1.0) |
| capacity | f32 | Actual capacity (mAh) |
| design_capacity | f32 | Design capacity (mAh) |
| temperature | f32 | Battery temperature |
| charge_status | BatteryChargeStatus | Charge status |
| health | BatteryHealth | Battery health status |
| technology | BatteryTechnology | Battery technology type |
| present | bool | Whether battery is present |
| serial_number | u32 | Serial number |
| cell_voltages | Vec\<f32\| | Cell voltages |
| cell_temperatures | Vec\<f32\| | Cell temperatures |

### SystemInfo

System information including firmware version.

| Field | Type | Description |
|-------|------|-------------|
| device_id | u16 | STM32 device ID |
| uid | u32 | STM32 unique ID |
| imu_id | u8 | IMU chip ID |
| uptime_s | u32 | Uptime (seconds) |
| cpu_usage_percent | u8 | CPU usage |
| free_heap_kb | u16 | Free heap memory (KB) |
| stack_watermark_min_kb | u16 | Stack watermark minimum (KB) |
| i2c_error_count | u16 | I2C error count |
| spi_error_count | u16 | SPI error count |
| uart_error_count | u16 | UART error count |
| usb_error_count | u16 | USB error count |
| frames_sent_total | u32 | Total frames sent |
| pd_request_voltage | u16 | PD protocol voltage (mV) |
| pd_request_current | u16 | PD protocol current (mA) |
| firmware_version | Version | Firmware version |

#### Version Structure

```rust
pub struct Version {
    pub major: u8,
    pub minor: u8,
    pub patch: u8,
}
// Display format: "major.minor.patch", e.g. "0.1.0"
```

### BoardEvent

Event data using bitfield flags.

| Field | Type | Description |
|-------|------|-------------|
| charge_phase | ChargePhase | Charging phase |
| state_change_flags | StateChangeFlags | State change flags |
| protection_flags | ProtectionFlags | Protection flags |
| error_flags | ErrorFlags | Error flags |

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

### Event Log Types

Type definitions for TUI event display:

```rust
/// Event log entry
pub struct EventLog {
    pub ts: u64,           // Timestamp (ms)
    pub kind: EventKind,   // Event type
}

/// Event type enum (24 variants)
pub enum EventKind {
    // Charger related
    ChargerConnected, ChargerDisconnected,
    ChargePhaseCc, ChargePhaseCv, ChargePhaseFull, ChargePhaseDone,
    // State changes
    ServoPowerOn, ServoPowerOff,
    Power5vOn, Power5vOff,
    FanEnabled, FanDisabled,
    BatExtOutOn, BatExtOutOff,
    // Protection events
    ServoOvercurrent, ServoOvertemp,
    V5vOvertemp,
    ChargeOvercurrent, ChargeOvertemp,
    BatLowVoltage,
    // Error events
    UnknownError, UartError, I2cError, SpiError,
}

/// Event category (4 types)
pub enum EventCategory {
    Charger,
    StateChange,
    Protection,
    Error,
}
```

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

### BoardConfigSnapshot

Board configuration snapshot for querying and displaying current configuration state.

| Field | Type | Default | Description |
|-------|------|---------|-------------|
| servo_current_limit | f32 | 5.0 | Servo current limit (A) |
| servo_temp_limit | f32 | 80.0 | Servo temperature limit (°C) |
| temp_5v_limit | f32 | 70.0 | 5V temperature limit (°C) |
| charge_max_current | f32 | 9.0 | Charge max current (A) |
| charge_temp_derating | f32 | 60.0 | Charge temp derating (°C) |
| charge_temp_limit | f32 | 70.0 | Charge stop temperature (°C) |
| charge_stop_voltage | f32 | 16.8 | Charge stop voltage (V) |
| charge_stop_percentage | f32 | 1.0 | Charge stop percentage (0~1) |
| charge_enable | bool | true | Charge enable |
| power_servo_on | bool | true | Servo power switch |
| power_5v_on | bool | true | 5V power switch |
| charge_on | bool | true | Charge switch |
| bat_ext_out_on | bool | true | Battery extra output switch |
| tx_log_level | LogLevel | Info | STM32 transmit log level |

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

### Event Handling

```rust
use servo_robot_protocol::event::{BoardEvent, EventKind, EventCategory};

// Compare with previous state to get new events
let prev_event = BoardEvent::default();
let new_events = current_event.diff_events(&prev_event);

for kind in new_events {
    let category = kind.category();
    match category {
        EventCategory::Charger => println!("Charger event: {:?}", kind),
        EventCategory::Protection => println!("Protection event: {:?}", kind),
        EventCategory::Error => println!("Error event: {:?}", kind),
        _ => {}
    }
}
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
├── system.rs           # SystemInfo, Version
├── event.rs            # BoardEvent, EventLog, EventKind, EventCategory
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

## License

GPL-3.0
