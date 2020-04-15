use std::error::Error;

use clap::{crate_version, App, Arg};
use config::File;

use fcp::FcpConnection;

fn main() -> Result<(), Box<dyn Error>> {
    let arguments = parse_arguments();

    println!("fcp {}", crate_version!());

    let mut fcp_connection = FcpConnection::default(&arguments.hostname);
    fcp_connection.connect("TestClient")?;
    println!("Connected to {}:{}.", arguments.hostname, 9481);

    Ok(())
}

struct FcpArguments {
    hostname: String,
}

fn parse_arguments() -> FcpArguments {
    let mut config = config::Config::default();
    config
        .merge(File::with_name("./fcp").required(false))
        .unwrap()
        .set_default("fcp-hostname", "localhost".to_string())
        .unwrap();

    let default_fcp_hostname = config.get_str("fcp-hostname").unwrap();

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
        .get_matches();

    FcpArguments {
        hostname: arg_matches.value_of("hostname").unwrap().to_string(),
    }
}
