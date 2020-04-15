use std::collections::HashMap;
use std::io::{BufRead, BufReader, Error, ErrorKind, Result as IoResult, Write};
use std::net::Shutdown::Both;
use std::net::TcpStream;
use std::ops::Drop;

#[derive(Debug)]
pub struct FcpConnection {
    host: String,
    port: u16,
    stream: Option<Box<TcpStream>>,
}

impl FcpConnection {
    fn default(host: &str) -> FcpConnection {
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
    fn connect(&mut self) -> IoResult<()> {
        self.stream = Option::Some(Box::new(TcpStream::connect((
            self.host.as_str(),
            self.port,
        ))?));
        Ok(())
    }

    #[allow(unused_must_use)]
    fn disconnect(&mut self) {
        if let Some(stream) = &self.stream {
            stream.shutdown(Both);
        }
    }

    fn send_message(&mut self, fcp_message: FcpMessage) -> IoResult<()> {
        match self.stream.as_mut() {
            None => return Err(Error::new(ErrorKind::NotConnected, "not connected")),
            Some(stream) => {
                stream.write(fcp_message.to_string().as_bytes())?;
            }
        }
        Ok(())
    }

    fn recv_message(&mut self) -> Result<FcpMessage, Error> {
        match self.stream.as_mut() {
            None => return Err(Error::new(ErrorKind::NotConnected, "not connected")),
            Some(stream) => {
                let mut name = String::new();
                let mut reader = BufReader::new(stream);
                reader.read_line(&mut name)?;
                let mut message = FcpMessage::create(&name);
                loop {
                    let mut line = String::new();
                    reader.read_line(&mut line)?;
                    if let Some(equal_sign) = line.find('=') {
                        message.add_field(&line.as_str()[..equal_sign], &line[(equal_sign + 1)..]);
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
struct FcpMessage {
    name: String,
    fields: HashMap<Box<String>, Box<String>>,
}

impl FcpMessage {
    fn create(name: &str) -> FcpMessage {
        FcpMessage {
            name: String::from(name),
            fields: HashMap::new(),
        }
    }

    fn add_field(&mut self, name: &str, value: &str) {
        self.fields
            .insert(Box::new(name.to_string()), Box::new(value.to_string()));
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

fn main() {
    println!("fcp 0.1");

    let mut fcp_connection = FcpConnection::default("freenet");
    fcp_connection.connect().expect("could not connect");
    let mut client_hello = FcpMessage::create("ClientHello");
    client_hello.add_field("Name", "TestClient");
    client_hello.add_field("ExpectedVersion", "2.0");
    fcp_connection.send_message(client_hello).unwrap();
    println!("got: {:?}", fcp_connection.recv_message().unwrap());
}
