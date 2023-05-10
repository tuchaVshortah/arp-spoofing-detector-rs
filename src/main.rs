use std::collections::HashMap;
use std::env;
use std::error::Error;
use std::io::Write;
use std::net::{Ipv4Addr, UdpSocket, TcpStream};
use std::process::Command;
use std::str::{self, FromStr};
use std::fmt::Display;
use std::sync::mpsc;
use clap::Parser;
use serde_json::json;

#[macro_use]
extern crate windows_service;

use std::ffi::OsString;
use windows_service::service_dispatcher;
use std::time::Duration;
use windows_service::service::{
    ServiceControl, ServiceControlAccept, ServiceExitCode, ServiceState, ServiceStatus,
    ServiceType,
};
use windows_service::service_control_handler::{self, ServiceControlHandlerResult};

//fix detector_service name
//reason -> function does not exist yet
//fix: create a function 
define_windows_service!(ffi_service_main, detector_service_main);

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
fn detector(options: LoggerOptions) -> Result<(), Box<dyn std::error::Error>> {
    
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

    std::thread::sleep(std::time::Duration::from_secs_f32(options.timeout));
}

fn run_detector_service(options: LoggerOptions) {

     // Create a channel to be able to poll a stop event from the service worker loop.
     let (shutdown_tx, shutdown_rx) = mpsc::channel();

     // Define system service event handler that will be receiving service events.
     let event_handler = move |control_event| -> ServiceControlHandlerResult {
         match control_event {
             // Notifies a service to report its current status information to the service
             // control manager. Always return NoError even if not implemented.
             ServiceControl::Interrogate => ServiceControlHandlerResult::NoError,

             // Handle stop
             ServiceControl::Stop => {
                 shutdown_tx.send(()).unwrap();
                 ServiceControlHandlerResult::NoError
             }

             _ => ServiceControlHandlerResult::NotImplemented,
         }
     };

     // Register system service event handler.
     // The returned status handle should be used to report service status changes to the system.
     let status_handle = service_control_handler::register("ArpSpoofDetectService", event_handler)?;

     // Tell the system that service is running
     status_handle.set_service_status(ServiceStatus {
         service_type: ServiceType::OWN_PROCESS,
         current_state: ServiceState::Running,
         controls_accepted: ServiceControlAccept::STOP,
         exit_code: ServiceExitCode::Win32(0),
         checkpoint: 0,
         wait_hint: Duration::default(),
         process_id: None,
     })?;

     loop {
         detector(options);
         // Poll shutdown event.
         match shutdown_rx.recv_timeout(Duration::from_secs(1)) {
             // Break the loop either upon stop or channel disconnect
             Ok(_) | Err(mpsc::RecvTimeoutError::Disconnected) => break,

             // Continue work if no events were received within the timeout
             Err(mpsc::RecvTimeoutError::Timeout) => (),
         };
     }

     // Tell the system that service has stopped.
     status_handle.set_service_status(ServiceStatus {
         service_type: ServiceType::OWN_PROCESS,
         current_state: ServiceState::Stopped,
         controls_accepted: ServiceControlAccept::empty(),
         exit_code: ServiceExitCode::Win32(0),
         checkpoint: 0,
         wait_hint: Duration::default(),
         process_id: None,
     })?;

     Ok(())
}

fn detector_service_main(arguments: Vec<OsString>) {

    let cli = Cli::parse();

    let options = LoggerOptions {
        syslog_ip: cli.syslog_ip.to_string(),
        syslog_port: cli.syslog_port,
        proto: cli.proto,
        local_ip: cli.local_ip.to_string(),
        local_port: cli.local_port,
        timeout: cli.timeout,
    };

    if let Err(error) = run_detector_service(options) {
        println!("Something bad happened when statring the service")
    }

}

//structure that handles CLI arguments/flags
#[derive(Parser)]
#[command(author = "tuchaVshortah", version = "1.0.1", about = "ARP spoofing detector program", long_about = None)]
struct Cli {

    #[arg(short = 'i', long, help="Installs a service that allows the program to run as a background process")]
    install_service: bool,

