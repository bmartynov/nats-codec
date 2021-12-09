use std::io;

use bytes::{BufMut, Bytes};

pub mod connect;
pub mod info;
pub mod message;
pub mod op;
pub mod publish;
pub mod subscribe;
pub mod unsubscribe;
pub mod error;

#[derive(Debug, Clone, PartialEq)]
pub enum Message {
    Ok,
    Err(error::Payload),
    Ping,
    Pong,
    Info(info::Payload),
    Connect(connect::Payload),
    Message(message::Payload),
    Publish(publish::Payload),
    Subscribe(subscribe::Payload),
    Unsubscribe(unsubscribe::Payload),
}

impl Message {
    pub fn encode(&self, dst: &mut bytes::BytesMut) -> Result<(), io::Error> {
        match self {
            Self::Ok => {
                dst.put_slice(&b"+OK\r\n"[..]);
            }
            Self::Err(p) => p.encode(dst)?,
            Self::Ping => {
                dst.put_slice(&b"PING\r\n"[..]);
            }
            Self::Pong => {
                dst.put_slice(&b"PONG\r\n"[..]);
            }
            Self::Info(p) => p.encode(dst)?,
            Self::Connect(p) => p.encode(dst)?,
            Self::Message(p) => p.encode(dst)?,
            Self::Publish(p) => p.encode(dst)?,
            Self::Subscribe(p) => p.encode(dst)?,
            Self::Unsubscribe(p) => p.encode(dst)?,
        };

        Ok(())
    }
}

impl Message {
    #[inline]
    pub fn with_body(&self) -> bool {
        matches!(self, Self::Message(_) | Self::Publish(_))
    }

    #[inline]
    pub fn body_size(&self) -> Option<usize> {
        Some(match self {
            Message::Message(m) => m.payload_size,
            Message::Publish(m) => m.payload_size,
            _ => return None,
        })
    }

    #[inline]
    pub fn set_body(&mut self, body: Bytes) {
        match self {
            Message::Message(m) => m.payload = Some(body),
            Message::Publish(m) => m.payload = Some(body),
            _ => unreachable!("bodyless"),
        }
    }
}
