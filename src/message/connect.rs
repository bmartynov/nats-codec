use std::io;

use bytes::BufMut;
use serde::{Deserialize, Serialize};

const HEADER: &'static [u8] = &[b'I', b'N', b'F', b'O', b' '];
const FOOTER: &'static [u8] = &[b'\r', b'\n'];

#[derive(Debug, Default, Clone, PartialEq, Serialize, Deserialize)]
pub struct Payload {
    pub verbose: bool,
    pub pedantic: bool,
    pub tls_required: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub auth_token: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub user: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub pass: Option<String>,
    pub lang: String,
    pub name: String,
    pub version: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub protocol: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sig: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub jwt: Option<String>,
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
