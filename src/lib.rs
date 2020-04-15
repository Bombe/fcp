#![crate_name = "fcp"]
//! # Freenet Client Protocol Implementation
//!
//! The Freenet Client Protocol (FCP) is the protocol that is used
//! for the communication between a [Freenet](https://freenetproject.org/)
//! node and its client applications.
//!
use std::collections::HashMap;
use std::io::{BufRead, BufReader, Write};
use std::net::Shutdown::Both;
use std::net::TcpStream;

use crate::error::Error::{NotConnected, ProtocolError};
use crate::error::{Error, ToFcpError};

/// FCP-specific errors.
pub mod error {
    use std::fmt::{Display, Formatter, Result};

    use crate::error::Error::IoError;

    /// Enumeration of possible errors during FCP communication.
    #[derive(Debug)]
    pub enum Error {
        /// Wrapper for an I/O error.
        IoError(std::io::Error),

        /// Error signaling that an FCP connection was used before
        /// it was connected.
        NotConnected,

        /// Error during FCP communication, signaling unexpected
        /// or invalid messages.
        ProtocolError,
    }

    impl Display for Error {
        fn fmt(&self, f: &mut Formatter<'_>) -> Result {
            write!(f, "{}", self)
        }
    }

    impl std::error::Error for Error {}

    pub trait ToFcpError<T> {
        fn to_fcp_error(self) -> core::result::Result<T, Error>;
    }

    impl<T> ToFcpError<T> for core::result::Result<T, std::io::Error> {
        fn to_fcp_error(self) -> core::result::Result<T, Error> {
            self.map_err(|error| IoError(error))
        }
    }
}

/// A connection to a Freenet node.
///
/// Use [default](#method.default) or [create](#method.create) to create new connections.
#[derive(Debug)]
pub struct FcpConnection {
    host: String,
    port: u16,
    stream: Option<Box<TcpStream>>,
}

/// Methods for creating new FCP connections.
impl FcpConnection {
    /// Creates a new connection to a node running on the given
    /// host, using the default FCP port number of `9481`.
    pub fn default(host: &str) -> FcpConnection {
        FcpConnection {
            host: String::from(host),
            port: 9481,
            stream: None,
        }
    }

    /// Creates a new connection to a node running on the given
    /// host and port number.
    pub fn create(host: &str, port: u16) -> FcpConnection {
        FcpConnection {
            host: String::from(host),
            port,
            stream: None,
        }
    }
}

impl Drop for FcpConnection {
    // don’t care if disconnecting fails when going out of scope
    #[allow(unused_must_use)]
    fn drop(&mut self) {
        self.disconnect();
    }
}

/// Methods for manipulating FCP connections, sending
/// messages, and basically doing things with it.
impl FcpConnection {
    /// Starts this FCP connection, sending the given client name
    /// to the node as identifier. A connection has to be connected
    /// before messages can be sent; failure to do so will result
    /// in [NotConnected] errors!
    ///
    /// # Errors
    ///
    /// Any I/O error from the underlying `TcpStream` is wrapped
    /// into an [FCP Error] and returned.
    ///
    /// If the node does not answer our `ClientHello` message
    /// with a corresponding `NodeHello` message, a
    /// [ProtocolError] is returned.
    ///
    /// [FCP Error]: ./error/index.html
    /// [NotConnected]: ./error/enum.Error.html
    /// [ProtocolError]: ./error/enum.Error.html
    pub fn connect(&mut self, client_name: &str) -> Result<(), Error> {
        let stream = TcpStream::connect((self.host.as_str(), self.port)).to_fcp_error()?;
        self.stream = Option::Some(Box::new(stream));

        let mut client_hello = FcpMessage::create("ClientHello");
        client_hello.add_field("Name", client_name);
        client_hello.add_field("ExpectedVersion", "2.0");
        self.send_message(client_hello)?;

        let node_hello = self.recv_message()?;
        if node_hello.name != "NodeHello" {
            return Err(ProtocolError);
        }
        Ok(())
    }

