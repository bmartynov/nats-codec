use std::io;

use bytes::BufMut;

static UNKNOWN_PROTOCOL_OPERATION: &[u8] = "Unknown Protocol Operation".as_bytes();
static ATTEMPTED_TO_CONNECT_TO_ROUTE_PORT: &[u8] = "Attempted To Connect To Route Port".as_bytes();
static AUTHORIZATION_VIOLATION: &[u8] = "Authorization Violation".as_bytes();
static AUTHORIZATION_TIMEOUT: &[u8] = "Authorization Timeout".as_bytes();
static INVALID_CLIENT_PROTOCOL: &[u8] = "Invalid Client Protocol".as_bytes();
static MAXIMUM_CONTROL_LINE_EXCEEDED: &[u8] = "Maximum Control Line Exceeded".as_bytes();
static PARSER_ERROR: &[u8] = "Parser Error".as_bytes();
static SECURE_CONNECTION_TLSREQUIRED: &[u8] = "Secure Connection - TLS Required".as_bytes();
static STALE_CONNECTION: &[u8] = "Stale Connection".as_bytes();
static MAXIMUM_CONNECTIONS_EXCEEDED: &[u8] = "Maximum Connections Exceeded".as_bytes();
static SLOW_CONSUMER: &[u8] = "Slow Consumer".as_bytes();
static MAXIMUM_PAYLOAD_VIOLATION: &[u8] = "Maximum Payload Violation".as_bytes();
static INVALID_SUBJECT: &[u8] = "Invalid Subject".as_bytes();
static PERMISSIONS_VIOLATION_FOR_SUBSCRIPTION: &[u8] =
    "Permissions Violation for Subscription to ".as_bytes();
static PERMISSIONS_VIOLATION_FOR_PUBLISH_TO: &[u8] =
    "Permissions Violation for Publish to ".as_bytes();

const HEADER: &'static [u8] = &[b'-', b'E', b'R', b'R', b' '];
const FOOTER: &'static [u8] = &[b'\'', b'\r', b'\n'];

#[derive(Debug, Clone, PartialEq)]
pub enum Payload {
    // Unknown error
    Unknown(String),
    // Unknown protocol error
    UnknownProtocolOperation,
    // Client attempted to connect to a route port instead of the client port
    AttemptedToConnectToRoutePort,
    // Client failed to authenticate to the server with credentials specified in the
    AuthorizationViolation,
    // Client took too long to authenticate to the server after establishing a connection (default 1 second)
    AuthorizationTimeout,
    // Client specified an invalid protocol version in the message
    InvalidClientProtocol,
    // Message destination subject and reply subject length exceeded the maximum control line value
    // specified by the max_control_line server option. The default is 1024 bytes
    MaximumControlLineExceeded,
    // Cannot parse the protocol message sent by the client
    ParserError,
    // The server requires TLS and the client does not have TLS enabled
    SecureConnectionTLSRequired,
    // The server hasn't received a message from the client, including a PONG in too long
    StaleConnection,
    // This error is sent by the server when creating a new connection and the server
    // has exceeded the maximum number of connections specified by the max_connections server option. The default is 64k
    MaximumConnectionsExceeded,
    // The server pending data size for the connection has reached the maximum size (default 10MB).
    SlowConsumer,
    // Client attempted to publish a message with a payload size that exceeds the max_payload size configured on the server.
    // This value is supplied to the client upon connection in the initial message.
    // The client is expected to do proper accounting of byte size to be sent to the server in order to handle this error synchronously.
    MaximumPayloadViolation,
    // Client sent a malformed subject (e.g. sub foo. 90)
    InvalidSubject,
    // The user specified in the message does not have permission to subscribe to the subject.
    PermissionsViolationForSubscription(String),
    // The user specified in the message does not have permission to publish to the subject.
    PermissionsViolationForPublishTo(String),
}

