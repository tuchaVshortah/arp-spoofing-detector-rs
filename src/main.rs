use std::net::Ipv4Addr;
use std::str::{self, FromStr};
use clap::Parser;

mod utils;
mod local;
mod remote;

use utils::*;
use local::*;

#[allow(unused, unused_imports, unused_variables, dead_code)]

//structure that handles CLI arguments/flags
#[derive(Parser)]
#[command(author = "tuchaVshortah", version = "1.2.1", about = "ARP spoofing detector program", long_about = None)]
struct Cli {

    #[arg(short, long, default_value="tcp", help="Specifies which protocol to use. Can be tcp or udp (case sensitive)")]
    proto: Proto,

    #[arg(long, default_value_t = Ipv4Addr::from_str("127.0.0.1").unwrap(), help="Takes IP address of the Syslog server")]
    syslog_ip: Ipv4Addr,

    #[arg(long, default_value_t = String::from("1468"), help="Specifies the server port to connect to")]
    syslog_port: String,

    #[arg(long, default_value_t = Ipv4Addr::from_str("127.0.0.1").unwrap(), help="Takes IP address of the local machine. Required when udp is used")]
    local_ip: Ipv4Addr,

    #[arg(long, default_value_t = String::from("9999"), help="Specifies the local port to use. Required when udp is used")]
    local_port: String,

    #[arg(long, default_value_t = 3.0)]
    timeout: f32,
}

//the main function
fn main() {
    let cli = Cli::parse();
    
    let options = LoggerOptions {
        syslog_ip: cli.syslog_ip.to_string(),
        syslog_port: cli.syslog_port,
        proto: cli.proto,
        local_ip: cli.local_ip.to_string(),
        local_port: cli.local_port,
    };

    loop {
        detector(&options);
        std::thread::sleep(std::time::Duration::from_secs_f32(cli.timeout));
    }

}
