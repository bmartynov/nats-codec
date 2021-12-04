use atoi::FromRadix10;
use bytes::{Buf, Bytes, BytesMut};

use nom::combinator::opt;
use nom::sequence::{delimited, preceded, terminated, tuple};
use nom::Needed;

use super::message;
use super::message::{op::Op, Message};

pub type ParseResult<O> = Result<O, nom::Err<nom::error::Error<Bytes>>>;

#[inline]
pub fn parse(input: Bytes) -> ParseResult<Message> {
    let (input, op) = op(input)?;

    let message = match op {
        // hottest
        Op::Publish => parse_publish(input)?,
        Op::Message => parse_message(input)?,
        //
        Op::Ok => Message::Ok,
        Op::Err => parse_error(input)?,
        Op::Info => parse_info(input)?,
        Op::Ping => Message::Ping,
        Op::Pong => Message::Pong,
        Op::Connect => parse_connect(input)?,
        Op::Subscribe => parse_subscribe(input)?,
        Op::Unsubscribe => parse_unsubscribe(input)?,
    };

    Ok(message)
}

#[inline]
fn op(input: Bytes) -> ParseResult<(Bytes, Op)> {
    let (input, value) = ident1(input)?;

    let op = Op::try_from(&value[..]).map_err(|_| {
        let code = nom::error::ErrorKind::Tag;

        nom::Err::Error(nom::error::Error::new(input.clone(), code))
    })?;

    Ok((input, op))
}

#[inline]
fn parse_error(input: Bytes) -> ParseResult<Message> {
    use message::error::Payload;

    let cond =
        |c| matches!(c, b'a'..=b'z' | b'A'..=b'Z' | b'0'..=b'9' | b'.'| b'*'| b'_' | b'-' | b' ');

    let (_, raw) = preceded(
        space1,
        delimited(
            // '
            tag_u8(b'\''),
            // error message
            take_while1(cond),
            // '
            tag_u8(b'\''),
        ),
    )(input)?;

    let err: Payload = (&raw[..]).into();

    Ok(Message::Err(err))
}

#[inline]
fn parse_info(input: Bytes) -> ParseResult<Message> {
    use message::info::Payload;

    let (input, _) = space1(input)?;

    let payload = serde_json::from_slice::<Payload>(&input).map_err(|_| {
        let code = nom::error::ErrorKind::Fail;

        nom::Err::Error(nom::error::Error::new(input.clone(), code))
    })?;

    Ok(Message::Info(payload))
}

#[inline]
fn parse_publish(input: Bytes) -> ParseResult<Message> {
    use message::publish::Payload;

    let (_, (subject, reply_to, payload_size)) = preceded(
        space1,
        tuple((
            // subject
            terminated(ident1, space1),
            // reply_to
            opt(terminated(ident1, space1)),
            // payload_size
            digit1,
        )),
    )(input)?;

    Ok(Message::Publish(Payload {
        subject,
        reply_to,
        payload_size,
        payload: None,
    }))
}

#[inline]
fn parse_connect(input: Bytes) -> ParseResult<Message> {
    use message::connect::Payload;

    let (input, _) = space1(input)?;

    let payload = serde_json::from_slice::<Payload>(&input).map_err(|_| {
        let code = nom::error::ErrorKind::Fail;

        nom::Err::Error(nom::error::Error::new(input.clone(), code))
    })?;

    Ok(Message::Connect(payload))
}

#[inline]
fn parse_message(input: Bytes) -> ParseResult<Message> {
    use message::message::Payload;

    let (_, (subject, sid, reply_to, payload_size)) = preceded(
        space1,
        tuple((
            // subject
            terminated(ident1, space1),
            // sid
            terminated(digit1, space1),
            // reply_to
            opt(terminated(ident1, space1)),
            // payload_size
            digit1,
        )),
    )(input)?;

    Ok(Message::Message(Payload {
        subject,
        sid,
        reply_to,
        payload_size,
        payload: None,
    }))
}