    #[arg(short = 'c', long, help="Checks if service is installed")]
    check_service: bool,

    #[arg(short = 'd', long, help="Deletes the service only if it has already been installed")]
    delete_service: bool,

    #[arg(short = 'r', long, help="Reinstalls the service only if it has already been installed (simple wrapper for --install-service and --delete-service)")]
    reinstall_service: bool,

    #[arg(short = 'x', long, help="Starts the program in background")]
    start_service: bool,

    #[arg(short = 's', long, help="Stops the background process")]
    stop_service: bool,

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

fn install_service(cli: &Cli) {

    let cwd = env::current_dir().unwrap().into_os_string().into_string().unwrap();

        let install_service_string;
        
        match cli.proto {
            Proto::Udp => {
                install_service_string = format!("New-Service -Name \"ArpSpoofDetectService\" -DisplayName \"ARP spoofing detector service\" -Description \"A service that detects ARP spoofing in your network\" -StartupType Automatic -BinaryPathName \"{}\\arp-spoofing-detector.exe -p udp --local-ip {} --local-port {} --syslog-ip {} --syslog-port {} --timeout {}\"", cwd, cli.local_ip, cli.local_port, cli.syslog_ip, cli.syslog_port, cli.timeout);
            },

            Proto::Tcp => {
                install_service_string = format!("New-Service -Name \"ArpSpoofDetectService\" -DisplayName \"ARP spoofing detector service\" -Description \"A service that detects ARP spoofing in your network\" -StartupType Automatic -BinaryPathName \"{}\\arp-spoofing-detector.exe -p tcp --syslog-ip {} --syslog-port {} --timeout {}\"", cwd, cli.syslog_ip, cli.syslog_port, cli.timeout);
            }
        }
        
        let install_service_command = install_service_string.split_whitespace();


        Command::new("powershell")
            .args(install_service_command)
            .output()
            .expect("Failed to execute the install command");

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

fn delete_service() {

    let delete_service_command = "sc.exe Delete \"ArpSpoofDetectService\"".split_whitespace();

    Command::new("powershell")
        .args(delete_service_command)
        .output()
        .expect("Failed to execute the delete service command");

}

fn reinstall_service(cli: &Cli) {

    if !check_service_installed() {

        panic!("Could not reinstall the service: Not Installed")

    } else {

        delete_service();

    }

    install_service(cli);

}



//the main function
fn main() -> Result<(), windows_service::Error>{

    let cli = Cli::parse();

    if cli.install_service {

        install_service(&cli);

    } else if cli.check_service {

        if check_service_installed() {

            println!("The \"ArpSpoofDetectService\" service is installed");

        } else {

            println!("The \"ArpSpoofDetectService\" service is not installed");

        }

    } else if cli.delete_service {

        if !check_service_installed() {

            panic!("Could not delete the service: Not Installed")

        } else {

            delete_service();

        }

    } else if cli.reinstall_service {

        reinstall_service(&cli);

    } else if cli.start_service {

        //let start_service_command = "Start-Service -Name \"ArpSpoofDetectService\"".split_whitespace();

        //Command::new("powershell")
        //    .args(start_service_command)
        //    .output()
        //    .expect("Failed to execute the start service command");

        service_dispatcher::start("ArpSpoofDetectService", ffi_service_main)?;

    } else if cli.stop_service {

        let stop_service_command = "Stop-Service -Name \"ArpSpoofDetectService\"".split_whitespace();

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

        /*
        let mut status = unsafe { mem::zeroed::<Services::SERVICE_STATUS>() };
        status.dwServiceType = Services::SERVICE_WIN32_OWN_PROCESS;
        status.dwCurrentState = Services::SERVICE_RUNNING;
        status.dwControlsAccepted = Services::SERVICE_ACCEPT_STOP;
        status.dwWin32ExitCode = 0;
        status.dwServiceSpecificExitCode = 0;
        status.dwCheckPoint = 0;
        status.dwWaitHint = 3000;
        */
        
        //Services::SetServiceStatus(Services::SERVICE_STATUS_HANDLE, status);
        detector(options).unwrap();
        
    }

    Ok(())

}
