use std::net::Ipv4Addr;
use std::str::{self, FromStr};
pub use clap::Parser;
use crate::utils::*;

//structure that handles CLI arguments/flags
#[derive(Parser)]
#[command(author = "tuchaVshortah", version = "1.3.0", about = "ARP spoofing detector program", long_about = None)]
pub struct Cli {

    #[arg(short, long, help="Installs ArpSpoofDetectService with launch arguments")]
    pub install_service: bool,

    #[arg(short, long, help="Uninstalls the service")]
    pub uninstall_service: bool,

    #[arg(short, long, help="Runs the service")]
    pub run_service: bool,

    #[arg(short, long, default_value="tcp", help="Specifies which protocol to use. Can be tcp or udp (case sensitive)")]
    pub proto: Proto,

    #[arg(long, default_value_t = Ipv4Addr::from_str("127.0.0.1").unwrap(), help="Takes IP address of the Syslog server")]
    pub syslog_ip: Ipv4Addr,

    #[arg(long, default_value_t = String::from("1468"), help="Specifies the server port to connect to")]
    pub syslog_port: String,

    #[arg(long, default_value_t = Ipv4Addr::from_str("127.0.0.1").unwrap(), help="Takes IP address of the local machine. Required when udp is used")]
    pub local_ip: Ipv4Addr,

    #[arg(long, default_value_t = String::from("9999"), help="Specifies the local port to use. Required when udp is used")]
    pub local_port: String,

    #[arg(long, default_value_t = 3.0)]
    pub timeout: f32,
}