use std::io;

use bytes::{BufMut, Bytes};

const HEADER: &'static [u8] = &[b'M', b'S', b'G', b' '];
const CLRF: &'static [u8] = &[b'\r', b'\n'];

#[derive(Debug, Clone, Default, PartialEq)]
pub struct Payload {
    pub subject: Bytes,
    pub sid: usize,
    pub reply_to: Option<Bytes>,
    pub payload_size: usize,
    pub payload: Option<Bytes>,
}

// MSG <subject> <sid> [reply-to] <#bytes>\r\n[payload]

impl Payload {
    pub fn encode(&self, dst: &mut bytes::BytesMut) -> Result<(), io::Error> {
        dst.put_slice(HEADER);
        dst.put_slice(&self.subject);

        dst.put_slice(&b" "[..]);
        dst.put_slice(&self.sid.to_string().as_bytes());

        if let Some(reply_to) = &self.reply_to {
            dst.put_slice(&b" "[..]);
            dst.put_slice(reply_to);
        };

        dst.put_slice(&b" "[..]);
        dst.put_slice(&self.payload_size.to_string().as_bytes());

        dst.put_slice(CLRF);

        if let Some(payload) = &self.payload {
            dst.put_slice(payload);
        }
        dst.put_slice(CLRF);

        Ok(())
    }
}
