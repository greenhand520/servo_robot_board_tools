//! 系统信息类型

use crate::error::FrameError;
use crate::enum_from_u8;
use crate::frame::{FromPayload, ToPayload};
use alloc::vec::Vec;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
#[repr(u8)]
pub enum ResetReason {
    #[default]
    Unknown = 0,
    Watchdog = 1,
    WindowWdg = 2,
    Software = 3,
    PowerOn = 4,
    PinReset = 5,
    BrownOut = 6,
}

enum_from_u8!(ResetReason, Unknown, Watchdog=1, WindowWdg=2, Software=3, PowerOn=4, PinReset=5, BrownOut=6);

/// 系统信息
#[derive(Debug, Clone, Default)]
pub struct SystemInfo {
    pub device_id: u16,
    pub uid: u32,
    pub imu_id: u8,
    pub uptime_s: u32,
    pub reset_reason: ResetReason,
    pub error_code: u8,
    pub cpu_usage_percent: u8,
    pub free_heap_kb: u16,
    pub stack_watermark_min_kb: u16,
    pub i2c_error_count: u16,
    pub uart_error_count: u16,
    pub frames_sent_total: u32,
    pub pd_request_voltage: u16,
    pub pd_request_current: u16,
}

impl SystemInfo {
    pub fn from_bytes(data: &[u8]) -> Result<Self, FrameError> {
        if data.len() < 30 {
            return Err(FrameError::PayloadTooShort { expected: 30, got: data.len() });
        }
        let mut o = 0;
        let device_id = u16::from_le_bytes([data[o], data[o+1]]); o += 2;
        let uid = u32::from_le_bytes([data[o], data[o+1], data[o+2], data[o+3]]); o += 4;
        let imu_id = data[o]; o += 1;
        let uptime_s = u32::from_le_bytes([data[o], data[o+1], data[o+2], data[o+3]]); o += 4;
        let reset_reason = ResetReason::from_u8(data[o]); o += 1;
        let error_code = data[o]; o += 1;
        let cpu_usage_percent = data[o]; o += 1;
        let free_heap_kb = u16::from_le_bytes([data[o], data[o+1]]); o += 2;
        let stack_watermark_min_kb = u16::from_le_bytes([data[o], data[o+1]]); o += 2;
        let i2c_error_count = u16::from_le_bytes([data[o], data[o+1]]); o += 2;
        let uart_error_count = u16::from_le_bytes([data[o], data[o+1]]); o += 2;
        let frames_sent_total = u32::from_le_bytes([data[o], data[o+1], data[o+2], data[o+3]]); o += 4;
        let pd_request_voltage = u16::from_le_bytes([data[o], data[o+1]]); o += 2;
        let pd_request_current = u16::from_le_bytes([data[o], data[o+1]]);
        Ok(SystemInfo { device_id, uid, imu_id, uptime_s, reset_reason, error_code, cpu_usage_percent, free_heap_kb, stack_watermark_min_kb, i2c_error_count, uart_error_count, frames_sent_total, pd_request_voltage, pd_request_current })
    }

    pub fn to_bytes(&self) -> Vec<u8> {
        let mut buf = Vec::with_capacity(30);
        buf.extend_from_slice(&self.device_id.to_le_bytes());
        buf.extend_from_slice(&self.uid.to_le_bytes());
        buf.push(self.imu_id);
        buf.extend_from_slice(&self.uptime_s.to_le_bytes());
        buf.push(self.reset_reason as u8);
        buf.push(self.error_code);
        buf.push(self.cpu_usage_percent);
        buf.extend_from_slice(&self.free_heap_kb.to_le_bytes());
        buf.extend_from_slice(&self.stack_watermark_min_kb.to_le_bytes());
        buf.extend_from_slice(&self.i2c_error_count.to_le_bytes());
        buf.extend_from_slice(&self.uart_error_count.to_le_bytes());
        buf.extend_from_slice(&self.frames_sent_total.to_le_bytes());
        buf.extend_from_slice(&self.pd_request_voltage.to_le_bytes());
        buf.extend_from_slice(&self.pd_request_current.to_le_bytes());
        buf
    }
}

impl ToPayload for SystemInfo { fn to_payload(&self) -> Vec<u8> { self.to_bytes() } }
impl FromPayload for SystemInfo { fn from_payload(p: &[u8]) -> Result<Self, FrameError> { Self::from_bytes(p) } }
