use fcp::{FcpConnection, FcpMessage};

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
