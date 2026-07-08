//! # Authors
//! greenhand520
//! # Since
//! version: 0.1.0
//! # Date
//! 2026/7/4 10:30

//!Data type conversion, include:
//! protocol::ImuData -> sensors_msg::Imu
//! protocol::BatteryState -> sensors_msg::BatteryState
//! other protocol data -> servo_robot_board_interface::msg::*
//!
//! Note: Protocol uses integer types (u16/i16) with scaling factors:
//! - PowerData: u16 values = actual * 10 (e.g., 866 = 86.6V)
//! - ThermalData: i16 values = actual * 10 (e.g., 571 = 57.1°C)
//! - BatteryState: voltage mV, current mA, capacity mAh, percentage 1~100, temp *10
//! - Config: u16 values, some *10 (e.g., charge_stop_voltage 168 = 16.8V)
//! ROS2 interface uses f32, so conversion is needed.

use geometry_msgs::msg as geo_msg;
use sensor_msgs::msg as sensor_msg;
use servo_robot_board_interface::msg as board_msg;
use servo_robot_driver::protocol::*;

pub fn convert_imu(data: &imu::ImuData) -> sensor_msg::Imu {
    sensor_msg::Imu {
        orientation: geo_msg::Quaternion {
            x: data.quaternion[1] as f64,
            y: data.quaternion[2] as f64,
            z: data.quaternion[3] as f64,
            w: data.quaternion[0] as f64,
        },
        angular_velocity: geo_msg::Vector3 {
            x: data.gyro[0] as f64,
            y: data.gyro[1] as f64,
            z: data.gyro[2] as f64,
        },
        linear_acceleration: geo_msg::Vector3 {
            x: data.accel[0] as f64,
            y: data.accel[1] as f64,
            z: data.accel[2] as f64,
        },
        ..Default::default()
    }
}

/// Convert PowerData to ROS2 message
/// Protocol: u16 values = actual * 10 (e.g., 866 = 86.6V)
/// ROS2: f32 values (actual)
pub fn convert_power(data: &power::PowerData) -> board_msg::BoardPower {
    board_msg::BoardPower {
        servo_voltage_mv: data.servo_voltage_mv,
        servo_current_ma: data.servo_current_ma,
        charge_in_voltage_mv: data.charge_in_voltage_mv,
        charge_in_current_ma: data.charge_in_current_ma,
        bat_voltage_mv: data.bat_voltage_mv,
        bat_current_ma: data.bat_current_ma,
    }
}

pub fn convert_system(data: &system::SystemInfo) -> board_msg::BoardSystem {
    board_msg::BoardSystem {
        device_id: data.device_id,
        uid: data.uid,
        imu_id: data.imu_id,
        uptime_s: data.uptime_s,
        cpu_usage_percent: data.cpu_usage_percent,
        free_heap_kb: data.free_heap_kb,
        stack_watermark_min_kb: data.stack_watermark_min_kb,
        i2c_error_count: data.i2c_error_count,
        spi_error_count: data.spi_error_count,
        uart_error_count: data.uart_error_count,
        usb_error_count: data.usb_error_count,
        frames_sent_total: data.frames_sent_total,
        pd_request_voltage_mv: data.pd_request_voltage_mv,
        pd_request_current_ma: data.pd_request_current_ma,
        firmware_version: format!("{}", data.firmware_version),
        // Convert SystemInfo thermal data to ROS2 message
        // Protocol: i16 values = actual * 10 (e.g., 571 = 57.1°C)
        // ROS2: f32 values (actual)
        // Thermal data is now part of SystemInfo
        temp_servo_power: data.temp_servo_power as f32 / 10.0,
        temp_5v_power: data.temp_5v_power as f32 / 10.0,
        temp_mcu: data.temp_mcu as f32 / 10.0,
        temp_charge: data.temp_charge as f32 / 10.0,
        temp_battery: data.temp_battery as f32 / 10.0,
    }
}

pub fn convert_event(data: &event::BoardEvent) -> board_msg::BoardEvent {
    board_msg::BoardEvent {
        charge_phase: data.charge_phase as u8,
        state_change_flags: data.state_change_flags.bits(),
        protection_flags: data.protection_flags.bits(),
        error_flags: data.error_flags.bits(),
    }
}

/// Convert BoardConfigSnapshot to ROS2 message
/// Protocol: u16 values, some scaled by *10
/// ROS2: f32 values (actual)
pub fn convert_config(data: &config::BoardConfigSnapshot) -> board_msg::BoardConfig {
    board_msg::BoardConfig {
        servo_current_limit_ma: data.servo_current_limit_ma,
        servo_temp_limit: data.servo_temp_limit as f32 / 10.0,
        temp_5v_limit: data.temp_5v_limit as f32 / 10.0,
        charge_max_current_ma: data.charge_max_current_ma,
        charge_temp_derating: data.charge_temp_derating as f32 / 10.0,
        charge_temp_limit: data.charge_temp_limit as f32 / 10.0,
        charge_stop_voltage_mv: data.charge_stop_voltage_mv,
        charge_stop_percentage: data.charge_stop_percentage,
        charge_enable: data.charge_enable,
        power_servo_on: data.power_servo_on,
        power_5v_on: data.power_5v_on,
        charge_on: data.charge_on,
        bat_ext_out_on: data.bat_ext_out_on,
        reset: false,
        shutdown: false,
        tx_log_level: data.tx_log_level as u8,
    }
}

/// Convert BatteryState to ROS2 message
/// Protocol: voltage mV, current mA, capacity mAh, percentage 1~100, temp *10
/// ROS2: voltage V, current A, capacity Ah, percentage 0~1, temp °C
pub fn convert_battery(data: &battery_state::BatteryState) -> sensor_msg::BatteryState {
    sensor_msg::BatteryState {
        voltage: data.voltage_mv as f32 / 1000.0,
        current: data.current_ma as f32 / 1000.0,
        charge: data.capacity_mah as f32 * data.percentage as f32 / 100.0 / 1000.0,
        capacity: data.capacity_mah as f32 / 1000.0,
        design_capacity: data.design_capacity_mah as f32 / 1000.0,
        percentage: data.percentage as f32 / 100.0,
        temperature: data.temperature as f32 / 10.0,
        power_supply_status: data.charge_status as u8,
        power_supply_health: data.health as u8,
        power_supply_technology: data.technology as u8,
        present: data.present,
        location: data.serial_number.to_string(),
        cell_voltage: data
            .cell_voltages_mv
            .iter()
            .map(|v| *v as f32 / 1000.0)
            .collect(),
        cell_temperature: data
            .cell_temperatures
            .iter()
            .map(|t| *t as f32 / 10.0)
            .collect(),
        ..Default::default()
    }
}
