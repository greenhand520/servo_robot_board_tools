//! 帧格式定义

use crate::error::FrameError;
use alloc::vec::Vec;

use crate::battery_state::BatteryState;
use crate::config::{BoardConfigSnapshot, Config, ConfigType};
use crate::event::BoardEvent;
use crate::imu::ImuData;
use crate::power::PowerData;
use crate::system::SystemInfo;
use crate::thermal::ThermalData;
/// 帧头
pub const FRAME_HEAD: u8 = 0xAA;

/// 帧头长度 (HEAD + TYPE + LEN)
const FRAME_HEADER_SIZE: usize = 4;
/// CRC 长度
const FRAME_CRC_SIZE: usize = 2;

/// 帧类型枚举
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum FrameType {
    // ═══ 上行 (STM32 → PC) ═══
    Imu = 0x01,
    Power = 0x02,
    Thermal = 0x03,
    Config = 0x04,
    Battery = 0x05,
    System = 0x06,
    Event = 0x07,

    // ═══ 下行 (PC → STM32) ═══
    CfgWrite = 0x80,
    CfgQuery = 0x81,
    CfgQueryAll = 0x82,

    // ═══ 应答 (STM32 → PC) ═══
    AckCfgWrite = 0xC0,
    AckCfgQuery = 0xC1,
    AckCfgQueryAll = 0xC2,

    // ═══ 未知类型 ═══
    Unknown(u8),
}

impl FrameType {
    pub fn from_u8(v: u8) -> Self {
        match v {
            0x01 => Self::Imu,
            0x02 => Self::Power,
            0x03 => Self::Thermal,
            0x04 => Self::Config,
            0x05 => Self::Battery,
            0x06 => Self::System,
            0x07 => Self::Event,
            0x80 => Self::CfgWrite,
            0x81 => Self::CfgQuery,
            0x82 => Self::CfgQueryAll,
            0xC0 => Self::AckCfgWrite,
            0xC1 => Self::AckCfgQuery,
            0xC2 => Self::AckCfgQueryAll,
            _ => Self::Unknown(v),
        }
    }

    pub fn as_u8(&self) -> u8 {
        match self {
            Self::Imu => 0x01,
            Self::Power => 0x02,
            Self::Thermal => 0x03,
            Self::Config => 0x04,
            Self::Battery => 0x05,
            Self::System => 0x06,
            Self::Event => 0x07,
            Self::CfgWrite => 0x80,
            Self::CfgQuery => 0x81,
            Self::CfgQueryAll => 0x82,
            Self::AckCfgWrite => 0xC0,
            Self::AckCfgQuery => 0xC1,
            Self::AckCfgQueryAll => 0xC2,
            Self::Unknown(v) => *v,
        }
    }

    pub fn is_uplink(&self) -> bool {
        matches!(
            self,
            Self::Imu
                | Self::Power
                | Self::Thermal
                | Self::Config
                | Self::Battery
                | Self::System
                | Self::Event
        )
    }

    pub fn is_downlink(&self) -> bool {
        matches!(self, Self::CfgWrite | Self::CfgQuery | Self::CfgQueryAll)
    }

    pub fn is_response(&self) -> bool {
        matches!(
            self,
            Self::AckCfgWrite | Self::AckCfgQuery | Self::AckCfgQueryAll
        )
    }

    pub fn name(&self) -> &'static str {
        match self {
            Self::Imu => "IMU",
            Self::Power => "Power",
            Self::Thermal => "Thermal",
            Self::Config => "Config",
            Self::Battery => "Battery",
            Self::System => "System",
            Self::Event => "Event",
            Self::CfgWrite => "CfgWrite",
            Self::CfgQuery => "CfgQuery",
            Self::CfgQueryAll => "CfgQueryAll",
            Self::AckCfgWrite => "AckCfgWrite",
            Self::AckCfgQuery => "AckCfgQuery",
            Self::AckCfgQueryAll => "AckCfgQueryAll",
            Self::Unknown(_) => "Unknown",
        }
    }
}

