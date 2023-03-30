use std::collections::HashMap;
use std::error::Error;
use std::net::Ipv4Addr;
use std::process::Command;
use std::str::{self, FromStr};
use std::fmt::Display;
use clap::Parser;
use serde_json::json;
use syslog::{Facility, Formatter5424};
use log::{SetLoggerError, LevelFilter, info};

#[allow(unused, unused_variables, dead_code)]

fn logsender(syslog_ip: &String, syslog_port: &String, proto: &Proto, severity: SyslogLevels, message: &HashMap<&str, &str>) -> Result<(), Box<dyn std::error::Error>> {
    let json_message = json!(message).to_string();

    /*
    let send_tcp_logs = format!(
        "& {{ \
        Import-Module SyslogMessage; \
        $server = \"{}\"; \
        $port = \"{}\"; \
        $message = \"{}\"; \
        $facility = \"Local0\"; \
        $severity = \"{}\"; \
        $protocol = \"{}\"; \
        Send-SyslogMessage -Server $server -Port $port -Message $message -Facility $facility -Severity $severity -Protocol $protocol \
        }}",
        syslog_ip, syslog_port, json_message, severity, proto.to_string()
    );
    */
    
    Ok(())
}


//arp spoofing detector
fn detector(syslog_ip: String, syslog_port: String, proto: Proto, timeout: f32) -> Result<(), Box<dyn std::error::Error>> {
    
    let formatter = Formatter5424::default();
    let logger;
    
    if proto == Proto::Udp {

        logger = match syslog::udp(formatter, local_ip, format!("{}:{}", syslog_ip, syslog_port)) {

            Err(e) => { println!("impossible to connect to syslog: {:?}", e); return Ok(()); },
            Ok(logger) => logger,

        };

    } else {

        logger = match syslog::tcp(formatter, format!("{}:{}", syslog_ip, syslog_port)) {

            Err(e) => { println!("impossible to connect to syslog: {:?}", e); return Ok(()); },
            Ok(logger) => logger,

        };

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
                    if let Err(error) = logsender(&syslog_ip, &syslog_port, &proto, SyslogLevels::Warning,&message) {
                        println!("Error in the loop: {}", error);
                        return Err(error);
                    }

                    is_spoofed = true;
                }
                arp_cache.insert(ip, mac);
            }
        }

        if !is_spoofed {
            println!("No ARP spoofing detected");
            
            let mut message = HashMap::new();
            message.insert("description", "ARP spoofing not detected");
            if let Err(error) = logsender(&syslog_ip, &syslog_port, &proto,SyslogLevels::Informational, &message) {
                println!("Error in the loop: {}", error);
                return Err(error);
            }
        }

        std::thread::sleep(std::time::Duration::from_secs_f32(timeout));
    }
}


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


enum SyslogLevels {
    Emergency,
    Alert,
    Critical,
    Error,
    Warning,
    Notice,
    Informational,
    Debug 
}

impl Display for SyslogLevels {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        
        match self {

            SyslogLevels::Emergency => write!(f, "Emergency"),
            SyslogLevels::Alert => write!(f, "Alert"),
            SyslogLevels::Critical => write!(f, "Critical"),
            SyslogLevels::Error => write!(f, "Error"),
            SyslogLevels::Warning => write!(f, "Warning"),
            SyslogLevels::Notice => write!(f, "Notice"),
            SyslogLevels::Informational => write!(f, "Informational"),
            SyslogLevels::Debug => write!(f, "Debug"),

        }

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
    #[clap(short, long, value_name="proto", default_value="udp")]
    proto: Proto,
    #[arg(short = 'a', long, default_value_t = Ipv4Addr::from_str("127.0.0.1").unwrap())]
    syslog_ip: Ipv4Addr,
    #[arg(short = 'p', long, default_value_t = String::from("1468"))]
    syslog_port: String,
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

        return detector(cli.syslog_ip.to_string(), cli.syslog_port, cli.proto, cli.timeout);
        
    }
    Ok(())
}
