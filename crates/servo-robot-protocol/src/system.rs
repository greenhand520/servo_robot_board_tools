//! # Authors
//! greenhand520
//! # Since
//! version: 0.1.0
//! # Date
//! 2026/7/3 11:37

use crate::error::FrameError;
use crate::frame::{FromPayload, ToPayload};
use alloc::vec::Vec;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Version {
    pub major: u8,
    pub minor: u8,
    pub patch: u8,
}

impl Version {
    pub const fn new(major: u8, minor: u8, patch: u8) -> Self {
        Self { major, minor, patch }
    }
}

impl core::fmt::Display for Version {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "{}.{}.{}", self.major, self.minor, self.patch)
    }
}

/// 系统信息
#[derive(Debug, Clone)]
pub struct SystemInfo {
    // stm32设备id
    pub device_id: u16,
    // stm32全球唯一id
    pub uid: u32,
    // imu的id
    pub imu_id: u8,
    // 运行时间
    pub uptime_s: u32,
    // cpu使用率
    pub cpu_usage_percent: u8,
    pub free_heap_kb: u16,
    // 系统启动以来栈剩余可用空间的最小值
    pub stack_watermark_min_kb: u16,
    pub i2c_error_count: u16,
    pub spi_error_count: u16,
    pub uart_error_count: u16,
    pub usb_error_count: u16,
    // 总共发送的帧
    pub frames_sent_total: u32,
    // pd握手协议电压
    pub pd_request_voltage: u16,
    // pd握手协议电流
    pub pd_request_current: u16,
    /// 固件版本
    pub firmware_version: Version,
}

impl Default for SystemInfo {
    fn default() -> Self {
        Self {
            device_id: 0,
            uid: 0,
            imu_id: 0,
            uptime_s: 0,
            cpu_usage_percent: 0,
            free_heap_kb: 0,
            stack_watermark_min_kb: 0,
            i2c_error_count: 0,
            spi_error_count: 0,
            uart_error_count: 0,
            usb_error_count: 0,
            frames_sent_total: 0,
            pd_request_voltage: 0,
            pd_request_current: 0,
            firmware_version: Version::new(0, 1, 0),
        }
    }
}

impl SystemInfo {
    pub fn from_bytes(data: &[u8]) -> Result<Self, FrameError> {
        if data.len() < 31 {
            return Err(FrameError::PayloadTooShort {
                expected: 31,
                got: data.len(),
            });
        }
        let mut o = 0;
        let device_id = u16::from_le_bytes([data[o], data[o + 1]]);
        o += 2;
        let uid = u32::from_le_bytes([data[o], data[o + 1], data[o + 2], data[o + 3]]);
        o += 4;
        let imu_id = data[o];
        o += 1;
        let uptime_s = u32::from_le_bytes([data[o], data[o + 1], data[o + 2], data[o + 3]]);
        o += 4;
        let cpu_usage_percent = data[o];
        o += 1;
        let free_heap_kb = u16::from_le_bytes([data[o], data[o + 1]]);
        o += 2;
        let stack_watermark_min_kb = u16::from_le_bytes([data[o], data[o + 1]]);
        o += 2;
        let i2c_error_count = u16::from_le_bytes([data[o], data[o + 1]]);
        o += 2;
        let spi_error_count = u16::from_le_bytes([data[o], data[o + 1]]);
        o += 2;
        let uart_error_count = u16::from_le_bytes([data[o], data[o + 1]]);
        o += 2;
        let usb_error_count = u16::from_le_bytes([data[o], data[o + 1]]);
        o += 2;
        let frames_sent_total =
            u32::from_le_bytes([data[o], data[o + 1], data[o + 2], data[o + 3]]);
        o += 4;
        let pd_request_voltage = u16::from_le_bytes([data[o], data[o + 1]]);
        o += 2;
        let pd_request_current = u16::from_le_bytes([data[o], data[o + 1]]);
        o += 2;
        let version = Version::new(data[o], data[o + 1], data[o + 2]);
        Ok(SystemInfo {
            device_id,
            uid,
            imu_id,
            uptime_s,
            cpu_usage_percent,
            free_heap_kb,
            stack_watermark_min_kb,
            i2c_error_count,
            spi_error_count,
            uart_error_count,
            usb_error_count,
            frames_sent_total,
            pd_request_voltage,
            pd_request_current,
            firmware_version: version,
        })
    }

    pub fn to_bytes(&self) -> Vec<u8> {
        let mut buf = Vec::with_capacity(31);
        buf.extend_from_slice(&self.device_id.to_le_bytes());
        buf.extend_from_slice(&self.uid.to_le_bytes());
        buf.push(self.imu_id);
        buf.extend_from_slice(&self.uptime_s.to_le_bytes());
        buf.push(self.cpu_usage_percent);
        buf.extend_from_slice(&self.free_heap_kb.to_le_bytes());
        buf.extend_from_slice(&self.stack_watermark_min_kb.to_le_bytes());
        buf.extend_from_slice(&self.i2c_error_count.to_le_bytes());
        buf.extend_from_slice(&self.spi_error_count.to_le_bytes());
        buf.extend_from_slice(&self.uart_error_count.to_le_bytes());
        buf.extend_from_slice(&self.usb_error_count.to_le_bytes());
        buf.extend_from_slice(&self.frames_sent_total.to_le_bytes());
        buf.extend_from_slice(&self.pd_request_voltage.to_le_bytes());
        buf.extend_from_slice(&self.pd_request_current.to_le_bytes());
        buf.push(self.firmware_version.major);
        buf.push(self.firmware_version.minor);
        buf.push(self.firmware_version.patch);
        buf
    }
}

impl ToPayload for SystemInfo {
    fn to_payload(&self) -> Vec<u8> {
        self.to_bytes()
    }
}
impl FromPayload for SystemInfo {
    fn from_payload(p: &[u8]) -> Result<Self, FrameError> {
        Self::from_bytes(p)
    }
}

impl core::fmt::Display for SystemInfo {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(
            f,
            "V{} up={}s CPU={}% heap={}KB stack={}KB frames={} errs(i={},s={},u={},usb={}) PD={}V/{}A",
            self.firmware_version,
            self.uptime_s,
            self.cpu_usage_percent,
            self.free_heap_kb,
            self.stack_watermark_min_kb,
            self.frames_sent_total,
            self.i2c_error_count,
            self.spi_error_count,
            self.uart_error_count,
            self.usb_error_count,
            self.pd_request_voltage / 1000,
            self.pd_request_current / 1000,
        )
    }
}