impl core::fmt::Display for FrameType {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::Unknown(v) => write!(f, "Unknown({:#04x})", v),
            _ => write!(f, "{}", self.name()),
        }
    }
}

/// 低级帧结构
#[derive(Debug, Clone)]
pub struct RawFrame {
    pub frame_type: FrameType,
    pub payload: Vec<u8>,
}

impl RawFrame {
    /// 从字节缓冲区解码一帧
    pub fn decode(buf: &[u8]) -> Result<(Self, usize), FrameError> {
        let header_pos = buf
            .iter()
            .position(|&b| b == FRAME_HEAD)
            .ok_or(FrameError::NoHeader)?;

        if buf.len() - header_pos < FRAME_HEADER_SIZE {
            return Err(FrameError::Incomplete {
                needed: FRAME_HEADER_SIZE - (buf.len() - header_pos),
            });
        }

        let frame_type = FrameType::from_u8(buf[header_pos + 1]);
        let payload_len = u16::from_le_bytes([buf[header_pos + 2], buf[header_pos + 3]]) as usize;
        let total_len = FRAME_HEADER_SIZE + payload_len + FRAME_CRC_SIZE;

        if buf.len() - header_pos < total_len {
            return Err(FrameError::Incomplete {
                needed: total_len - (buf.len() - header_pos),
            });
        }

        let payload_start = header_pos + FRAME_HEADER_SIZE;
        let payload_end = payload_start + payload_len;
        let payload = buf[payload_start..payload_end].to_vec();

        let crc_start = payload_end;
        let received_crc = u16::from_le_bytes([buf[crc_start], buf[crc_start + 1]]);

        let crc_data = &buf[header_pos + 1..payload_end];
        let calculated_crc = crate::crc::crc16_ccitt_table(crc_data);

        if received_crc != calculated_crc {
            return Err(FrameError::CrcMismatch {
                expected: calculated_crc,
                got: received_crc,
            });
        }

        Ok((
            RawFrame {
                frame_type,
                payload,
            },
            header_pos + total_len,
        ))
    }

    /// 编码为字节
    pub fn encode(&self) -> Vec<u8> {
        let payload_len = self.payload.len();
        let total_len = FRAME_HEADER_SIZE + payload_len + FRAME_CRC_SIZE;
        let mut buf = Vec::with_capacity(total_len);

        buf.push(FRAME_HEAD);
        buf.push(self.frame_type.as_u8());

        let len_bytes = (payload_len as u16).to_le_bytes();
        buf.push(len_bytes[0]);
        buf.push(len_bytes[1]);

        buf.extend_from_slice(&self.payload);

        let crc_data = &buf[1..];
        let crc = crate::crc::crc16_ccitt_table(crc_data);
        let crc_bytes = crc.to_le_bytes();
        buf.push(crc_bytes[0]);
        buf.push(crc_bytes[1]);

        buf
    }
}

/// 可序列化为帧 payload 的类型
pub trait ToPayload {
    fn to_payload(&self) -> Vec<u8>;
}

/// 可从帧 payload 反序列化的类型
pub trait FromPayload: Sized {
    fn from_payload(payload: &[u8]) -> Result<Self, FrameError>;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_frame_type_from_u8() {
        assert_eq!(FrameType::from_u8(0x01), FrameType::Imu);
        assert_eq!(FrameType::from_u8(0x80), FrameType::CfgWrite);
        assert_eq!(FrameType::from_u8(0xC0), FrameType::AckCfgWrite);
        assert_eq!(FrameType::from_u8(0xFF), FrameType::Unknown(0xFF));
    }

    #[test]
    fn test_raw_frame_encode_decode() {
        let frame = RawFrame {
            frame_type: FrameType::Imu,
            payload: vec![0x01, 0x02, 0x03, 0x04],
        };
        let encoded = frame.encode();
        let (decoded, consumed) = RawFrame::decode(&encoded).unwrap();
        assert_eq!(consumed, encoded.len());
        assert_eq!(decoded.frame_type, frame.frame_type);
        assert_eq!(decoded.payload, frame.payload);
    }
}

