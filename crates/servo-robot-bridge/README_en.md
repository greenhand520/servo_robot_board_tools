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
# or colcon build --cmake-args -Wno-dev  
```

### 2. 安装 Servo Robot Board Interface

```bash
mkdir -p ~/ros_pkgs/servo_robot_board_ws/src
cd ~/ros_pkgs/servo_robot_board_ws/src
git clone https://github.com/greenhand520/servo_robot_board_interface.git

cd ~/ros_pkgs/servo_robot_board_ws/
colcon build
# or colcon build --cmake-args -Wno-dev  
```

### 2. Enable ROS2 Bridge

```bash
# Default paths
source scripts/init_workspace.sh --ros2_support

# Custom paths
source scripts/init_workspace.sh --ros2_support ~/ros_pkgs/ros2_rust_ws ~/ros_pkgs/servo_robot_board_ws
```

The script will:
- Generate `scripts/.env` environment variable file
- Generate `Cargo.toml` and `build.rs` from templates
- Enable the bridge crate in the workspace

### 3. Set IDE Environment Variables (Optional)

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
| `/robot/board/system` | `BoardSystem` | System info (device ID, uptime, error counts, PD voltage/current, firmware version) |
| `/robot/board/event` | `BoardEvent` | Events (charge state, state change flags, protection flags, error flags) |
| `/robot/board/config` | `BoardConfig` | Config snapshot (all config params + switch states + log level) |
| `/robot/board/battery` | `sensor_msgs/BatteryState` | Battery state (voltage, current, percentage, cell voltage & temperature) |
| `/robot/board/log` | `rcl_interfaces/Log` | Board logs (timestamp, level, file name, function name, message) |

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

Board logs are forwarded to two destinations via `DriverCallback::on_log`:
1. **ROS2 logging system** (`rosout`) - using rclrs logging macros
2. **`/robot/board/log` topic** - using `rcl_interfaces/msg/Log` message type for easy filtering and subscription

### View Logs

```bash
# Method 1: Via /robot/board/log topic (recommended)
ros2 topic echo /robot/board/log

# Method 2: Via rosout (includes all node logs)
ros2 topic echo /rosout | grep servo_robot_board
```

### Log Message Format

The `/robot/board/log` topic uses `rcl_interfaces/msg/Log` message type:

| Field | Type | Description |
|-------|------|-------------|
| `stamp` | `builtin_interfaces/Time` | Timestamp (seconds + nanoseconds) |
| `level` | `uint8` | Log level (DEBUG=10, INFO=20, WARN=30, ERROR=40) |
| `name` | `string` | Node name (fixed as `servo_robot_board`) |
| `msg` | `string` | Formatted log message `[HH:MM:SS.mmm] file::func: message` |
| `file` | `string` | Source file name |
| `function` | `string` | Function name |
| `line` | `uint32` | Line number (fixed as 0) |

### Set Log Level at Runtime

```bash
ros2 run servo-robot-bridge servo_robot_board_bridge --ros-args --log-level debug
```

## Data Flow

```
Read Thread → DriverState (state.update_*) → Main Loop (state.snapshot) → publish_data()
                                                        ↓
Dispatch Thread → EventBus.dispatch()            ROS2 Topic Publishing
                        ↓                              ↓
                BridgeCallback::on_log      /robot/board/log (rcl_interfaces/Log)
                        ↓                              ↓
                ROS2 Logging System          TUI / Other Subscribers
```

- Main loop polls `state.snapshot()` every 50ms and publishes to ROS2 topics
- Service requests handled synchronously via `driver.query_config_sync()` / `driver.write_config_sync()`
- Board logs forwarded to both ROS2 logging system and `/robot/board/log` topic

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



