# servo-robot-tui

English | [简体中文](README.md)

A ratatui-based TUI for real-time monitoring of STM32 robot sensor data, system status, and device control.

![servo-robot-tui](assets/servo-robot-tui.png)

## Features

- Real-time line charts for servo current/power and battery current/charge power
- IMU attitude visualization (quaternion, acceleration, gyroscope)
- Battery state monitoring (SOC, cell voltage & temperature, charge/discharge power)
- STM32 system info (temperature, error counts, packet loss rate, uptime)
- Event status bar (charging, fan, protection flags, error flags)
- Interactive command bar (config query/edit)
- Board log viewer
- Supports Driver direct call or ROS2 data source

## Quick Start

```bash
# Run with MockTransport (no hardware needed)
cargo run -p servo-robot-tui

# Enable mock feature
cargo run -p servo-robot-tui --features mock

# Set log level
RUST_LOG=info cargo run -p servo-robot-tui
```

## Key Bindings

| Key | Function | Description |
|-----|----------|-------------|
| R | Reset device | Red (destructive) |
| S | Shutdown | Red (destructive) |
| 1 | Toggle servo power | Green=ON / Red=OFF |
| 2 | Toggle 5V power | Green=ON / Red=OFF |
| 3 | Toggle charge | Green=ON / Red=OFF |
| 4 | Toggle battery extra output | Green=ON / Red=OFF |
| Q | Query all configs | Blue |
| W | Open config editor | Blue |
| L | Cycle log level | - |
| F | Log focus mode | - |
| Esc | Exit / close popup | - |

### Data Flow

```
Driver/ROS2 → DataSource → App.update() → UI.render()
                ↓
           DataSnapshot (thread-safe snapshot)
                ↓
           ChartData (VecDeque ring buffer)
```

### Key Design Decisions

1. **Data Source Abstraction**: `DataSource` trait supports both Driver and ROS2 data sources
2. **State Snapshot**: `DataSnapshot` holds latest values of all sensor data
3. **Chart Data**: `ChartData` uses `VecDeque` to store the last N data points
4. **Color Thresholds**: Dynamic color adjustment based on config current/temperature limits
5. **Command Mode**: Supports confirm dialog, config query, and config editor modes

## Dependencies

- `ratatui` - TUI framework
- `crossterm` - Terminal control
- `servo-robot-driver` - Driver library

## License

GPL-3.0
