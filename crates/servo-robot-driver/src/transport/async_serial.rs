//! 基于 tokio 的异步串口传输层实现

use crate::error::DriverError;
use crate::transport::async_trait::AsyncTransport;
use serialport::SerialPort;
use std::future::Future;
use std::io::{Read, Write};
use std::pin::Pin;
use std::sync::{Arc, Mutex};
use std::time::Duration;

/// 异步串口传输层
///
/// 使用 `tokio::task::spawn_blocking` 包装同步的 `serialport` crate
pub struct TokioSerialTransport {
    port: Arc<Mutex<Box<dyn SerialPort>>>,
}

impl TokioSerialTransport {
    /// 创建新的异步串口传输层
    pub fn new(port: Box<dyn SerialPort>) -> Self {
        TokioSerialTransport {
            port: Arc::new(Mutex::new(port)),
        }
    }

    /// 打开串口
    pub fn open(port_name: &str, baud_rate: u32) -> Result<Self, DriverError> {
        let port = serialport::new(port_name, baud_rate)
            .timeout(Duration::from_millis(100))
            .open()?;
        Ok(TokioSerialTransport {
            port: Arc::new(Mutex::new(port)),
        })
    }
}

impl AsyncTransport for TokioSerialTransport {
    fn read_frame(&mut self) -> Pin<Box<dyn Future<Output = Result<Vec<u8>, DriverError>> + Send + '_>> {
        let port = self.port.clone();

        Box::pin(async move {
            tokio::task::spawn_blocking(move || {
                let mut port = port.lock().map_err(|_| DriverError::LockPoisoned)?;

                // 读取帧头
                let mut header = [0u8; 1];
                loop {
                    match port.read_exact(&mut header) {
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
                port.read_exact(&mut type_buf)?;

                // 读取 LEN (2 bytes, little-endian)
                let mut len_buf = [0u8; 2];
                port.read_exact(&mut len_buf)?;
                let payload_len = u16::from_le_bytes(len_buf) as usize;

                // 读取 PAYLOAD
                let mut payload = vec![0u8; payload_len];
                port.read_exact(&mut payload)?;

                // 读取 CRC (2 bytes)
                let mut crc_buf = [0u8; 2];
                port.read_exact(&mut crc_buf)?;

                // 组装完整帧
                let mut frame = Vec::with_capacity(4 + payload_len + 2);
                frame.push(0xAA);
                frame.push(type_buf[0]);
                frame.extend_from_slice(&len_buf);
                frame.extend_from_slice(&payload);
                frame.extend_from_slice(&crc_buf);

                Ok(frame)
            })
            .await
            .map_err(|e| DriverError::Io(e.to_string()))?
        })
    }

    fn write_frame(&mut self, frame: &[u8]) -> Pin<Box<dyn Future<Output = Result<(), DriverError>> + Send + '_>> {
        let port = self.port.clone();
        let frame = frame.to_vec();

        Box::pin(async move {
            tokio::task::spawn_blocking(move || {
                let mut port = port.lock().map_err(|_| DriverError::LockPoisoned)?;
                port.write_all(&frame)?;
                port.flush()?;
                Ok(())
            })
            .await
            .map_err(|e| DriverError::Io(e.to_string()))?
        })
    }

    fn close(&mut self) -> Pin<Box<dyn Future<Output = Result<(), DriverError>> + Send + '_>> {
        Box::pin(async { Ok(()) })
    }
}
