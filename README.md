# servo_robot_board_tools

Rust workspace for communicating with an STM32-based robot power/servo management board over UART. Provides a binary protocol, serial driver, ratatui TUI for real-time monitoring, and a ROS2 bridge node.

## Overview

```
┌─────────────────────────────────────────────────────────────┐
│                        User Code                             │
│         ┌──────────────┐        ┌──────────────┐            │
│         │  servo-robot  │        │  servo-robot  │            │
│         │    -tui       │        │   -bridge     │            │
│         │  (ratatui)    │        │   (ROS2)      │            │
│         └──────┬───────┘        └──────┬───────┘            │
│                │                       │                     │
│         ┌──────┴───────────────────────┴───────┐            │
│         │          servo-robot-driver           │            │
│         │   (Transport / EventBus / DriverState) │            │
│         └──────────────────┬───────────────────┘            │
│                            │                                 │
│         ┌──────────────────┴───────────────────┐            │
│         │       servo-robot-protocol            │            │
│         │   (no_std frame/data types/CRC)       │            │
│         └──────────────────────────────────────┘            │
└─────────────────────────────────────────────────────────────┘
                              │
                         UART (115200)
                              │
                    ┌─────────┴─────────┐
                    │  STM32 Power/Servo │
                    │  Management Board  │
                    └───────────────────┘
```

## Crates

| Crate | Type | Description |
|-------|------|-------------|
| [`servo-robot-protocol`](crates/servo-robot-protocol) | Library | `no_std` protocol: frame format, CRC-16/CCITT, data types |
| [`servo-robot-driver`](crates/servo-robot-driver) | Library | Serial driver with EventBus, DriverState, sync/async, MockTransport |
| [`servo-robot-tui`](crates/servo-robot-tui) | Binary | ratatui TUI with charts, IMU visualization, battery monitor, log viewer |
| [`servo-robot-bridge`](crates/servo-robot-bridge) | Binary | ROS2 bridge: 7 topics, 4 services, board log forwarding |

## Screenshot

![servo-robot-tui](crates/servo-robot-tui/assets/servo-robot-tui.png)

## Quick Start

```bash
# Build all crates
cargo build

# Run TUI with mock data (no hardware needed)
cargo run -p servo-robot-tui

# Run all tests
cargo test
```

For more details on each crate, see their respective README:

- [servo-robot-protocol](crates/servo-robot-protocol/README.md) — protocol format, data types, CRC
- [servo-robot-driver](crates/servo-robot-driver/README.md) — driver architecture, callback API, reconnection
- [servo-robot-tui](crates/servo-robot-tui/README.md) — key bindings, UI layout, data source
- [servo-robot-bridge](crates/servo-robot-bridge/README.md) — ROS2 topics, services, parameters, launch files

## Feature Flags

| Flag | Crate | Effect |
|------|-------|--------|
| `mock` | driver, bridge, tui | Enable MockTransport (simulated sensor data, no hardware needed) |
| `async` | driver | Enable tokio-based AsyncDriver |
| `ros2` | tui | Enable ROS2 data source |
| `std` | protocol | Enable std (default) |
| `embedded` | protocol | no_std mode for STM32 |

## Conventions

- **Rust edition**: 2024
- **Commit style**: `feat:`, `fix:`, `update:` prefixes

## License

GPL-3.0