#[inline]
fn parse_subscribe(input: Bytes) -> ParseResult<Message> {
    use message::subscribe::Payload;

    let (_, (subject, queue_group, sid)) = preceded(
        space1,
        tuple((
            // subject
            terminated(ident1, space1),
            // reply_to
            opt(terminated(ident1, space1)),
            // sid
            digit1,
        )),
    )(input)?;

    Ok(Message::Subscribe(Payload {
        subject,
        sid,
        queue_group,
    }))
}

#[inline]
fn parse_unsubscribe(input: Bytes) -> ParseResult<Message> {
    use message::unsubscribe::Payload;

    let (_, (sid, max_messages)) = preceded(
        space1,
        tuple((
            // sid
            digit1,
            // max_messages
            opt(preceded(space1, digit1)),
        )),
    )(input)?;

    Ok(Message::Unsubscribe(Payload { sid, max_messages }))
}

#[inline]
pub fn cl1(input: &mut bytes::BytesMut) -> nom::IResult<&mut BytesMut, Bytes> {
    use memchr::memchr;

    let found = memchr(b'\r', input).and_then(|r_idx| {
        let n_idx = input.get(r_idx + 1).and_then(|v| Some(*v == b'\n'));

        Some((r_idx, n_idx))
    });

    let idx = match found {
        Some((idx, Some(_))) => idx,
        _ => {
            return Err(nom::Err::Incomplete(nom::Needed::Unknown));
        }
    };

    let cl = input.split_to(idx);
    input.advance(2);

    Ok((input, cl.freeze()))
}

#[inline]
fn space1(input: Bytes) -> ParseResult<(Bytes, ())> {
    let cond = |c| matches!(c, b' ' | b'\t');

    skip_while1(cond)(input)
}

#[inline]
fn tag_u8(b: u8) -> impl Fn(Bytes) -> ParseResult<(Bytes, ())> {
    move |mut input| {
        if input.len() < 1 {
            return Err(nom::Err::Incomplete(Needed::new(1)));
        };

        if !input[0] == b {
            let code = nom::error::ErrorKind::Tag;

            return Err(nom::Err::Error(nom::error::Error::new(input, code)));
        };

        input.advance(1);

        Ok((input, ()))
    }
}

#[inline]
fn clrf1(mut input: Bytes) -> ParseResult<(Bytes, ())> {
    if input.len() < 2 {
        return Err(nom::Err::Incomplete(Needed::new(2)));
    };

    let found = input[0] == b'\r' && input[1] == b'\n';

    if !found {
        let code = nom::error::ErrorKind::TakeWhile1;

        return Err(nom::Err::Error(nom::error::Error::new(input, code)));
    };

    input.advance(2);

    Ok((input, ()))
}

#[inline]
fn digit1(input: Bytes) -> ParseResult<(Bytes, usize)> {
    let cond = |c| matches!(c, b'0'..=b'9');

    let (input, found) = take_while1(cond)(input)?;

    let (d, _) = FromRadix10::from_radix_10(&found);

    Ok((input, d))
}

#[inline]
fn ident1(input: Bytes) -> ParseResult<(Bytes, Bytes)> {
    take_while1(|c| c != b' ')(input)
}

#[inline]
fn take_while1(cond: impl Fn(u8) -> bool) -> impl Fn(Bytes) -> ParseResult<(Bytes, Bytes)> {
    move |mut input: Bytes| {
        let at = input.iter().take_while(|c| cond(**c)).count();

        match at {
            0 => {
                let code = nom::error::ErrorKind::TakeWhile1;

                Err(nom::Err::Error(nom::error::Error::new(input, code)))
            }
            _ => {
                let found = input.split_to(at);

                Ok((input, found))
            }
        }
    }
}

#[inline]
fn skip_while1(cond: impl Fn(u8) -> bool) -> impl Fn(Bytes) -> ParseResult<(Bytes, ())> {
    move |mut input: Bytes| {
        let cnt = input.iter().take_while(|c| cond(**c)).count();

        match cnt {
            0 => {
                let code = nom::error::ErrorKind::TakeWhile1;

                Err(nom::Err::Error(nom::error::Error::new(input, code)))
            }
            _ => {
                input.advance(cnt);

                Ok((input, ()))
            }
        }
    }
}