    /// Disconnects this connection from the node.
    ///
    /// # Errors
    ///
    /// Errors from the underlying `TcpStream` are wrapped in an
    /// [FCP Error] and returned.
    ///
    /// [FCP Error]: ./error/index.html
    pub fn disconnect(&mut self) -> Result<(), Error> {
        if let Some(stream) = &self.stream {
            stream.shutdown(Both).to_fcp_error()?;
        }
        Ok(())
    }

    /// Sends the given message to the node.
    ///
    /// # Errors
    ///
    /// If the connection has not been [connected], a
    /// [NotConnected] error is returned.
    ///
    /// Errors from the underlying `TcpStream` are wrapped in an
    /// [FCP Error] and returned.
    ///
    /// [connected]: #method.connect
    /// [NotConnected]: ./error/enum.Error.html
    /// [FCP Error]: ./error/index.html
    pub fn send_message(&mut self, fcp_message: FcpMessage) -> Result<(), Error> {
        match self.stream.as_mut() {
            None => return Err(NotConnected),
            Some(stream) => {
                stream
                    .write(fcp_message.to_field_set().as_bytes())
                    .to_fcp_error()?;
            }
        }
        Ok(())
    }

    /// Receives a message from the node, blocking until it has
    /// been received completely.
    ///
    /// This method can not handle messages with payload.
    ///
    /// # Errors
    ///
    /// If the connection has not been [connected], a
    /// [NotConnected] error is returned.
    ///
    /// Errors from the underlying `TcpStream` are wrapped in an
    /// [FCP Error] and returned.
    ///
    /// [connected]: #method.connect
    /// [NotConnected]: ./error/enum.Error.html
    /// [FCP Error]: ./error/index.html
    pub fn recv_message(&mut self) -> Result<FcpMessage, Error> {
        match self.stream.as_mut() {
            None => return Err(NotConnected),
            Some(stream) => {
                let mut name = String::new();
                let mut reader = BufReader::new(stream);
                reader.read_line(&mut name).to_fcp_error()?;
                let mut message = FcpMessage::create(&name.trim_end_matches('\n'));
                loop {
                    let mut line = String::new();
                    reader.read_line(&mut line).to_fcp_error()?;
                    if let Some(equal_sign) = line.find('=') {
                        message.add_field(
                            &line.as_str()[..equal_sign],
                            &line[(equal_sign + 1)..].trim_end_matches('\n'),
                        );
                    } else {
                        break;
                    }
                }

                Ok(message)
            }
        }
    }
}

/// An FCP message.
///
/// A message consists of a name and an arbitrary number of
/// key-value pairs.
#[derive(Debug)]
pub struct FcpMessage {
    /// The name of the message.
    name: String,

    /// The key-value pairs making up the content of the message.
    fields: HashMap<String, String>,
}

/// Methods that create [FCP Message]s.
///
/// [FCP Message]: struct.FcpMessage.html
impl FcpMessage {
    /// Creates a new FCP message with the given name.
    pub fn create(name: &str) -> FcpMessage {
        FcpMessage {
            name: String::from(name),
            fields: HashMap::new(),
        }
    }
}

/// Methods that manipulate and query [FCP Message]s.
///
/// [FCP Message]: struct.FcpMessage.html
impl FcpMessage {
    /// Adds a field to the message.
    ///
    /// If a field with the given name already exists, it will be
    /// overwritten.
    pub fn add_field(&mut self, name: &str, value: &str) {
        self.fields.insert(name.to_string(), value.to_string());
    }

    /// Renders the message into a field set suitable for transfering
    /// it over FCP.
    fn to_field_set(&self) -> String {
        let mut string = String::new();
        string.push_str(&self.name);
        string.push('\n');
        for (key, value) in self.fields.iter() {
            string.push_str(&format!("{}={}", key, value));
            string.push('\n');
        }
        string.push_str("EndMessage\n");
        string
    }
}
