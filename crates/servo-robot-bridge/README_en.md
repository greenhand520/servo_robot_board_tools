# servo-robot-bridge

English | [简体中文](README.md)

ROS2 bridge node that publishes servo-robot-driver data as ROS2 topics and exposes ROS2 services for device control.

## Quick Start

### 1. Install ROS2 Rust

```bash
# 1. Install Rust(Optiolal)
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# 2. Clone ros2_rust
mkdir -p ~/ros_pkgs/ros2_rust_ws/src
cd ~/ros_pkgs/ros2_rust_ws/src
git clone https://github.com/ros2-rust/ros2_rust.git
vcs import < ros2_rust/ros2_rust_humble.repos

# 3. Build
cd ~/ros_pkgs/ros2_rust_ws
colcon build
```

### 2. Enable ROS2 Bridge

```bash
# Default paths
source scripts/enable-ros2-bridge.sh

# Custom paths
source scripts/enable-ros2-bridge.sh ~/ros_pkgs/ros2_rust_ws ~/ros_pkgs/servo_robot_board_ws
```

The script will:
- Generate `scripts/.env` environment variable file
- Generate `Cargo.toml` and `build.rs` from templates
- Enable the bridge crate in the workspace

### 3. Set IDE Environment Variables

**RustRover / CLion users**: Add variables from `scripts/.env` to Rust environment variables:

1. Open **Settings → Rust → Environment Variables**
2. Add the following:
   - `ROS_DISTRO=humble`
   - `RUST_BACKTRACE=full`
   - `AMENT_PREFIX_PATH=...` (copy from `scripts/.env`)
   - `LD_LIBRARY_PATH=...` (copy from `scripts/.env`)

### 4. Build and Run

```bash
# Build (real serial port)
cargo build -p servo-robot-bridge

# Build (MockTransport, no hardware needed)
cargo build -p servo-robot-bridge --features mock

# Run (real serial port)
cargo run -p servo-robot-bridge

# Run (MockTransport, no hardware needed for dev/test)
cargo run -p servo-robot-bridge --features mock

# Run with custom serial params
cargo run -p servo-robot-bridge -- --ros-args -p port:=/dev/ttyUSB1 -p baud_rate:=921600

# Run with launch file
ros2 launch crates/servo-robot-bridge/launch/bridge.launch.py

# Run with launch file and override params
ros2 launch crates/servo-robot-bridge/launch/bridge.launch.py port:=/dev/ttyUSB1 baud_rate:=921600
```

> **Mock mode**: With `mock` feature enabled, the driver uses `MockTransport` to simulate STM32 data (IMU 100Hz, Battery 10Hz, etc.) — no hardware needed for developing and testing ROS2 topics/services.

## Parameters

| Parameter | Type | Default | Description |
|-----------|------|---------|-------------|
| `port` | `string` | `/dev/ttyUSB0` | Serial port device path |
| `baud_rate` | `int` | `115200` | Serial baud rate |

### Setting Parameters

#### 1. Command Line

```bash
ros2 run servo-robot-bridge servo_robot_board_bridge --ros-args -p port:=/dev/ttyUSB1 -p baud_rate:=921600
```

#### 2. YAML Parameter File

Edit `config/servo_robot_bridge.yaml`:

```yaml
servo_robot_board_bridge:
  ros__parameters:
    port: "/dev/ttyUSB0"
    baud_rate: 115200
```

Then load via launch file:

```bash
ros2 launch crates/servo-robot-bridge/launch/bridge.launch.py
```

#### 3. Runtime Inspection

```bash
# List all parameters
ros2 param list servo_robot_board_bridge

# Get parameter values
ros2 param get servo_robot_board_bridge port
ros2 param get servo_robot_board_bridge baud_rate
```

## Topics and Services

### Published Topics

