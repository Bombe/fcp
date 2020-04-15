use std::collections::HashMap;
use std::io::{BufRead, BufReader, Error, ErrorKind, Write};
use std::net::Shutdown::Both;
use std::net::TcpStream;

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
    fn drop(&mut self) {
        self.disconnect();
    }
}

impl FcpConnection {
    pub fn connect(&mut self) -> Result<(), Error> {
        self.stream = Option::Some(Box::new(TcpStream::connect((
            self.host.as_str(),
            self.port,
        ))?));
        Ok(())
    }

    #[allow(unused_must_use)]
    pub fn disconnect(&mut self) {
        if let Some(stream) = &self.stream {
            stream.shutdown(Both);
        }
    }

    pub fn send_message(&mut self, fcp_message: FcpMessage) -> Result<(), Error> {
        match self.stream.as_mut() {
            None => return Err(Error::new(ErrorKind::NotConnected, "not connected")),
            Some(stream) => {
                stream.write(fcp_message.to_string().as_bytes())?;
            }
        }
        Ok(())
    }

    pub fn recv_message(&mut self) -> Result<FcpMessage, Error> {
        match self.stream.as_mut() {
            None => return Err(Error::new(ErrorKind::NotConnected, "not connected")),
            Some(stream) => {
                let mut name = String::new();
                let mut reader = BufReader::new(stream);
                reader.read_line(&mut name)?;
                let mut message = FcpMessage::create(&name.trim_end_matches('\n'));
                loop {
                    let mut line = String::new();
                    reader.read_line(&mut line)?;
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
