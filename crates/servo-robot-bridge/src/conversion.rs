//! # Authors
//! greenhand520
//! # Since
//! version: 0.1.0
//! # Date
//! 2026/7/4 10:30
//! ROS2 bridge node
//!Data type conversion, include:
//! protocol::ImuData -> sensors_msg::Imu
//! protocol::BatteryState -> sensors_msg::BatteryState
//! other protocol data -> servo_robot_board_interface::msg::*

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

pub fn convert_power(data: &power::PowerData) -> board_msg::BoardPower {
    board_msg::BoardPower {
        servo_voltage: data.servo_voltage,
        servo_current: data.servo_current,
        charge_in_voltage: data.charge_in_voltage,
        charge_in_current: data.charge_in_current,
        bat_voltage: data.bat_voltage,
        bat_current: data.bat_current,
    }
}

pub fn convert_thermal(data: &thermal::ThermalData) -> board_msg::BoardThermal {
    board_msg::BoardThermal {
        temp_servo_power: data.temp_servo_power,
        temp_5v_power: data.temp_5v_power,
        temp_mcu: data.temp_mcu,
        temp_charge: data.temp_charge,
        temp_battery: data.temp_battery,
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
        pd_request_voltage: data.pd_request_voltage,
        pd_request_current: data.pd_request_current,
        firmware_version: format!("{}", data.firmware_version),
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

pub fn convert_config(data: &config::BoardConfigSnapshot) -> board_msg::BoardConfig {
    board_msg::BoardConfig {
        servo_current_limit: data.servo_current_limit,
        servo_temp_limit: data.servo_temp_limit,
        temp_5v_limit: data.temp_5v_limit,
        charge_max_current: data.charge_max_current,
        charge_temp_derating: data.charge_temp_derating,
        charge_temp_limit: data.charge_temp_limit,
        charge_stop_voltage: data.charge_stop_voltage,
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

pub fn convert_battery(data: &battery_state::BatteryState) -> sensor_msg::BatteryState {
    sensor_msg::BatteryState {
        voltage: data.voltage,
        current: data.current,
        charge: data.capacity * data.percentage,
        capacity: data.capacity,
        design_capacity: data.design_capacity,
        percentage: data.percentage,
        temperature: data.temperature,
        power_supply_status: data.charge_status as u8,
        power_supply_health: data.health as u8,
        power_supply_technology: data.technology as u8,
        present: data.present,
        location: data.serial_number.to_string(),
        cell_voltage: data.cell_voltages.clone(),
        cell_temperature: data.cell_temperatures.clone(),
        ..Default::default()
    }
}
