use std::io;

use bytes::{BufMut, Bytes};

const HEADER: &'static [u8] = &[b'S', b'U', b'B', b' '];
const CLRF: &'static [u8] = &[b'\r', b'\n'];

#[derive(Debug, Clone, Default, PartialEq)]
pub struct Payload {
    pub subject: Bytes,
    pub sid: usize,
    pub queue_group: Option<Bytes>,
}

// SUB <subject> [queue group] <sid>

impl Payload {
    pub fn encode(&self, dst: &mut bytes::BytesMut) -> Result<(), io::Error> {
        dst.put_slice(HEADER);
        dst.put_slice(&self.subject);

        if let Some(queue_group) = &self.queue_group {
            dst.put_slice(&b" "[..]);
            dst.put_slice(queue_group);
        };

        dst.put_slice(&b" "[..]);
        dst.put_slice(&self.sid.to_string().as_bytes());

        dst.put_slice(CLRF);

        Ok(())
    }
}
