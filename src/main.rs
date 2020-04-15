use std::error::Error;

use clap::{crate_version, App, Arg};
use config::File;

use fcp::FcpConnection;

fn main() -> Result<(), Box<dyn Error>> {
    let mut config = config::Config::default();

    config
        .merge(File::with_name("./fcp").required(false))
        .unwrap()
        .set_default("fcp-hostname", "localhost".to_string())
        .unwrap();

    let default_fcp_hostname = config.get_str("fcp-hostname").unwrap();

    let matches = App::new("fcp")
        .version(crate_version!())
        .author("David “Bombe” Roden")
        .about("Command-line FCP client")
        .arg(
            Arg::with_name("hostname")
                .short("h")
                .long("fcp-host")
                .help("The FCP host name")
                .default_value(default_fcp_hostname.as_str()),
        )
        .get_matches();

    println!("fcp {}", crate_version!());

    let mut fcp_connection = FcpConnection::default(matches.value_of("hostname").unwrap());
    fcp_connection.connect("TestClient")?;
    println!(
        "Connected to {}:{}.",
        matches.value_of("hostname").unwrap(),
        9481
    );

    Ok(())
}
