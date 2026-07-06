#!/bin/bash
# Enable ROS2 Bridge build
# Usage: Source script/enable-ros2-bridge.sh [ROS2_RUST_WS] [SERVO_ROBOT_BOARD_WS]

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_DIR="$(dirname "$SCRIPT_DIR")"

ROS2_RUST_WS="${1:-$HOME/ros_pkgs/ros2_rust_ws}"
SERVO_ROBOT_BOARD_WS="${2:-$HOME/ros_pkgs/servo_robot_board_ws}"

# generate .env to the scripts directory
AMENT_PATHS="/opt/ros/humble:$ROS2_RUST_WS/install:$SERVO_ROBOT_BOARD_WS/install"

# source ROS2 setup scripts to capture LD_LIBRARY_PATH
_LD_LIB="$(bash -c '. /opt/ros/humble/setup.sh && . '"$SERVO_ROBOT_BOARD_WS"'/install/setup.sh && . '"$ROS2_RUST_WS"'/install/setup.sh && echo "$LD_LIBRARY_PATH"')"

cat > "$SCRIPT_DIR/.env" << EOF
RUST_BACKTRACE=full
ROS_DISTRO=humble
AMENT_PREFIX_PATH=$AMENT_PATHS
CMAKE_PREFIX_PATH=$AMENT_PATHS
LD_LIBRARY_PATH=$_LD_LIB
EOF

# generate files from templates
# workspace
sed "s|\${ROS2_RUST_WS}|$ROS2_RUST_WS|g; s|\${SERVO_ROBOT_BOARD_WS}|$SERVO_ROBOT_BOARD_WS|g" \
    "$PROJECT_DIR/Cargo.toml.template" > "$PROJECT_DIR/Cargo.toml"

# bridge
sed "s|\${ROS2_RUST_WS}|$ROS2_RUST_WS|g; s|\${SERVO_ROBOT_BOARD_WS}|$SERVO_ROBOT_BOARD_WS|g" \
    "$PROJECT_DIR/crates/servo-robot-bridge/Cargo.toml.template" > "$PROJECT_DIR/crates/servo-robot-bridge/Cargo.toml"

sed "s|\${ROS2_RUST_WS}|$ROS2_RUST_WS|g; s|\${SERVO_ROBOT_BOARD_WS}|$SERVO_ROBOT_BOARD_WS|g" \
    "$PROJECT_DIR/crates/servo-robot-bridge/build.rs.template" > "$PROJECT_DIR/crates/servo-robot-bridge/build.rs"

# tui
sed "s|\${ROS2_RUST_WS}|$ROS2_RUST_WS|g; s|\${SERVO_ROBOT_BOARD_WS}|$SERVO_ROBOT_BOARD_WS|g" \
    "$PROJECT_DIR/crates/servo-robot-tui/Cargo.toml.template" > "$PROJECT_DIR/crates/servo-robot-tui/Cargo.toml"

# enable bridge crate
sed -i 's|# "crates/servo-robot-bridge",|"crates/servo-robot-bridge",|' "$PROJECT_DIR/Cargo.toml"

echo "✅ ROS2 Bridge Enabled"
echo "   ROS2_RUST_WS=$ROS2_RUST_WS"
echo "   SERVO_ROBOT_BOARD_WS=$SERVO_ROBOT_BOARD_WS"
echo "   .env file: $SCRIPT_DIR/.env"
