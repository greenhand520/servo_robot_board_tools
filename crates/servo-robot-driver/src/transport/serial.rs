//! 基于 serialport crate 的传输层实现

use crate::error::DriverError;
use crate::transport::Transport;
use serialport::SerialPort;
use std::io::Read;
use std::io::Write;
use std::time::Duration;

/// 串口传输层实现
pub struct SerialTransport {
    port: Box<dyn SerialPort>,
}

impl SerialTransport {
    /// 创建新的串口传输层
    pub fn new(port: Box<dyn SerialPort>) -> Self {
        SerialTransport { port }
    }

    /// 打开串口
    pub fn open(port_name: &str, baud_rate: u32) -> Result<Self, DriverError> {
        let port = serialport::new(port_name, baud_rate)
            .timeout(Duration::from_millis(100))
            .open()?;
        Ok(SerialTransport { port })
    }
}

impl Transport for SerialTransport {
    fn read_frame(&mut self) -> Result<Vec<u8>, DriverError> {
        // 读取帧头
        let mut header = [0u8; 1];
        loop {
            match self.port.read_exact(&mut header) {
                Ok(()) => {
                    if header[0] == 0xAA {
                        break;
                    }
                }
                Err(e) => {
                    if e.kind() == std::io::ErrorKind::TimedOut {
                        continue;
                    }
                    return Err(DriverError::Io(e.to_string()));
                }
            }
        }

        // 读取 TYPE
        let mut type_buf = [0u8; 1];
        self.port.read_exact(&mut type_buf)?;

        // 读取 LEN (2 bytes, little-endian)
        let mut len_buf = [0u8; 2];
        self.port.read_exact(&mut len_buf)?;
        let payload_len = u16::from_le_bytes(len_buf) as usize;

        // 读取 PAYLOAD
        let mut payload = vec![0u8; payload_len];
        self.port.read_exact(&mut payload)?;

        // 读取 CRC (2 bytes)
        let mut crc_buf = [0u8; 2];
        self.port.read_exact(&mut crc_buf)?;

        // 组装完整帧
        let mut frame = Vec::with_capacity(4 + payload_len + 2);
        frame.push(0xAA);
        frame.push(type_buf[0]);
        frame.extend_from_slice(&len_buf);
        frame.extend_from_slice(&payload);
        frame.extend_from_slice(&crc_buf);

        Ok(frame)
    }

    fn write_frame(&mut self, frame: &[u8]) -> Result<(), DriverError> {
        self.port.write_all(frame)?;
        self.port.flush()?;
        Ok(())
    }

    fn close(&mut self) -> Result<(), DriverError> {
        // serialport crate 会在 Drop 时自动关闭
        Ok(())
    }
}
