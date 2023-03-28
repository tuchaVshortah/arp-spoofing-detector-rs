use std::collections::HashMap;
use std::net::Ipv4Addr;
use std::process::Command;
use std::str;
use clap::Parser;


async fn logsender(syslog_ip: &String, syslog_port: &String, message: &HashMap<String, String>) -> Result<(), Box<dyn std::error::Error>> {
    let url = format!("http://{}:{}/", syslog_ip, syslog_port);

    assert_eq!(url, String::from("http://0.0.0.0:514/"));

    let client = reqwest::Client::new();

    let resp = client.post(url).json(message).send().await?;
    println!("{:#?}", resp);
    Ok(())
}



//arp spoofing detector
fn detector(syslog_ip: String, syslog_port: String) {
    let mut arp_cache: HashMap<Ipv4Addr, String> = HashMap::new();
    
    loop {
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
                    is_spoofed = true;
                }

                arp_cache.insert(ip, mac);
            }
        }

        if !is_spoofed {
            println!("No ARP spoofing detected");
        }

        std::thread::sleep(std::time::Duration::from_secs(3));
    }
}



//structure that handles CLI arguments/flags
#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Cli {
    #[arg(short, long)]
    install_service: bool,
    #[arg(short, long)]
    check_service: bool,
    #[arg(short, long)]
    delete_service: bool,
    #[arg(short, long)]
    start_service: bool,
    #[arg(short, long)]
    stop_service: bool,
    #[arg(short, long, default_value_t = String::from("0.0.0.0"))]
    syslog_ip: String,
    #[arg(short, long, default_value_t = String::from("514"))]
    syslog_port: String,    
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
fn main() {
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
        detector(cli.syslog_ip, cli.syslog_port)
    }
}