/// 类型化帧枚举
#[derive(Debug, Clone)]
pub enum TypedFrame {
    // 上行帧
    Imu(ImuData),
    Power(PowerData),
    Thermal(ThermalData),
    Config(BoardConfigSnapshot),
    Battery(BatteryState),
    System(SystemInfo),
    Event(BoardEvent),

    // 应答帧
    AckCfgWrite { success: bool },
    AckCfgQuery(Config),
    AckCfgQueryAll(BoardConfigSnapshot),

    // 下行帧（用于发送）
    ConfigWrite(Config),
    ConfigQuery(ConfigType),
    ConfigQueryAll,
}

impl TypedFrame {
    pub fn from_raw(frame: &RawFrame) -> Result<Self, FrameError> {
        match frame.frame_type {
            FrameType::Imu => Ok(TypedFrame::Imu(ImuData::from_bytes(&frame.payload)?)),
            FrameType::Power => Ok(TypedFrame::Power(PowerData::from_bytes(&frame.payload)?)),
            FrameType::Thermal => Ok(TypedFrame::Thermal(ThermalData::from_bytes(
                &frame.payload,
            )?)),
            FrameType::Config => Ok(TypedFrame::Config(BoardConfigSnapshot::from_bytes(
                &frame.payload,
            )?)),
            FrameType::Battery => Ok(TypedFrame::Battery(BatteryState::from_bytes(
                &frame.payload,
            )?)),
            FrameType::System => Ok(TypedFrame::System(SystemInfo::from_bytes(&frame.payload)?)),
            FrameType::Event => Ok(TypedFrame::Event(BoardEvent::from_bytes(&frame.payload)?)),
            FrameType::AckCfgWrite => {
                if frame.payload.is_empty() {
                    return Err(FrameError::PayloadTooShort {
                        expected: 1,
                        got: 0,
                    });
                }
                Ok(TypedFrame::AckCfgWrite {
                    success: frame.payload[0] != 0,
                })
            }
            FrameType::AckCfgQuery => {
                Ok(TypedFrame::AckCfgQuery(Config::from_bytes(&frame.payload)?))
            }
            FrameType::AckCfgQueryAll => Ok(TypedFrame::AckCfgQueryAll(
                BoardConfigSnapshot::from_bytes(&frame.payload)?,
            )),
            FrameType::Unknown(_v) => Err(FrameError::PayloadDecode("Unknown frame type")),
            _ => Err(FrameError::PayloadDecode("Unexpected frame type")),
        }
    }

    pub fn frame_type(&self) -> FrameType {
        match self {
            TypedFrame::Imu(_) => FrameType::Imu,
            TypedFrame::Power(_) => FrameType::Power,
            TypedFrame::Thermal(_) => FrameType::Thermal,
            TypedFrame::Config(_) => FrameType::Config,
            TypedFrame::Battery(_) => FrameType::Battery,
            TypedFrame::System(_) => FrameType::System,
            TypedFrame::Event(_) => FrameType::Event,
            TypedFrame::AckCfgWrite { .. } => FrameType::AckCfgWrite,
            TypedFrame::AckCfgQuery(_) => FrameType::AckCfgQuery,
            TypedFrame::AckCfgQueryAll(_) => FrameType::AckCfgQueryAll,
            TypedFrame::ConfigWrite(_) => FrameType::CfgWrite,
            TypedFrame::ConfigQuery(_) => FrameType::CfgQuery,
            TypedFrame::ConfigQueryAll => FrameType::CfgQueryAll,
        }
    }
}

impl RawFrame {
    /// 解析为类型化帧
    pub fn parse_typed(&self) -> Result<TypedFrame, FrameError> {
        TypedFrame::from_raw(self)
    }
}
