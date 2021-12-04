use std::convert::TryFrom;

#[derive(Debug, PartialEq)]
pub struct InvalidOp(pub String);

#[derive(Debug, PartialEq)]
pub enum Op {
    Ok,
    Err,
    Info,
    Ping,
    Pong,
    Publish,
    Connect,
    Message,
    Subscribe,
    Unsubscribe,
}

impl TryFrom<&[u8]> for Op {
    type Error = InvalidOp;

    #[inline]
    fn try_from(value: &[u8]) -> Result<Self, Self::Error> {
        Ok(match value {
            // hottest
            [b'p' | b'P', b'u' | b'U', b'b' | b'B'] => Op::Publish,
            [b'm' | b'M', b's' | b'S', b'g' | b'G'] => Op::Message,
            //
            [b's' | b'S', b'u' | b'U', b'b' | b'B'] => Op::Subscribe,
            [b'i' | b'I', b'n' | b'N', b'f' | b'F', b'o' | b'O'] => Op::Info,
            [b'p' | b'P', b'i' | b'I', b'n' | b'N', b'g' | b'G'] => Op::Ping,
            [b'p' | b'P', b'o' | b'O', b'n' | b'N', b'g' | b'G'] => Op::Pong,
            [b'u' | b'U', b'n' | b'N', b's' | b'S', b'u' | b'U', b'b' | b'B'] => Op::Unsubscribe,
            [b'c' | b'C', b'o' | b'O', b'n' | b'N', b'n' | b'N', b'e' | b'E', b'c' | b'C', b't' | b'T'] => {
                Op::Connect
            }

            [b'+', b'o' | b'O', b'k' | b'K'] => Op::Ok,
            [b'-', b'e' | b'E', b'r' | b'R', b'r' | b'R'] => Op::Err,
            _ => return Err(invalid_op(value)),
        })
    }
}

fn invalid_op(value: &[u8]) -> InvalidOp {
    let invalid_op = String::from_utf8_lossy(value).to_string();

    InvalidOp(invalid_op)
}


#[cfg(test)]
#[test]
fn test_op_try_from() {
    assert_eq!(Ok(Op::Ok), Op::try_from(&b"+ok"[..]));
    assert_eq!(Ok(Op::Ok), Op::try_from(&b"+OK"[..]));

    assert_eq!(Ok(Op::Err), Op::try_from(&b"-err"[..]));
    assert_eq!(Ok(Op::Err), Op::try_from(&b"-ERR"[..]));

    assert_eq!(Ok(Op::Info), Op::try_from(&b"info"[..]));
    assert_eq!(Ok(Op::Info), Op::try_from(&b"INFO"[..]));

    assert_eq!(Ok(Op::Ping), Op::try_from(&b"ping"[..]));
    assert_eq!(Ok(Op::Ping), Op::try_from(&b"PING"[..]));

    assert_eq!(Ok(Op::Pong), Op::try_from(&b"pong"[..]));
    assert_eq!(Ok(Op::Pong), Op::try_from(&b"PONG"[..]));

    assert_eq!(Ok(Op::Publish), Op::try_from(&b"pub"[..]));
    assert_eq!(Ok(Op::Publish), Op::try_from(&b"PUB"[..]));

    assert_eq!(Ok(Op::Connect), Op::try_from(&b"connect"[..]));
    assert_eq!(Ok(Op::Connect), Op::try_from(&b"CONNECT"[..]));

    assert_eq!(Ok(Op::Subscribe), Op::try_from(&b"sub"[..]));
    assert_eq!(Ok(Op::Subscribe), Op::try_from(&b"SUB"[..]));

    assert_eq!(Ok(Op::Unsubscribe), Op::try_from(&b"unsub"[..]));
    assert_eq!(Ok(Op::Unsubscribe), Op::try_from(&b"UNSUB"[..]));

    assert_eq!(Err(InvalidOp("invalid".into())), Op::try_from(&b"invalid"[..]));
}
