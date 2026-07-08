//! # Authors
//! greenhand520
//! # Since
//! version: 0.1.0
//! # Date
//! 2026/7/4 10:30
//!
//! ROS2 service processing

use std::sync::{Arc, Mutex};

use servo_robot_board_interface::srv::*;
use servo_robot_driver::Driver;
use servo_robot_driver::protocol::config::{Config, ConfigType};
use servo_robot_driver::protocol::servo::ServoCmdWrapper;

pub fn handle_query_config(
    driver: &Arc<Mutex<Driver>>,
    req: BoardQueryConfig_Request,
) -> BoardQueryConfig_Response {
    let config_type = ConfigType::from_u8(req.config_type);
    match config_type {
        Some(ct) => {
            let driver = driver.lock().unwrap();
            match driver.query_config_sync(ct) {
                Ok(config) => BoardQueryConfig_Response {
                    success: true,
                    value: config.value() as i16,
                    msg: String::new(),
                },
                Err(e) => BoardQueryConfig_Response {
                    success: false,
                    value: 0,
                    msg: format!("Query failed: {}", e),
                },
            }
        }
        None => BoardQueryConfig_Response {
            success: false,
            value: 0,
            msg: format!("Invalid config type: 0x{:02X}", req.config_type),
        },
    }
}

pub fn handle_query_all_config(driver: &Arc<Mutex<Driver>>) -> BoardQueryAllConfig_Response {
    let driver = driver.lock().unwrap();
    match driver.query_all_configs_sync() {
        Ok(config) => BoardQueryAllConfig_Response {
            success: true,
            config: super::conversion::convert_config(&config),
            msg: String::new(),
        },
        Err(e) => BoardQueryAllConfig_Response {
            success: false,
            config: servo_robot_board_interface::msg::BoardConfig::default(),
            msg: format!("Query all failed: {}", e),
        },
    }
}

pub fn handle_write_config(
    driver: &Arc<Mutex<Driver>>,
    req: BoardWriteConfig_Request,
) -> BoardWriteConfig_Response {
    let config_type = ConfigType::from_u8(req.config_type);
    match config_type {
        Some(ct) => {
            let config = Config::from_type_value(ct, req.value as u16);
            let driver = driver.lock().unwrap();
            match driver.write_config_sync(config) {
                Ok(success) => BoardWriteConfig_Response {
                    success,
                    msg: if success {
                        String::new()
                    } else {
                        "Write failed".to_string()
                    },
                },
                Err(e) => BoardWriteConfig_Response {
                    success: false,
                    msg: format!("Write failed: {}", e),
                },
            }
        }
        None => BoardWriteConfig_Response {
            success: false,
            msg: format!("Invalid config type: 0x{:02X}", req.config_type),
        },
    }
}

pub fn handle_switch(
    driver: &Arc<Mutex<Driver>>,
    req: BoardSwitch_Request,
) -> BoardSwitch_Response {
    let config = match req.switch_type {
        0x10 => Config::SwitchPowerServo(req.enable),
        0x11 => Config::SwitchPower5V(req.enable),
        0x12 => Config::SwitchCharge(req.enable),
        0x13 => Config::SwitchBatExtOut(req.enable),
        _ => {
            return BoardSwitch_Response {
                success: false,
                msg: format!("Invalid switch type: 0x{:02X}", req.switch_type),
            };
        }
    };
    let driver = driver.lock().unwrap();
    match driver.write_config_sync(config) {
        Ok(success) => BoardSwitch_Response {
            success,
            msg: if success {
                String::new()
            } else {
                "Switch failed".to_string()
            },
        },
        Err(e) => BoardSwitch_Response {
            success: false,
            msg: format!("Switch failed: {}", e),
        },
    }
}

pub fn handle_servo_forward(
    driver: &Arc<Mutex<Driver>>,
    req: ServoForward_Request,
) -> ServoForward_Response {
    let cmd = ServoCmdWrapper::new(req.command);
    let driver = driver.lock().unwrap();
    match driver.forward_servo_sync(&cmd) {
        Ok(response) => ServoForward_Response {
            success: true,
            response: response.data().to_vec(),
            msg: String::new(),
        },
        Err(e) => ServoForward_Response {
            success: false,
            response: vec![],
            msg: format!("Servo forward failed: {}", e),
        },
    }
}
