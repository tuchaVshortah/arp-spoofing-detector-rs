use std::collections::HashMap;
use std::env;
use std::error::Error;
use std::io::Write;
use std::net::{Ipv4Addr, UdpSocket, TcpStream};
use std::process::Command;
use std::str::{self, FromStr};
use std::fmt::Display;
use clap::Parser;
use serde_json::json;
use job_scheduler_ng::{Job, JobScheduler};
use std::time::Duration;

#[allow(unused, unused_imports, unused_variables, dead_code)]


#[derive(Debug, PartialEq, Eq, Clone, Copy)]
enum Proto {
    Udp,
    Tcp,
}

impl Default for Proto {
    fn default() -> Self {
        Proto::Udp
    }
}

impl FromStr for Proto {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "udp" => Ok(Proto::Udp),
            "tcp" => Ok(Proto::Tcp),
            _ => Err(format!("Invalid protocol type: {}", s)),
        }
    }
}

impl Display for Proto {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        
        match self {

            Proto::Udp => write!(f, "Udp"),
            Proto::Tcp => write!(f, "Tcp"),

        }

    }
}


struct LoggerOptions {
    //log levels
    //min_level: SyslogLevels,
    //max_level: SyslogLevels,

    //remote machine
    syslog_ip: String,
    syslog_port: String,

    //protocol used to establish a connection
    proto: Proto,

    //local machine
    local_ip: String,
    local_port: String,

}


fn warning(options: &LoggerOptions, message: String) {
    match options.proto {
        Proto::Udp => {
            let socket = match UdpSocket::bind(format!("{}:{}", options.local_ip, options.local_port)) {
                Ok(socket) => socket,
                Err(error) => { println!("Could not create a UDP socket: {}", error); return; },
            };

            match socket.connect(format!("{}:{}", options.syslog_ip, options.syslog_port)) {
                Ok(())  => {

                    println!("Successfully connected to the server");

                    if let Err(error) = socket.send(message.as_bytes()) {

                        println!("Couldn't send a log through UDP socket: {}", error);

                    }
                },
                Err(error) => {

                    println!("An error happened when sending data to the server: {}", error);

                }
            }
        },
        
        Proto::Tcp => {
            match TcpStream::connect(format!("{}:{}", options.syslog_ip, options.syslog_port)) {
                Ok(mut stream) => {

                    println!("Successfully connected to the server");

                    if let Err(error) = stream.write_all(message.as_bytes()) {

                        println!("Couldn't send a log through TCP stream: {}", error);

                    }
                },
                Err(error) => {

                    println!("An error happened when sending data to the server: {}", error);

                }
            }

        }
    }
}

//arp spoofing detector
fn detector(options: &LoggerOptions) -> Result<(), Box<dyn std::error::Error>> {
    
    let mut arp_cache: HashMap<Ipv4Addr, String> = HashMap::new();

    let output = Command::new("arp")
        .arg("-a")
        .output()
        .expect("Failed to execute command");

    let arp_table = str::from_utf8(&output.stdout).unwrap();
    let mut is_spoofed = false;

    for line in arp_table.lines() {

        let parts: Vec<&str> = line.split_whitespace().collect();

        if parts.len() == 3 {

            let ip = parts[0].parse::<Ipv4Addr>().unwrap();
            let mac = parts[1].to_string();

            if arp_cache.contains_key(&ip) && arp_cache.get(&ip).unwrap() != &mac {

                println!("ARP spoofing detected for IP address {}", ip);

                let mut message = HashMap::new();
                message.insert("description", "ARP spoofing detected");

                let ip_string = ip.to_string();
                message.insert("ip", &ip_string);

                message.insert("First MAC", arp_cache.get(&ip).unwrap());
                message.insert("Second MAC", &mac);

                let json_message = json!(message).to_string();

                warning(&options, json_message);

                is_spoofed = true;

            }

            arp_cache.insert(ip, mac);

        }
    }

    if !is_spoofed {

        println!("No ARP spoofing detected");
        
        let mut message = HashMap::new();
        message.insert("description", "ARP spoofing not detected");

        let json_message = json!(message).to_string();

        warning(&options, json_message);
        
    }
    
    Ok(())
}

//structure that handles CLI arguments/flags
#[derive(Parser)]
#[command(author = "tuchaVshortah", version = "1.0.1", about = "ARP spoofing detector program", long_about = None)]
struct Cli {

    #[arg(short, long, default_value_t = String::from("1/10 * * * * *"), help="Specifiy how often the job should be run using the Cron syntax")]
    job_schedule: String,

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

}

//the main function
fn main() {

    let cli = Cli::parse();
    let mut sched = JobScheduler::new();

    let options = LoggerOptions {
        syslog_ip: cli.syslog_ip.to_string(),
        syslog_port: cli.syslog_port,
        proto: cli.proto,
        local_ip: cli.local_ip.to_string(),
        local_port: cli.local_port,
    };

    let job_id = sched.add(Job::new(cli.job_schedule.as_str().parse().unwrap(), move || {
        detector(&options);
    }));
    println!("Job id: {}", job_id);

    loop {
        sched.tick();
        std::thread::sleep(Duration::from_millis(500));
    }
}
