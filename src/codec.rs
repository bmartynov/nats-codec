use std::io;
use bytes::Buf;
use tokio_util;

use super::{message::Message, parser};

enum State {
    Message,
    Payload(usize),
}

pub struct Codec {
    state: State,
    message: Option<Message>,
}

impl Default for Codec {
    fn default() -> Self {
        Self {
            state: State::Message,
            message: None,
        }
    }
}

impl Codec {
    pub fn new() -> Self {
        Self {
            state: State::Message,
            message: None,
        }
    }
}

impl tokio_util::codec::Decoder for Codec {
    type Item = Message;

    type Error = io::Error;

    fn decode(&mut self, input: &mut bytes::BytesMut) -> Result<Option<Self::Item>, Self::Error> {
        loop {
            match self.state {
                State::Message => {
                    // take control line and parse message
                    let message = match parser::cl1(input) {
                        Err(e) => {
                            return into_result(e);
                        }
                        Ok((_, input)) => match parser::parse(input) {
                            Ok(message) => message,
                            Err(e) => {
                                return into_result(e);
                            }
                        },
                    };

                    if !message.with_body() {
                        break Ok(Some(message));
                    } else {
                        self.state = State::Payload(message.body_size().unwrap());

                        self.message = Some(message);
                    }
                }
                State::Payload(size) => {
                    break if input.len() >= size + 2 {
                        let body = input.split_to(size).freeze();

                        input.advance(2);

                        let mut message = self.message.take().unwrap();

                        message.set_body(body);

                        self.state = State::Message;

                        Ok(Some(message))
                    } else {
                        Ok(None)
                    };
                }
            }
        }
    }
}

impl tokio_util::codec::Encoder<Message> for Codec {
    type Error = io::Error;

    fn encode(&mut self, item: Message, dst: &mut bytes::BytesMut) -> Result<(), Self::Error> {
        item.encode(dst)
    }
}

fn into_result<T>(e: nom::Err<nom::error::Error<T>>) -> Result<Option<Message>, io::Error> {
    match e {
        nom::Err::Error(_e) => {
            let kind = io::ErrorKind::Other;

            Err(io::Error::new(kind, "into_result"))
        }
        nom::Err::Failure(_) => Ok(None),
        nom::Err::Incomplete(_) => Ok(None),
    }
}

#[cfg(test)]
mod tests {
    use bytes::BytesMut;
    use tokio_util::codec::Decoder;

    use super::Codec;

    #[test]
    fn it_works() {
        let mut input =
            BytesMut::from("MSG FRONT.DOOR 666 BACK.DOOR 11\r\n12345678901\r\nMSG FRONT.DOOR 666 11\r\n12345678901\r\n".as_bytes());

        let mut codec = Codec::new();

        println!("{:?}", codec.decode(&mut input).expect("ok"));
        println!("{:?}", codec.decode(&mut input).expect("ok"));

        println!("input: {:?}", input.len());
        println!("capacity: {:?}", input.capacity());
    }
}