| Topic | Message Type | Description |
|-------|-------------|-------------|
| `/robot/board/imu` | `sensor_msgs/Imu` | IMU data (quaternion, angular velocity, acceleration) |
| `/robot/board/power` | `BoardPower` | Power data (servo/charge/battery voltage & current) |
| `/robot/board/thermal` | `BoardThermal` | Temperature data (servo/5V/MCU/charge/battery) |
| `/robot/board/system` | `BoardSystem` | System info (device ID, uptime, error counts, PD voltage/current) |
| `/robot/board/event` | `BoardEvent` | Events (charge state, fan, protection flags, error flags) |
| `/robot/board/config` | `BoardConfig` | Config snapshot (all config params + switch states + log level) |
| `/robot/board/battery` | `sensor_msgs/BatteryState` | Battery state (voltage, current, SOC, cell voltage & temperature) |

### Services

| Service | Service Type | Description |
|---------|-------------|-------------|
| `/robot/board/query_config` | `BoardQueryConfig` | Query single config |
| `/robot/board/query_all_config` | `BoardQueryAllConfig` | Query all configs |
| `/robot/board/write_config` | `BoardWriteConfig` | Write config |
| `/robot/board/switch` | `BoardSwitch` | Switch operation (servo/5V/charge/battery output) |

### Verification

```bash
# List topics
ros2 topic list | grep robot

# List services
ros2 service list | grep robot

# Echo IMU data
ros2 topic echo /robot/board/imu

# Query config
ros2 service call /robot/board/query_all_config servo_robot_board_interface/srv/BoardQueryAllConfig
```

## Logging

Board logs are forwarded to the ROS2 logging system (`rosout`) via `DriverCallback::on_log`, using rclrs logging macros.

### View Logs

```bash
# Real-time view of all logs
ros2 topic echo /rosout

# Board logs only
ros2 topic echo /rosout | grep servo_robot_board
```

### Log Levels

| Level | Description |
|-------|-------------|
| `DEBUG` | Data publishing (IMU/Power/Battery/Thermal throttled to 1s) |
| `INFO` | Service calls (params & results), system events |
| `ERROR` | Publish failures, service execution failures |

### Set Log Level at Runtime

```bash
ros2 run servo-robot-bridge servo_robot_board_bridge --ros-args --log-level debug
```

## Architecture

```
┌─────────────────────────────────────────┐
│         ROS2 Bridge Node                │
│  ┌─────────────┐  ┌─────────────────┐  │
│  │  Publishers  │  │    Services     │  │
│  │  imu, power  │  │  query, write   │  │
│  │  thermal...  │  │  switch         │  │
│  └──────┬───────┘  └───────┬─────────┘  │
│         │                  │            │
│  ┌──────┴──────────────────┴──────┐     │
│  │         Driver (sync)          │     │
│  │  SerialTransport / Mock        │     │
│  └────────────────────────────────┘     │
└─────────────────────────────────────────┘
```

### Data Flow

```
Read Thread → DriverState (state.update_*) → Main Loop (state.snapshot) → publish_data()
                                                        ↓
Dispatch Thread → EventBus.dispatch()            ROS2 Topic Publishing (rosout log)
                        ↓
                BridgeCallback::on_log → ROS2 Logging System
```

- Main loop polls `state.snapshot()` every 50ms and publishes to ROS2 topics
- Service requests handled synchronously via `driver.query_config_sync()` / `driver.write_config_sync()`
- Board logs forwarded to ROS2 logging system via `DriverCallback::on_log`

## File Description

| File | Description | Committed |
|------|-------------|-----------|
| `Cargo.toml.template` | Dependency template | ✅ |
| `build.rs.template` | Build script template | ✅ |
| `config/servo_robot_bridge.yaml` | Parameter config file | ✅ |
| `launch/bridge.launch.py` | ROS2 launch file | ✅ |
| `Cargo.toml` | Actual dependencies (generated from template) | ❌ |
| `build.rs` | Actual build script (generated from template) | ❌ |
| `scripts/.env` | Environment variables | ❌ |

## Install ROS2 Rust

```bash
# 1. Install Rust(Optiolal)
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# 2. Clone ros2_rust
mkdir -p ~/ros_pkgs/ros2_rust_ws/src
cd ~/ros_pkgs/ros2_rust_ws/src
git clone https://github.com/ros2-rust/ros2_rust.git
vcs import < ros2_rust/ros2_rust_humble.repos

# 3. Build
cd ~/ros_pkgs/ros2_rust_ws
colcon build --packages-up-to rclrs rosidl_generator_rs rosidl_runtime_rs
```
