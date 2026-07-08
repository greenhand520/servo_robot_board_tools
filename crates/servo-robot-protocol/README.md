# servo-robot-protocol

Definition of communication protocol between ServoRobotBoard and host computer, supports `no_std` + `alloc`, compatible with PC and embedded platforms.

## Features

- `#![no_std]` compatible, supports embedded environments
- Frame protocol parsing (HEAD + TYPE + LEN + PAYLOAD + CRC)
- Complete data type definitions (IMU, Power, Battery, System Info + Thermal, Event, Log, Config, Servo)
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
| Config | 0x03 | Uplink | Configuration snapshot |
| Battery | 0x04 | Uplink | Battery status |
| System | 0x05 | Uplink | System information + temperature data |
| Event | 0x06 | Uplink | Event |
| Log | 0x07 | Uplink | Log message |
| CfgWrite | 0x80 | Downlink | Write configuration |
| CfgQuery | 0x81 | Downlink | Query single config |
| CfgQueryAll | 0x82 | Downlink | Query all configs |
| ServoForward | 0x83 | Downlink | Forward servo command |
| AckCfgWrite | 0xC0 | Response | Write confirmation |
| AckCfgQuery | 0xC1 | Response | Single config response |
| AckCfgQueryAll | 0xC2 | Response | All configs response |
| AckServoCmd | 0xC3 | Response | Servo command response |

## Data Types

### Integer Type Convention

All protocol data uses integer types with scaling factors for efficient transmission:

| Data Type | Protocol Type | Scaling | Example |
|-----------|---------------|---------|---------|
| Voltage | u16 | mV | 8660 = 8.66V |
| Current | u16/i16 | mA | 1250 = 1.25A |
| Capacity | u16 | mAh | 4600 = 4600mAh |
| Temperature | i16 | ×10 | 571 = 57.1°C |
| Percentage | u8 | 1~100 | 75 = 75% |

### Uplink Data

| Type | Fields | Update Rate |
|------|--------|-------------|
| `ImuData` | accel[3], gyro[3], quaternion[4], roll, pitch, yaw | 100Hz |
| `PowerData` | servo_voltage_mv/current_ma, charge_in_voltage_mv/current_ma, bat_voltage_mv/current_ma | 20Hz |
| `BatteryState` | voltage_mv, current_ma, percentage, capacity_mah, cell_voltages_mv, ... | 10Hz |
| `SystemInfo` | device_id, uid, uptime, cpu_usage, heap, stack, frames_sent, pd_voltage/current, version, temperatures | 1Hz |
| `BoardEvent` | charge_phase, state_change_flags, protection_flags, error_flags | Triggered |
| `LogMessage` | level, file_name, fun_name, msg | Triggered |
| `BoardConfigSnapshot` | All config params + switch states | Event triggered |

### PowerData

Power electrical measurements.

| Field | Type | Unit | Description |
|-------|------|------|-------------|
| servo_voltage_mv | u16 | mV | Servo power supply voltage |
| servo_current_ma | u16 | mA | Servo power supply current |
| charge_in_voltage_mv | u16 | mV | USB-PD input voltage |
| charge_in_current_ma | u16 | mA | USB-PD input current |
| bat_voltage_mv | u16 | mV | Battery voltage |
| bat_current_ma | i16 | mA | Battery current (+ charging, - discharging) |

### BatteryState

Battery status information.

| Field | Type | Unit | Description |
|-------|------|------|-------------|
| voltage_mv | u16 | mV | Total battery voltage |
| current_ma | i16 | mA | Battery current |
| percentage | u8 | 1~100 | Battery percentage |
| capacity_mah | u16 | mAh | Actual capacity |
| design_capacity_mah | u16 | mAh | Design capacity |
| temperature | i16 | ×10 | Battery temperature |
| charge_status | BatteryChargeStatus | - | Charge status |
| health | BatteryHealth | - | Battery health status |
| technology | BatteryTechnology | - | Battery technology type |
| present | bool | - | Whether battery is present |
| serial_number | u16 | - | Serial number |
| cell_voltages_mv | Vec\<u16\> | mV | Cell voltages |
| cell_temperatures | Vec\<i16\> | ×10 | Cell temperatures |

### SystemInfo

System information including temperature data (merged from ThermalData).

