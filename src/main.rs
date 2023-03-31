use std::collections::HashMap;
use std::error::Error;
use std::io::Write;
use std::net::{Ipv4Addr, UdpSocket, TcpStream};
use std::process::Command;
use std::str::{self, FromStr};
use std::fmt::Display;
use clap::Parser;
use serde_json::{json, error};

#[allow(unused, unused_variables, dead_code)]


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

    //timeout used to sleep between requests
    timeout: f32,
}


fn warning(options: &LoggerOptions, message: String) {
    match options.proto {
        Proto::Udp => {
            let socket = match UdpSocket::bind(format!("{}:{}", options.local_ip, options.local_port)) {
                Ok(socket) => socket,
                Err(error) => { println!("Could not create a UDP socket: {}", error); return; },
            };

            match socket.connect(format!("{}:{}", options.syslog_ip, options.syslog_port)) {
                Ok(())  =>{

                    println!("Successfully connected to the server");
                    socket.send(message.as_bytes());
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
                    stream.write_all(message.as_bytes()).unwrap();
                },
                Err(error) => {

                    println!("An error happened when sending data to the server: {}", error);
                }
            }

        }
    }
}

//arp spoofing detector
fn detector(options: LoggerOptions) -> Result<(), Box<dyn std::error::Error>> {
    
    if options.proto == Proto::Udp {

    } else {

    }

    
    let mut arp_cache: HashMap<Ipv4Addr, String> = HashMap::new();
    loop {
        println!("The detector loop has started");
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
                    let bytes = json_message.as_bytes();

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
            let bytes = json_message.as_bytes();

            warning(&options, json_message);
            
        }

        std::thread::sleep(std::time::Duration::from_secs_f32(options.timeout));
    }
}



//structure that handles CLI arguments/flags
#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Cli {
    #[arg(short = 'i', long)]
    install_service: bool,
    #[arg(short = 'c', long)]
    check_service: bool,
    #[arg(short = 'd', long)]
    delete_service: bool,
    #[arg(short = 'x', long)]
    start_service: bool,
    #[arg(short = 's', long)]
    stop_service: bool,
    #[clap(short, long, default_value="tcp")]
    proto: Proto,
    #[arg(short = 'a', long, default_value_t = Ipv4Addr::from_str("127.0.0.1").unwrap())]
    syslog_ip: Ipv4Addr,
    #[arg(short = 'z', long, default_value_t = String::from("1468"))]
    syslog_port: String,
    #[arg(short, long, default_value_t = Ipv4Addr::from_str("127.0.0.1").unwrap())]
    local_ip: Ipv4Addr,
    #[arg(short = 'b', long, default_value_t = String::from("9999"))]
    local_port: String,
    #[arg(short, long, default_value_t = 3.0)]
    timeout: f32,   
}


fn check_service_installed() -> bool {
    let  check_service_command = "& { $service = Get-Service -Name \"ArpSpoofDetectService\" -ErrorAction SilentlyContinue ; Write-Output $service.Length }";
    
    let output =  Command::new("powershell")
        .args(["-Command", check_service_command])
        .output()
        .expect("Failed to execute the checking command");

    let content = str::from_utf8(&output.stdout).unwrap();
    content.contains("1")
}





//the main function
fn main() -> Result<(), Box<dyn Error>>{
    let install_service_command = "New-Service -Name \"ArpSpoofDetectService\" -DisplayName \"ARP spoofing detector service\" -Description \"A service that detects ARP spoofing in your network\" -StartupType Manual -BinaryPathName \"arp-spoofing-detector.exe\"".split_whitespace();
    let start_service_command = "Start-Service -Name \"ArpSpoofDetectService\"".split_whitespace();
    let stop_service_command = "Stop-Service -Name \"ArpSpoofDetectService\"".split_whitespace();
    let delete_service_command = "sc.exe Delete \"ArpSpoofDetectService\"".split_whitespace();

    let cli = Cli::parse();
    //println!("{}", cli.syslog_ip);
    //println!("{}", cli.syslog_port);

    if cli.install_service {
        Command::new("powershell")
            .args(install_service_command)
            .output()
            .expect("Failed to execute the install command");
    } else if cli.check_service {
        if check_service_installed() {
            println!("The \"ArpSpoofDetectService\" service is installed")
        } else {
            println!("The \"ArpSpoofDetectService\" service is not installed")
        }
    } else if cli.delete_service {
        if !check_service_installed() {
            panic!("Cannot delete service: Not Installed")
        } else {
            Command::new("powershell")
            .args(delete_service_command)
            .output()
            .expect("Failed to execute the delete service command");
        }
    } else if cli.start_service {
        Command::new("powershell")
            .args(start_service_command)
            .output()
            .expect("Failed to execute the start service command");
    } else if cli.stop_service {
        Command::new("powershell")
            .args(stop_service_command)
            .output()
            .expect("Failed to execute the stop service command");
    } else {

        let options = LoggerOptions {
            syslog_ip: cli.syslog_ip.to_string(),
            syslog_port: cli.syslog_port,
            proto: cli.proto,
            local_ip: cli.local_ip.to_string(),
            local_port: cli.local_port,
            timeout: cli.timeout,
        };

        return detector(options);
        
    }
    Ok(())
}
