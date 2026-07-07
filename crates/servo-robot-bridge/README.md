# servo-robot-bridge

[English](README_en.md) | 简体中文

ROS2 桥接节点，将 servo-robot-driver 的数据发布为 ROS2 话题，提供 ROS2 服务控制设备。

## 快速开始

### 1. 安装 ROS2 Rust

```bash
# 1. 安装 Rust(可选)
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# 2. 克隆 ros2_rust
mkdir -p ~/ros_pkgs/ros2_rust_ws/src
cd ~/ros_pkgs/ros2_rust_ws/src
git clone https://github.com/ros2-rust/ros2_rust.git
vcs import < ros2_rust/ros2_rust_humble.repos

# 3. 构建
cd ~/ros_pkgs/ros2_rust_ws
colcon build
```

### 2. 启用 ROS2 Bridge

```bash
# 使用默认路径
source scripts/init_workspace.sh --ros2_support

# 或指定路径
source scripts/init_workspace.sh --ros2_support ~/ros_pkgs/ros2_rust_ws ~/ros_pkgs/servo_robot_board_ws
```

脚本会：
- 生成 `scripts/.env` 环境变量文件
- 从模板生成 `Cargo.toml` 和 `build.rs`
- 启用 workspace 中的 bridge crate

### 3. 设置 IDE 环境变量

**RustRover / CLion 用户**：需要将 `scripts/.env` 中的变量添加到 Rust 环境变量：

1. 打开 **设置 → Rust → 环境变量**
2. 添加以下变量：
   - `ROS_DISTRO=humble`
   - `RUST_BACKTRACE=full`
   - `AMENT_PREFIX_PATH=...`（从 `scripts/.env` 复制）
   - `LD_LIBRARY_PATH=...`（从 `scripts/.env` 复制）

### 4. 构建和运行

```bash
# 构建（连接真实串口）
cargo build -p servo-robot-bridge

# 构建（使用 MockTransport，无需硬件）
cargo build -p servo-robot-bridge --features mock

# 运行（真实串口）
cargo run -p servo-robot-bridge

# 运行（MockTransport，无需硬件即可开发测试）
cargo run -p servo-robot-bridge --features mock

# 运行（指定串口参数）
cargo run -p servo-robot-bridge -- --ros-args -p port:=/dev/ttyUSB1 -p baud_rate:=921600

# 使用 launch 文件运行
ros2 launch crates/servo-robot-bridge/launch/bridge.launch.py

# 使用 launch 文件并覆盖参数
ros2 launch crates/servo-robot-bridge/launch/bridge.launch.py port:=/dev/ttyUSB1 baud_rate:=921600
```

> **Mock 模式**：启用 `mock` feature 后，驱动使用 `MockTransport` 模拟 STM32 数据（IMU 100Hz、Battery 10Hz 等），无需连接硬件即可开发和测试 ROS2 话题/服务。

## 参数

| 参数 | 类型 | 默认值 | 说明 |
|------|------|--------|------|
| `port` | `string` | `/dev/ttyUSB0` | 串口设备路径 |
| `baud_rate` | `int` | `115200` | 串口波特率 |

### 参数设置方式

#### 1. 命令行

```bash
ros2 run servo-robot-bridge servo_robot_board_bridge --ros-args -p port:=/dev/ttyUSB1 -p baud_rate:=921600
```

#### 2. YAML 参数文件

编辑 `config/servo_robot_bridge.yaml`：

```yaml
servo_robot_board_bridge:
  ros__parameters:
    port: "/dev/ttyUSB0"
    baud_rate: 115200
```

然后通过 launch 文件加载：

```bash
ros2 launch crates/servo-robot-bridge/launch/bridge.launch.py
```

#### 3. 运行时查看和修改

```bash
# 查看所有参数
ros2 param list servo_robot_board_bridge

# 查看参数值
ros2 param get servo_robot_board_bridge port
ros2 param get servo_robot_board_bridge baud_rate
```

## 话题和服务

### 话题发布

