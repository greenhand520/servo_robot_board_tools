//! Pattern B: 闭包注册存储

use crate::error::DriverError;
use crate::protocol::battery_state::BatteryState;
use crate::protocol::config::BoardConfigSnapshot;
use crate::protocol::event::BoardEvent;
use crate::protocol::imu::ImuData;
use crate::protocol::power::PowerData;
use crate::protocol::thermal::ThermalData;
use crate::protocol::system::SystemInfo;

/// 闭包存储
pub struct ClosureStore {
    imu_closures: Vec<Box<dyn FnMut(&ImuData) + Send>>,
    power_closures: Vec<Box<dyn FnMut(&PowerData) + Send>>,
    thermal_closures: Vec<Box<dyn FnMut(&ThermalData) + Send>>,
    battery_closures: Vec<Box<dyn FnMut(&BatteryState) + Send>>,
    config_snapshot_closures: Vec<Box<dyn FnMut(&BoardConfigSnapshot) + Send>>,
    board_event_closures: Vec<Box<dyn FnMut(&BoardEvent) + Send>>,
    system_info_closures: Vec<Box<dyn FnMut(&SystemInfo) + Send>>,
    error_closures: Vec<Box<dyn FnMut(&DriverError) + Send>>,
}

impl ClosureStore {
    pub fn new() -> Self {
        ClosureStore {
            imu_closures: Vec::new(),
            power_closures: Vec::new(),
            thermal_closures: Vec::new(),
            battery_closures: Vec::new(),
            config_snapshot_closures: Vec::new(),
            board_event_closures: Vec::new(),
            system_info_closures: Vec::new(),
            error_closures: Vec::new(),
        }
    }

    pub fn on_imu_data(&mut self, f: impl FnMut(&ImuData) + Send + 'static) {
        self.imu_closures.push(Box::new(f));
    }

    pub fn on_power_data(&mut self, f: impl FnMut(&PowerData) + Send + 'static) {
        self.power_closures.push(Box::new(f));
    }

    pub fn on_thermal_data(&mut self, f: impl FnMut(&ThermalData) + Send + 'static) {
        self.thermal_closures.push(Box::new(f));
    }

    pub fn on_battery_state(&mut self, f: impl FnMut(&BatteryState) + Send + 'static) {
        self.battery_closures.push(Box::new(f));
    }

    pub fn on_config_snapshot(
        &mut self,
        f: impl FnMut(&BoardConfigSnapshot) + Send + 'static,
    ) {
        self.config_snapshot_closures.push(Box::new(f));
    }

    pub fn on_board_event(&mut self, f: impl FnMut(&BoardEvent) + Send + 'static) {
        self.board_event_closures.push(Box::new(f));
    }

    pub fn on_system_info(&mut self, f: impl FnMut(&SystemInfo) + Send + 'static) {
        self.system_info_closures.push(Box::new(f));
    }

    pub fn on_error(&mut self, f: impl FnMut(&DriverError) + Send + 'static) {
        self.error_closures.push(Box::new(f));
    }

    pub fn call_imu(&mut self, data: &ImuData) {
        for f in self.imu_closures.iter_mut() {
            f(data);
        }
    }

    pub fn call_power(&mut self, data: &PowerData) {
        for f in self.power_closures.iter_mut() {
            f(data);
        }
    }

    pub fn call_thermal(&mut self, data: &ThermalData) {
        for f in self.thermal_closures.iter_mut() {
            f(data);
        }
    }

    pub fn call_battery(&mut self, state: &BatteryState) {
        for f in self.battery_closures.iter_mut() {
            f(state);
        }
    }

    pub fn call_config_snapshot(&mut self, config: &BoardConfigSnapshot) {
        for f in self.config_snapshot_closures.iter_mut() {
            f(config);
        }
    }

    pub fn call_board_event(&mut self, event: &BoardEvent) {
        for f in self.board_event_closures.iter_mut() {
            f(event);
        }
    }

    pub fn call_system_info(&mut self, info: &SystemInfo) {
        for f in self.system_info_closures.iter_mut() {
            f(info);
        }
    }

    pub fn call_error(&mut self, error: &DriverError) {
        for f in self.error_closures.iter_mut() {
            f(error);
        }
    }
}