impl Payload {
    pub fn encode(&self, dst: &mut bytes::BytesMut) -> Result<(), io::Error> {
        dst.put_slice(HEADER);
        dst.put_u8(b'\'');

        let msg = match self {
            Payload::Unknown(s) => s.as_bytes(),
            Payload::UnknownProtocolOperation => UNKNOWN_PROTOCOL_OPERATION,
            Payload::AttemptedToConnectToRoutePort => ATTEMPTED_TO_CONNECT_TO_ROUTE_PORT,
            Payload::AuthorizationViolation => AUTHORIZATION_VIOLATION,
            Payload::AuthorizationTimeout => AUTHORIZATION_TIMEOUT,
            Payload::InvalidClientProtocol => INVALID_CLIENT_PROTOCOL,
            Payload::MaximumControlLineExceeded => MAXIMUM_CONTROL_LINE_EXCEEDED,
            Payload::ParserError => PARSER_ERROR,
            Payload::SecureConnectionTLSRequired => SECURE_CONNECTION_TLSREQUIRED,
            Payload::StaleConnection => STALE_CONNECTION,
            Payload::MaximumConnectionsExceeded => MAXIMUM_CONNECTIONS_EXCEEDED,
            Payload::SlowConsumer => SLOW_CONSUMER,
            Payload::MaximumPayloadViolation => MAXIMUM_PAYLOAD_VIOLATION,
            Payload::InvalidSubject => INVALID_SUBJECT,
            Payload::PermissionsViolationForSubscription(s) => {
                dst.put_slice(PERMISSIONS_VIOLATION_FOR_SUBSCRIPTION);
                dst.put_slice(s.as_bytes());
                dst.put_slice(FOOTER);

                return Ok(());
            }
            Payload::PermissionsViolationForPublishTo(s) => {
                dst.put_slice(PERMISSIONS_VIOLATION_FOR_PUBLISH_TO);
                dst.put_slice(s.as_bytes());
                dst.put_slice(FOOTER);

                return Ok(());
            }
        };

        dst.put_slice(msg);
        dst.put_slice(FOOTER);

        Ok(())
    }
}

impl From<&[u8]> for Payload {
    fn from(raw: &[u8]) -> Self {
        match raw {
            s if s.starts_with(UNKNOWN_PROTOCOL_OPERATION) => Self::UnknownProtocolOperation,
            s if s.starts_with(ATTEMPTED_TO_CONNECT_TO_ROUTE_PORT) => {
                Self::AttemptedToConnectToRoutePort
            }
            s if s.starts_with(AUTHORIZATION_VIOLATION) => Self::AuthorizationViolation,
            s if s.starts_with(AUTHORIZATION_TIMEOUT) => Self::AuthorizationTimeout,
            s if s.starts_with(INVALID_CLIENT_PROTOCOL) => Self::InvalidClientProtocol,
            s if s.starts_with(MAXIMUM_CONTROL_LINE_EXCEEDED) => Self::MaximumControlLineExceeded,
            s if s.starts_with(PARSER_ERROR) => Self::ParserError,
            s if s.starts_with(SECURE_CONNECTION_TLSREQUIRED) => Self::SecureConnectionTLSRequired,
            s if s.starts_with(STALE_CONNECTION) => Self::StaleConnection,
            s if s.starts_with(MAXIMUM_CONNECTIONS_EXCEEDED) => Self::MaximumConnectionsExceeded,
            s if s.starts_with(SLOW_CONSUMER) => Self::SlowConsumer,
            s if s.starts_with(MAXIMUM_PAYLOAD_VIOLATION) => Self::MaximumPayloadViolation,
            s if s.starts_with(INVALID_SUBJECT) => Self::InvalidSubject,
            s if s.starts_with(PERMISSIONS_VIOLATION_FOR_SUBSCRIPTION) => {
                let plen = PERMISSIONS_VIOLATION_FOR_SUBSCRIPTION.len();

                Self::PermissionsViolationForSubscription(
                    String::from_utf8(raw[plen..].to_vec())
                        .unwrap_or_else(|_| "non utf8".to_string()),
                )
            }
            s if s.starts_with(PERMISSIONS_VIOLATION_FOR_PUBLISH_TO) => {
                let plen = PERMISSIONS_VIOLATION_FOR_PUBLISH_TO.len();

                Self::PermissionsViolationForPublishTo(
                    String::from_utf8(raw[plen..].to_vec())
                        .unwrap_or_else(|_| "non utf8".to_string()),
                )
            }
            s @ _ => Self::Unknown(
                String::from_utf8(s.to_vec()).unwrap_or_else(|_| "non utf8".to_string()),
            ),
        }
    }
}
