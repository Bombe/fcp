use std::collections::HashMap;
use std::io::{BufRead, BufReader, Write};
use std::net::Shutdown::Both;
use std::net::TcpStream;

use crate::error::{Error, ToFcpError};

mod error {
    use crate::error::Error::IoError;
    use std::fmt::{Display, Formatter, Result};

    #[derive(Debug)]
    pub enum Error {
        IoError(std::io::Error),
        NotConnected,
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

#[derive(Debug)]
pub struct FcpConnection {
    host: String,
    port: u16,
    stream: Option<Box<TcpStream>>,
}

impl FcpConnection {
    pub fn default(host: &str) -> FcpConnection {
        FcpConnection {
            host: String::from(host),
            port: 9481,
            stream: None,
        }
    }
    #[allow(dead_code)]
    fn create(host: &str, port: u16) -> FcpConnection {
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

impl FcpConnection {
    pub fn connect(&mut self) -> Result<(), Error> {
        self.stream = Option::Some(Box::new(
            TcpStream::connect((self.host.as_str(), self.port)).to_fcp_error()?,
        ));
        Ok(())
    }

    pub fn disconnect(&mut self) -> Result<(), Error> {
        if let Some(stream) = &self.stream {
            stream.shutdown(Both).to_fcp_error()?;
        }
        Ok(())
    }

    pub fn send_message(&mut self, fcp_message: FcpMessage) -> Result<(), Error> {
        match self.stream.as_mut() {
            None => return Err(Error::NotConnected),
            Some(stream) => {
                stream
                    .write(fcp_message.to_string().as_bytes())
                    .to_fcp_error()?;
            }
        }
        Ok(())
    }

    pub fn recv_message(&mut self) -> Result<FcpMessage, Error> {
        match self.stream.as_mut() {
            None => return Err(Error::NotConnected),
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

#[derive(Debug)]
pub struct FcpMessage {
    name: String,
    fields: HashMap<String, String>,
}

impl FcpMessage {
    pub fn create(name: &str) -> FcpMessage {
        FcpMessage {
            name: String::from(name),
            fields: HashMap::new(),
        }
    }

    pub fn add_field(&mut self, name: &str, value: &str) {
        self.fields.insert(name.to_string(), value.to_string());
    }
}

impl ToString for FcpMessage {
    fn to_string(&self) -> String {
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
