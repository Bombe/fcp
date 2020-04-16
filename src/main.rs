use std::error::Error;
use std::process;

use clap::{crate_version, App, AppSettings, Arg, SubCommand};
use config::File;

use fcp::FcpConnection;

use crate::FcpCommand::Test;

fn main() -> Result<(), Box<dyn Error>> {
    let arguments = parse_arguments(
        &parse_config_file("./fcp"),
        std::env::args().skip(1).collect(),
    );

    if arguments.command.is_none() {
        println!("No command to run.");
        return Ok(());
    }

    let mut fcp_connection = FcpConnection::create(&arguments.hostname, arguments.port);
    if arguments.verbose {
        println!("Connecting to {}:{}...", arguments.hostname, arguments.port);
    }
    if let Err(error) = fcp_connection.connect("TestClient") {
        if !arguments.quiet {
            return Err(Box::new(error));
        }
        process::exit(1)
    }
    if arguments.verbose {
        println!("Connected to {}:{}.", arguments.hostname, arguments.port);
    }

    Ok(())
}

#[derive(Debug, PartialEq)]
struct FcpArguments {
    hostname: String,
    port: u16,
    command: Option<FcpCommand>,
    verbose: bool,
    quiet: bool,
}

#[derive(Debug, PartialEq)]
enum FcpCommand {
    Test,
}

fn parse_arguments(config: &FcpConfig, args: Vec<String>) -> FcpArguments {
    let default_fcp_hostname = "localhost".to_string();
    let fcp_hostname = config
        .fcp_hostname
        .as_ref()
        .unwrap_or(&default_fcp_hostname);
    let fcp_port = config.fcp_port.unwrap_or(9481).to_string();

    let arg_matches = App::new("fcp")
        .version(crate_version!())
        .author("David “Bombe” Roden")
        .about("Command-line FCP client")
        .arg(
            Arg::with_name("hostname")
                .short("h")
                .long("fcp-host")
                .takes_value(true)
                .help("The FCP host name")
                .default_value(&fcp_hostname),
        )
        .arg(
            Arg::with_name("port")
                .short("p")
                .long("fcp-port")
                .takes_value(true)
                .help("The FCP port number")
                .default_value(&fcp_port),
        )
        .arg(
            Arg::with_name("verbose")
                .short("v")
                .long("verbose")
                .takes_value(false)
                .conflicts_with("quiet")
                .help("Be verbose"),
        )
        .arg(
            Arg::with_name("quiet")
                .short("q")
                .long("quiet")
                .takes_value(false)
                .conflicts_with("verbose")
                .help("Be quiet, only set exit codes"),
        )
        .subcommand(SubCommand::with_name("test").about("Tests whether a node is reachable"))
        .setting(AppSettings::NoBinaryName)
        .get_matches_from(args);

    FcpArguments {
        hostname: arg_matches.value_of("hostname").unwrap().to_string(),
        port: arg_matches
            .value_of("port")
            .unwrap()
            .parse()
            .unwrap_or(9481),
        command: match arg_matches.subcommand() {
            ("test", Some(_)) => Some(Test),
            _ => None,
        },
        verbose: arg_matches.is_present("verbose"),
        quiet: arg_matches.is_present("quiet"),
    }
}

fn parse_config_file(config_file: &str) -> FcpConfig {
    let mut config = config::Config::default();
    config
        .merge(File::with_name(config_file).required(false))
        .unwrap()
        .set_default("fcp-hostname", "localhost".to_string())
        .unwrap()
        .set_default("fcp-port", 9481.to_string())
        .unwrap();

    FcpConfig {
        fcp_hostname: config.get_str("fcp-hostname").ok(),
        fcp_port: config.get_int("fcp-port").map(|p| p as u16).ok(),
    }
}

/// Configuration as read from a configuration file.
#[derive(Debug, PartialEq)]
struct FcpConfig {
    /// The hostname to connect to.
    fcp_hostname: Option<String>,

    /// The port number to connect to.
    fcp_port: Option<u16>,
}

impl FcpConfig {
    fn create(fcp_hostname: Option<String>, fcp_port: Option<u16>) -> FcpConfig {
        FcpConfig {
            fcp_hostname,
            fcp_port,
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::{parse_arguments, FcpArguments, FcpConfig};

    #[test]
    fn empty_args_are_parsed_correctly() {
        let args = parse_arguments(&FcpConfig::create(None, None), Vec::new());
        assert_eq!(
            args,
            FcpArguments {
                hostname: "localhost".to_string(),
                port: 9481,
                command: None,
                verbose: false,
                quiet: false,
            }
        )
    }

    #[test]
    fn hostname_is_parsed_with_short_parameter_name() {
        let args = parse_arguments(
            &FcpConfig::create(None, None),
            vec!["-h".to_string(), "hostname.test".to_string()],
        );
        assert_eq!(
            args,
            FcpArguments {
                hostname: "hostname.test".to_string(),
                port: 9481,
                command: None,
                verbose: false,
                quiet: false,
            }
        )
    }

    #[test]
    fn hostname_is_parsed_with_long_parameter_name() {
        let args = parse_arguments(
            &FcpConfig::create(None, None),
            vec!["--fcp-host".to_string(), "hostname.test".to_string()],
        );
        assert_eq!(
            args,
            FcpArguments {
                hostname: "hostname.test".to_string(),
                port: 9481,
                command: None,
                verbose: false,
                quiet: false,
            }
        )
    }

    #[test]
    fn port_is_parsed_with_short_parameter_name() {
        let args = parse_arguments(
            &FcpConfig::create(None, None),
            vec!["-p".to_string(), "12345".to_string()],
        );
        assert_eq!(
            args,
            FcpArguments {
                hostname: "localhost".to_string(),
                port: 12345,
                command: None,
                verbose: false,
                quiet: false,
            }
        )
    }

    #[test]
    fn port_is_parsed_with_long_parameter_name() {
        let args = parse_arguments(
            &FcpConfig::create(None, None),
            vec!["--fcp-port".to_string(), "12345".to_string()],
        );
        assert_eq!(
            args,
            FcpArguments {
                hostname: "localhost".to_string(),
                port: 12345,
                command: None,
                verbose: false,
                quiet: false,
            }
        )
    }
}
