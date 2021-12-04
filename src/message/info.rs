use std::io;

use bytes::BufMut;
use serde::{Deserialize, Serialize};

const HEADER: &'static [u8] = &[b'I', b'N', b'F', b'O', b' '];
const FOOTER: &'static [u8] = &[b'\r', b'\n'];

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Payload {
    pub server_id: String,
    pub version: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub proto: Option<usize>,
    pub go: String,
    pub host: String,
    pub port: u64,
    #[serde(default)]
    pub auth_required: bool,
    #[serde(default)]
    pub tls_required: bool,
    #[serde(default)]
    pub max_payload: u64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub client_id: Option<usize>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub connect_urls: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub nonce: Option<String>,
}

impl Default for Payload {
    fn default() -> Self {
        Self {
            server_id: Default::default(),
            version: Default::default(),
            proto: Default::default(),
            go: Default::default(),
            host: Default::default(),
            port: Default::default(),
            auth_required: Default::default(),
            tls_required: Default::default(),
            max_payload: 1024,
            client_id: Default::default(),
            connect_urls: Default::default(),
            nonce: Default::default(),
        }
    }
}

impl Payload {
    pub fn encode(&self, dst: &mut bytes::BytesMut) -> Result<(), io::Error> {
        dst.put_slice(HEADER);

        serde_json::to_writer(dst.writer(), self).map_err(|_| {
            let kind = io::ErrorKind::Other;

            io::Error::new(kind, "cannot encode json")
        })?;

        dst.put_slice(FOOTER);

        Ok(())
    }
}

#[test]
fn test_payload() {
    use bytes::BytesMut;

    let mut input = BytesMut::new();

    let payload = Payload::default();

    payload.encode(&mut input).expect("ok");

    println!("{:?}", input);
}
