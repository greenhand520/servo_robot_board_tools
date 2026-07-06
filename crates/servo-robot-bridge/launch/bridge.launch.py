import os
from launch import LaunchDescription
from launch_ros.actions import Node
from ament_index_python.packages import get_package_share_directory


def generate_launch_description():
    # 参数文件路径（默认在 config/ 目录下）
    config_file = os.path.join(
        os.path.dirname(os.path.dirname(__file__)),
        'config',
        'servo_robot_bridge.yaml'
    )

    return LaunchDescription([
        Node(
            package='servo_robot_bridge',
            executable='servo_robot_board_bridge',
            name='servo_robot_board_bridge',
            output='screen',
            parameters=[config_file],
            arguments=[
                '--ros-args',
                '--log-level', 'info',
            ],
        ),
    ])
