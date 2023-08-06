use std::collections::HashMap;
use std::net::Ipv4Addr;
use std::{process::Command};
use serde_json::json;
use encoding_rs::*;
use crate::utils::*;
use crate::remote::*;

//arp spoofing detector
pub fn detector(options: &LoggerOptions) {
    
    let mut arp_cache: HashMap<Ipv4Addr, String> = HashMap::new();

    let output = Command::new("arp")
        .arg("-a")
        .output()
        .expect("Failed to execute command");

    let (cow, encoding_used, _) = UTF_8.decode(&output.stdout);

    #[cfg(debug_assertions)]
    {
        println!("Shell command output converted from: {}", encoding_used.name());
    }

    let arp_table = cow;

    /*
    let mut skip = false;

    let arp_table = str::from_utf8(&output.stdout).unwrap_or_else(|err| {
        eprintln!("Couldn't collect ARP data. Ensure your encoding is correct (UTF-8)");
        eprintln!("Full Error: {}", err);
        skip = true;
        ""
    });

    if skip {
        return;
    }
    */

    let mut is_spoofed = false;

    for line in arp_table.lines() {

        let parts: Vec<&str> = line.split_whitespace().collect();

        if parts.len() == 3 {

            let ip = parts[0].parse::<Ipv4Addr>().unwrap();
            let mac = parts[1].to_string();

            if arp_cache.contains_key(&ip) && arp_cache.get(&ip).unwrap() != &mac {

                #[cfg(debug_assertions)]
                {
                    println!("ARP spoofing detected for IP address {}", ip);
                }
                

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

        #[cfg(debug_assertions)]
        {
            println!("No ARP spoofing detected");
        }
        
        let mut message = HashMap::new();
        message.insert("description", "ARP spoofing not detected");

        let json_message = json!(message).to_string();

        warning(&options, json_message);
        
    }
    
}