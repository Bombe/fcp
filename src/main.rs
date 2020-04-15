use clap::{crate_version, App, Arg};

use fcp::{FcpConnection, FcpMessage};

fn main() {
    let matches = App::new("fcp")
        .version(crate_version!())
        .author("David “Bombe” Roden")
        .about("Command-line FCP client")
        .arg(
            Arg::with_name("hostname")
                .short("h")
                .long("fcp-host")
                .help("The FCP host name")
                .default_value("localhost"),
        )
        .get_matches();

    println!("fcp 0.1");

    let mut fcp_connection = FcpConnection::default(matches.value_of("hostname").unwrap());
    fcp_connection.connect().expect("could not connect");
    let mut client_hello = FcpMessage::create("ClientHello");
    client_hello.add_field("Name", "TestClient");
    client_hello.add_field("ExpectedVersion", "2.0");
    fcp_connection.send_message(client_hello).unwrap();
    println!("got: {:?}", fcp_connection.recv_message().unwrap());
}
