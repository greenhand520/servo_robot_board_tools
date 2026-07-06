//! ROS2 服务处理

use servo_robot_driver::Driver;
use servo_robot_driver::protocol::config::{Config, ConfigType};
use servo_robot_board_interface::srv::*;
use std::sync::{Arc, Mutex};

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
                    value: config.value(),
                    msg: String::new(),
                },
                Err(e) => BoardQueryConfig_Response {
                    success: false,
                    value: 0.0,
                    msg: format!("Query failed: {}", e),
                },
            }
        }
        None => BoardQueryConfig_Response {
            success: false,
            value: 0.0,
            msg: format!("Invalid config type: 0x{:02X}", req.config_type),
        },
    }
}

pub fn handle_query_all_config(
    driver: &Arc<Mutex<Driver>>,
) -> BoardQueryAllConfig_Response {
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
            let config = Config::from_type_value(ct, req.value);
            let driver = driver.lock().unwrap();
            match driver.write_config_sync(config) {
                Ok(success) => BoardWriteConfig_Response {
                    success,
                    msg: if success { String::new() } else { "Write failed".to_string() },
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
        0x10 => Config::SwitchServoPower(req.enable),
        0x11 => Config::Switch5VPower(req.enable),
        0x12 => Config::SwitchCharge(req.enable),
        0x13 => Config::SwitchBatExtOut(req.enable),
        _ => {
            return BoardSwitch_Response {
                success: false,
                msg: format!("Invalid switch type: 0x{:02X}", req.switch_type),
            }
        }
    };
    let driver = driver.lock().unwrap();
    match driver.write_config_sync(config) {
        Ok(success) => BoardSwitch_Response {
            success,
            msg: if success { String::new() } else { "Switch failed".to_string() },
        },
        Err(e) => BoardSwitch_Response {
            success: false,
            msg: format!("Switch failed: {}", e),
        },
    }
}