#[cfg(test)]
mod test {
    use bytes::{Bytes, BytesMut};

    use crate::parser::ParseResult;

    use super::message::{self, Message};
    use super::{cl1, parse};

    use super::{skip_while1, take_while1};

    #[test]
    fn test_cl1() {
        let mut input = BytesMut::from("-ERR 'Maximum Connections Exceeded'\r\n");

        let result = cl1(&mut input);

        assert!(result.is_ok());
    }

    #[test]
    fn test_skip_while1() {
        {
            let input = Bytes::from(" \t\tFOO.BAR".as_bytes());

            assert_eq!(
                Ok((Bytes::from("FOO.BAR"), ())),
                skip_while1(|c| matches!(c, b' ' | b'\t'))(input)
            );
        }
        {
            let input = Bytes::from("FOO.BAR".as_bytes());

            let code = nom::error::ErrorKind::TakeWhile1;
            assert_eq!(
                Err(nom::Err::Error(nom::error::Error::new(input.clone(), code))),
                skip_while1(|c| matches!(c, b' ' | b'\t'))(input)
            );
        }
    }

    #[test]
    fn test_take_while1() {
        {
            let input = Bytes::from(" \t\tFOO.BAR".as_bytes());

            assert_eq!(
                Ok((Bytes::from("FOO.BAR"), Bytes::from(" \t\t"))),
                take_while1(|c| matches!(c, b' ' | b'\t'))(input)
            );
        }
        {
            let input = Bytes::from("FOO.BAR".as_bytes());

            let code = nom::error::ErrorKind::TakeWhile1;
            assert_eq!(
                Err(nom::Err::Error(nom::error::Error::new(input.clone(), code))),
                take_while1(|c| matches!(c, b' ' | b'\t'))(input)
            );
        }
    }

    #[test]
    fn test_parse_error() {
        use message::error::Payload;

        let cases: &[(ParseResult<Message>, &str)] = &[
            (
                Ok(Message::Err(Payload::UnknownProtocolOperation)),
                "-ERR 'Unknown Protocol Operation'",
            ),
            (
                Ok(Message::Err(Payload::AttemptedToConnectToRoutePort)),
                "-ERR 'Attempted To Connect To Route Port'",
            ),
            (
                Ok(Message::Err(Payload::AuthorizationViolation)),
                "-ERR 'Authorization Violation'",
            ),
            (
                Ok(Message::Err(Payload::AuthorizationTimeout)),
                "-ERR 'Authorization Timeout'",
            ),
            (
                Ok(Message::Err(Payload::InvalidClientProtocol)),
                "-ERR 'Invalid Client Protocol'",
            ),
            (
                Ok(Message::Err(Payload::MaximumControlLineExceeded)),
                "-ERR 'Maximum Control Line Exceeded'",
            ),
            (
                Ok(Message::Err(Payload::ParserError)),
                "-ERR 'Parser Error'",
            ),
            (
                Ok(Message::Err(Payload::SecureConnectionTLSRequired)),
                "-ERR 'Secure Connection - TLS Required'",
            ),
            (
                Ok(Message::Err(Payload::StaleConnection)),
                "-ERR 'Stale Connection'",
            ),
            (
                Ok(Message::Err(Payload::MaximumConnectionsExceeded)),
                "-ERR 'Maximum Connections Exceeded'",
            ),
            (
                Ok(Message::Err(Payload::SlowConsumer)),
                "-ERR 'Slow Consumer'",
            ),
            (
                Ok(Message::Err(Payload::MaximumPayloadViolation)),
                "-ERR 'Maximum Payload Violation'",
            ),
            (
                Ok(Message::Err(Payload::InvalidSubject)),
                "-ERR 'Invalid Subject'",
            ),
            (
                Ok(Message::Err(Payload::PermissionsViolationForSubscription(
                    "topic".into(),
                ))),
                "-ERR 'Permissions Violation for Subscription to topic'",
            ),
            (
                Ok(Message::Err(Payload::PermissionsViolationForPublishTo(
                    "topic".into(),
                ))),
                "-ERR 'Permissions Violation for Publish to topic'",
            ),
            (
                Ok(Message::Err(Payload::Unknown("unknown error".into()))),
                "-ERR 'unknown error'",
            ),
        ];

        for (result, raw) in cases {
            assert_eq!(*result, parse(Bytes::from(*raw)));
        }
    }

