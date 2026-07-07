#!/bin/bash
# Initialize workspace
#
# Usage:
#   source scripts/init_workspace.sh                              # basic workspace (no ROS2)
#   source scripts/init_workspace.sh --ros2_support               # enable ROS2 with default paths
#   source scripts/init_workspace.sh --ros2_support [ROS2_RUST_WS] [SERVO_ROBOT_BOARD_WS]

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_DIR="$(dirname "$SCRIPT_DIR")"

if [[ "$1" == "--ros2_support" ]]; then
    # ═══ ROS2 support mode ═══
    ROS2_RUST_WS="${2:-$HOME/ros_pkgs/ros2_rust_ws}"
    SERVO_ROBOT_BOARD_WS="${3:-$HOME/ros_pkgs/servo_robot_board_ws}"

    # source ROS2 setup scripts 并提取环境变量
    # AMENT_PREFIX_PATH 需要精确到 install/package 级别，必须从 setup.sh 输出中提取
    _ENV_OUTPUT="$(bash -c '
        . /opt/ros/humble/setup.sh
        . '"$SERVO_ROBOT_BOARD_WS"'/install/setup.sh
        . '"$ROS2_RUST_WS"'/install/setup.sh
        echo "AMENT_PREFIX_PATH=$AMENT_PREFIX_PATH"
        echo "CMAKE_PREFIX_PATH=$CMAKE_PREFIX_PATH"
        echo "LD_LIBRARY_PATH=$LD_LIBRARY_PATH"
    ')"

    _AMENT="$(echo "$_ENV_OUTPUT" | grep "^AMENT_PREFIX_PATH=" | cut -d= -f2-)"
    _CMAKE="$(echo "$_ENV_OUTPUT" | grep "^CMAKE_PREFIX_PATH=" | cut -d= -f2-)"
    _LD_LIB="$(echo "$_ENV_OUTPUT" | grep "^LD_LIBRARY_PATH=" | cut -d= -f2-)"

    cat > "$SCRIPT_DIR/.env" << EOF
RUST_BACKTRACE=full
ROS_DISTRO=humble
AMENT_PREFIX_PATH=$_AMENT
CMAKE_PREFIX_PATH=$_CMAKE
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

    sed "s|\${ROS2_RUST_WS}|$ROS2_RUST_WS|g; s|\${SERVO_ROBOT_BOARD_WS}|$SERVO_ROBOT_BOARD_WS|g" \
        "$PROJECT_DIR/crates/servo-robot-tui/build.rs.template" > "$PROJECT_DIR/crates/servo-robot-tui/build.rs"

    # enable bridge crate
    sed -i 's|# "crates/servo-robot-bridge",|"crates/servo-robot-bridge",|' "$PROJECT_DIR/Cargo.toml"

    echo "✅ Workspace initialized with ROS2 support"
    echo "   ROS2_RUST_WS=$ROS2_RUST_WS"
    echo "   SERVO_ROBOT_BOARD_WS=$SERVO_ROBOT_BOARD_WS"
    echo "   .env file: $SCRIPT_DIR/.env"

else
    # ═══ Basic mode (no ROS2) ═══
    # copy Cargo.toml.template content before [patch.crates-io]
    sed '/^\[patch.crates-io\]/,$d' "$PROJECT_DIR/Cargo.toml.template" > "$PROJECT_DIR/Cargo.toml"

    # generate tui Cargo.toml without ROS2 optional dependencies
    # strip ROS2 optional deps and comment, replace ros2 feature with empty
    sed '/rclrs.*optional/d; /sensor_msgs.*optional/d; /servo_robot_board_interface.*optional/d; /^# ROS2/d; s/^ros2 = .*/ros2 = []/' \
        "$PROJECT_DIR/crates/servo-robot-tui/Cargo.toml.template" > "$PROJECT_DIR/crates/servo-robot-tui/Cargo.toml"

    echo "✅ Workspace initialized (without ROS2 support)"
    echo "   To enable ROS2: source scripts/init_workspace.sh --ros2_support"
fi
