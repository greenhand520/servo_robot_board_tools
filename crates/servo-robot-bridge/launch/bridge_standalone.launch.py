"""
独立启动文件（无需 ROS2 workspace 安装）

使用方式:
  ros2 launch /path/to/bridge_standalone.launch.py
  ros2 launch /path/to/bridge_standalone.launch.py port:=/dev/ttyUSB1 baud_rate:=921600
"""
import os
from launch import LaunchDescription
from launch.actions import DeclareLaunchArgument
from launch.substitutions import LaunchConfiguration
from launch_ros.actions import Node


def generate_launch_description():
    config_file = os.path.join(
        os.path.dirname(os.path.dirname(os.path.abspath(__file__))),
        'config',
        'servo_robot_bridge.yaml'
    )

    return LaunchDescription([
        DeclareLaunchArgument(
            'port',
            default_value='/dev/ttyUSB0',
            description='串口设备路径'
        ),
        DeclareLaunchArgument(
            'baud_rate',
            default_value='115200',
            description='串口波特率'
        ),
        Node(
            package='servo_robot_bridge',
            executable='servo_robot_board_bridge',
            name='servo_robot_board_bridge',
            output='screen',
            parameters=[
                config_file,
                {'port': LaunchConfiguration('port')},
                {'baud_rate': LaunchConfiguration('baud_rate')},
            ],
            arguments=['--ros-args', '--log-level', 'info'],
        ),
    ])