| Field | Type | Unit | Description |
|-------|------|------|-------------|
| device_id | u16 | - | STM32 device ID |
| uid | u32 | - | STM32 unique ID |
| imu_id | u8 | - | IMU chip ID |
| uptime_s | u32 | s | Uptime (seconds) |
| cpu_usage_percent | u8 | % | CPU usage |
| free_heap_kb | u16 | KB | Free heap memory |
| stack_watermark_min_kb | u16 | KB | Stack watermark minimum |
| i2c_error_count | u16 | - | I2C error count |
| spi_error_count | u16 | - | SPI error count |
| uart_error_count | u16 | - | UART error count |
| usb_error_count | u16 | - | USB error count |
| frames_sent_total | u32 | - | Total frames sent |
| pd_request_voltage_mv | u16 | mV | PD protocol voltage |
| pd_request_current_ma | u16 | mA | PD protocol current |
| firmware_version | Version | - | Firmware version |
| temp_servo_power | i16 | ×10 | Servo power temperature |
| temp_5v_power | i16 | ×10 | 5V power temperature |
| temp_mcu | i16 | ×10 | MCU temperature |
| temp_charge | i16 | ×10 | Charge circuit temperature |
| temp_battery | i16 | ×10 | Battery temperature |

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
| ChargeStopSoc | 0x20 | Charge stop SOC (%) |
| TxLogLevel | 0x21 | STM32 transmit log level |
| PowerServoCurrentLimitMa | 0x30 | Servo current limit (mA) |
| PowerServoTempLimit | 0x31 | Servo temperature limit (×10) |
| Power5vTempLimit | 0x32 | 5V temperature limit (×10) |
| ChargeMaxCurrentMa | 0x33 | Charge max current (mA) |
| ChargeTempDerating | 0x34 | Charge temp derating (×10) |
| ChargeTempLimit | 0x35 | Charge temp limit (×10) |
| ChargeStopVoltageMv | 0x36 | Charge stop voltage (mV) |

### BoardConfigSnapshot

Board configuration snapshot for querying and displaying current configuration state.

| Field | Type | Unit | Default | Description |
|-------|------|------|---------|-------------|
| servo_current_limit_ma | u16 | mA | 50 | Servo current limit |
| servo_temp_limit | u16 | ×10 | 800 | Servo temperature limit (80.0°C) |
| temp_5v_limit | u16 | ×10 | 700 | 5V temperature limit (70.0°C) |
| charge_max_current_ma | u16 | mA | 90 | Charge max current |
| charge_temp_derating | u16 | ×10 | 600 | Charge temp derating (60.0°C) |
| charge_temp_limit | u16 | ×10 | 700 | Charge stop temperature (70.0°C) |
| charge_stop_voltage_mv | u16 | mV | 168 | Charge stop voltage |
| charge_stop_percentage | u8 | 1~100 | 100 | Charge stop percentage |
| charge_enable | bool | - | true | Charge enable |
| power_servo_on | bool | - | true | Servo power switch |
| power_5v_on | bool | - | true | 5V power switch |
| charge_on | bool | - | true | Charge switch |
| bat_ext_out_on | bool | - | true | Battery extra output switch |
| tx_log_level | LogLevel | - | Info | STM32 transmit log level |

### ServoCmdWrapper

Servo command wrapper for forwarding raw bytes to servo bus. It may contain multiple servo operation commands

```rust
pub struct ServoCmdWrapper {
    data: Vec<u8>,  // Raw servo command bytes
}

impl ServoCmdWrapper {
    pub fn new(data: Vec<u8>) -> Self;
    pub fn data(&self) -> &[u8];
    pub fn into_data(self) -> Vec<u8>;
}
```

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
    FrameType::System => {
        let sys = SystemInfo::from_bytes(&frame.payload)?;
        println!("MCU Temp: {:.1}°C", sys.temp_mcu as f32 / 10.0);
    }
    _ => {}
}
```

### Encode Frame

```rust
use servo_robot_protocol::frame::{RawFrame, FrameType};
use servo_robot_protocol::config::Config;

let config = Config::PowerServoCurrentLimitMa(5000); // 5000mA = 5A
let frame = RawFrame {
    frame_type: FrameType::CfgWrite,
    payload: config.to_bytes(),
};
let bytes = frame.encode(); // Includes HEAD + TYPE + LEN + PAYLOAD + CRC
```

### Servo Command Forwarding

```rust
use servo_robot_protocol::servo::ServoCommand;
use servo_robot_protocol::frame::{RawFrame, FrameType};

// Create servo command from raw bytes
let cmd = ServoCommand::new(vec![0x01, 0x02, 0x03]);

// Encode as frame
let frame = RawFrame {
    frame_type: FrameType::ServoForward,
    payload: cmd.to_payload(),
};
let bytes = frame.encode();
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
├── battery_state.rs    # BatteryState
├── system.rs           # SystemInfo, Version (includes thermal data)
├── event.rs            # BoardEvent, EventLog, EventKind, EventCategory
├── log.rs              # LogMessage, LogLevel
├── config.rs           # ConfigType, Config, BoardConfigSnapshot
├── servo.rs            # ServoCommand (raw bytes wrapper)
└── thermal.rs          # ThermalData (deprecated, kept for compatibility)
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