    #[test]
    fn parse_publish() {
        use message::publish::Payload;

        let cases: &[(ParseResult<Message>, &str)] = &[
            (
                Ok(Message::Publish(Payload {
                    payload: None,
                    subject: Bytes::from("FRONT.DOOR"),
                    reply_to: Some(Bytes::from("BACK.DOOR")),
                    payload_size: 11,
                })),
                "PUB FRONT.DOOR BACK.DOOR 11",
            ),
            (
                Ok(Message::Publish(Payload {
                    payload: None,
                    subject: Bytes::from("FRONT.DOOR"),
                    reply_to: None,
                    payload_size: 11,
                })),
                "PUB FRONT.DOOR 11",
            ),
        ];

        for (result, raw) in cases {
            assert_eq!(*result, parse(Bytes::from(*raw)));
        }
    }

    #[test]
    fn parse_message() {
        use message::message::Payload;

        let cases: &[(ParseResult<Message>, &str)] = &[
            (
                Ok(Message::Message(Payload {
                    subject: Bytes::from("FOO.BAR"),
                    sid: 9,
                    reply_to: Some(Bytes::from("INBOX.34")),
                    payload_size: 11,
                    payload: None,
                })),
                "MSG FOO.BAR 9 INBOX.34 11",
            ),
            (
                Ok(Message::Message(Payload {
                    subject: Bytes::from("FOO.BAR"),
                    sid: 9,
                    reply_to: None,
                    payload_size: 11,
                    payload: None,
                })),
                "MSG FOO.BAR 9 11",
            ),
        ];
        for (result, raw) in cases {
            assert_eq!(*result, parse(Bytes::from(*raw)));
        }
    }

    #[test]
    fn parse_subscribe() {
        use message::subscribe::Payload;

        let cases: &[(ParseResult<Message>, &str)] = &[
            (
                Ok(Message::Subscribe(Payload {
                    sid: 44,
                    subject: Bytes::from("BAR"),
                    queue_group: Some(Bytes::from("G1")),
                })),
                "SUB BAR G1 44",
            ),
            (
                Ok(Message::Subscribe(Payload {
                    sid: 1,
                    subject: Bytes::from("FOO"),
                    queue_group: None,
                })),
                "SUB FOO 1",
            ),
        ];

        for (result, raw) in cases {
            assert_eq!(*result, parse(Bytes::from(*raw)));
        }
    }

    #[test]
    fn parse_unsubscribe() {
        use message::unsubscribe::Payload;

        let cases: &[(ParseResult<Message>, &str)] = &[
            (
                Ok(Message::Unsubscribe(Payload {
                    sid: 1,
                    max_messages: None,
                })),
                "UNSUB 1",
            ),
            (
                Ok(Message::Unsubscribe(Payload {
                    sid: 1,
                    max_messages: Some(5),
                })),
                "UNSUB 1 5",
            ),
        ];
        for (result, raw) in cases {
            assert_eq!(*result, parse(Bytes::from(*raw)));
        }
    }

    #[test]
    fn parse_info() {
        let input = Bytes::from("INFO {\"server_id\":\"Zk0GQ3JBSrg3oyxCRRlE09\",\"version\":\"1.2.0\",\"proto\":1,\"go\":\"go1.10.3\",\"host\":\"0.0.0.0\",\"port\":4222,\"max_payload\":1048576,\"client_id\":2392}");

        let result = parse(input);

        assert!(result.is_ok());
    }
}
