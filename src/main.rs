use std::error::Error;

use clap::{crate_version, App, Arg};
use config::File;

use fcp::FcpConnection;

fn main() -> Result<(), Box<dyn Error>> {
    let arguments = parse_arguments();

    println!("fcp {}", crate_version!());

    let mut fcp_connection = FcpConnection::create(&arguments.hostname, arguments.port);
    fcp_connection.connect("TestClient")?;
    println!("Connected to {}:{}.", arguments.hostname, arguments.port);

    Ok(())
}

struct FcpArguments {
    hostname: String,
    port: u16,
}

fn parse_arguments() -> FcpArguments {
    let mut config = config::Config::default();
    config
        .merge(File::with_name("./fcp").required(false))
        .unwrap()
        .set_default("fcp-hostname", "localhost".to_string())
        .unwrap()
        .set_default("fcp-port", 9481.to_string())
        .unwrap();

    let default_fcp_hostname = config.get_str("fcp-hostname").unwrap();
    let default_fcp_port = config.get_int("fcp-port").unwrap_or(9481).to_string();

    let arg_matches = App::new("fcp")
        .version(crate_version!())
        .author("David “Bombe” Roden")
        .about("Command-line FCP client")
        .arg(
            Arg::with_name("hostname")
                .short("h")
                .long("fcp-host")
                .help("The FCP host name")
                .default_value(&default_fcp_hostname),
        )
        .arg(
            Arg::with_name("port")
                .short("p")
                .long("fcp-port")
                .help("The FCP port number")
                .default_value(&default_fcp_port),
        )
        .get_matches();

    FcpArguments {
        hostname: arg_matches.value_of("hostname").unwrap().to_string(),
        port: arg_matches
            .value_of("port")
            .unwrap()
            .parse()
            .unwrap_or(9481),
    }
}
