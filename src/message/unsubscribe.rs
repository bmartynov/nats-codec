use std::io;

use bytes::BufMut;

const HEADER: &'static [u8] = &[b'U', b'N', b'S', b'U', b'B', b' '];
const CLRF: &'static [u8] = &[b'\r', b'\n'];

#[derive(Debug, Clone, Default, PartialEq)]
pub struct Payload {
    pub sid: usize,
    pub max_messages: Option<usize>,
}

// UNSUB <sid> [max_msgs]

impl Payload {
    pub fn encode(&self, dst: &mut bytes::BytesMut) -> Result<(), io::Error> {
        dst.put_slice(HEADER);

        dst.put_slice(&b" "[..]);
        dst.put_slice(&self.sid.to_string().as_bytes());

        if let Some(max_messages) = &self.max_messages {
            dst.put_slice(&b" "[..]);
            dst.put_slice(max_messages.to_string().as_bytes());
        };

        dst.put_slice(CLRF);

        Ok(())
    }
}
