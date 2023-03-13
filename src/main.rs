use std::collections::HashMap;
use std::net::Ipv4Addr;
use std::process::Command;
use std::str;
use clap::Parser;


//code for service runners
#[macro_use]
extern crate windows_service;

use std::ffi::OsString;
use windows_service::service_dispatcher;

define_windows_service!(ffi_service_main, arp_service_main);


fn arp_service_main(arguments: Vec<OsString>) {
    detector();
}



//arp spoofing detector
fn detector() {
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
    service: bool,
}


//the main function
fn main() -> Result<(), windows_service::Error> {
    let cli = Cli::parse();
    if cli.service {
        // Register generated `ffi_service_main` with the system and start the service, blocking
        // this thread until the service is stopped.
        service_dispatcher::start("arpdetectorservice", ffi_service_main)?;
        Ok(())
    } else {
        detector();
        Ok(())
    }
}