| 话题 | 消息类型 | 说明 |
|------|---------|------|
| `/robot/board/imu` | `sensor_msgs/Imu` | IMU 数据（四元数、角速度、加速度） |
| `/robot/board/power` | `BoardPower` | 电源数据（舵机/充电/电池电压电流） |
| `/robot/board/thermal` | `BoardThermal` | 温度数据（舵机/5V/MCU/充电/电池） |
| `/robot/board/system` | `BoardSystem` | 系统信息（设备ID、运行时间、错误计数、PD电压电流、固件版本） |
| `/robot/board/event` | `BoardEvent` | 事件（充电状态、状态变化标志、保护标志、错误标志） |
| `/robot/board/config` | `BoardConfig` | 配置快照（所有配置参数 + 开关状态 + 日志等级） |
| `/robot/board/battery` | `sensor_msgs/BatteryState` | 电池状态（电压、电流、电量百分比、电芯电压温度） |
| `/robot/board/log` | `rcl_interfaces/Log` | 板级日志（时间戳、等级、文件名、函数名、消息内容） |

### 服务

| 服务 | 服务类型 | 说明 |
|------|---------|------|
| `/robot/board/query_config` | `BoardQueryConfig` | 查询单个配置 |
| `/robot/board/query_all_config` | `BoardQueryAllConfig` | 查询所有配置 |
| `/robot/board/write_config` | `BoardWriteConfig` | 写入配置 |
| `/robot/board/switch` | `BoardSwitch` | 开关操作（舵机/5V/充电/电池输出） |

### 验证

```bash
# 查看话题
ros2 topic list | grep robot

# 查看服务
ros2 service list | grep robot

# 监听 IMU 数据
ros2 topic echo /robot/board/imu

# 查询配置
ros2 service call /robot/board/query_all_config servo_robot_board_interface/srv/BoardQueryAllConfig
```

## 日志

板级日志通过 `DriverCallback::on_log` 转发到两个地方：
1. **ROS2 日志系统**（`rosout`）- 使用 rclrs 日志宏输出
2. **`/robot/board/log` 话题** - 使用 `rcl_interfaces/msg/Log` 消息类型，便于过滤和订阅

### 查看日志

```bash
# 方式1：通过 /robot/board/log 话题（推荐）
ros2 topic echo /robot/board/log

# 方式2：通过 rosout（包含所有节点日志）
ros2 topic echo /rosout | grep servo_robot_board
```

### 日志消息格式

`/robot/board/log` 话题使用 `rcl_interfaces/msg/Log` 消息类型：

| 字段 | 类型 | 说明 |
|------|------|------|
| `stamp` | `builtin_interfaces/Time` | 时间戳（秒 + 纳秒） |
| `level` | `uint8` | 日志等级（DEBUG=10, INFO=20, WARN=30, ERROR=40） |
| `name` | `string` | 节点名称（固定为 `servo_robot_board`） |
| `msg` | `string` | 格式化的日志消息 `[HH:MM:SS.mmm] file::func: message` |
| `file` | `string` | 源文件名 |
| `function` | `string` | 函数名 |
| `line` | `uint32` | 行号（固定为 0） |

### 运行时设置日志等级

```bash
ros2 run servo-robot-bridge servo_robot_board_bridge --ros-args --log-level debug
```

## 数据流

```
读线程 → DriverState (state.update_*) → 主循环 (state.snapshot) → publish_data()
                                                    ↓
分发线程 → EventBus.dispatch()              ROS2 话题发布
                    ↓                              ↓
            BridgeCallback::on_log      /robot/board/log (rcl_interfaces/Log)
                    ↓                              ↓
            ROS2 日志系统 (rosout)        TUI / 其他订阅者
```

- 主循环每 50ms 轮询 `state.snapshot()` 获取最新数据并发布到 ROS2 话题
- 服务请求通过 `driver.query_config_sync()` / `driver.write_config_sync()` 同步处理
- 板级日志同时转发到 ROS2 日志系统和 `/robot/board/log` 话题

## 文件说明

| 文件 | 说明 | 是否提交 |
|------|------|---------|
| `Cargo.toml.template` | 依赖模板 | ✅ |
| `build.rs.template` | 构建脚本模板 | ✅ |
| `config/servo_robot_bridge.yaml` | 参数配置文件 | ✅ |
| `launch/bridge.launch.py` | ROS2 launch 文件 | ✅ |
| `Cargo.toml` | 实际依赖（从模板生成） | ❌ |
| `build.rs` | 实际构建脚本（从模板生成） | ❌ |
| `scripts/.env` | 环境变量 | ❌ |
