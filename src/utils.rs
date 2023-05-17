use std::str::{self, FromStr};
use std::fmt::Display;
use crate::cli::*;

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum Proto {
    Udp,
    Tcp,
}

impl Default for Proto {
    fn default() -> Self {
        Proto::Tcp
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

pub trait FromCli {
    
    fn from_cli(cli: &Cli) -> Self;

}

pub struct LoggerOptions {
    //log levels
    //min_level: SyslogLevels,
    //max_level: SyslogLevels,

    //remote machine
    pub syslog_ip: String,
    pub syslog_port: String,

    //protocol used to establish a connection
    pub proto: Proto,

    //local machine
    pub local_ip: String,
    pub local_port: String,

    //time to sleep between requests to a syslog remote
    pub timeout: f32,

}

impl Display for LoggerOptions {
    
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {

        if self.proto == Proto::Tcp {

            write!(f, "{}", format!("--syslog-ip {} --syslog-port {} --proto {} --timeout {}", self.syslog_ip.as_str(), self.syslog_port.as_str(), self.proto, self.timeout.to_string().as_str()))

        } else {

            write!(f, "{}", format!("--syslog-ip {} --syslog-port {} --proto {} --local-ip {} --local-port {} --timeout {}", self.syslog_ip.as_str(), self.syslog_port.as_str(), self.proto, self.local_ip.as_str(), self.local_port.as_str(), self.timeout.to_string().as_str()))

        }
    }
}

impl FromCli for LoggerOptions {

    fn from_cli(cli: &Cli) -> Self {

        LoggerOptions {
            syslog_ip: cli.syslog_ip.to_string().clone(),
            syslog_port: cli.syslog_port.clone(),
            proto: cli.proto,
            local_ip: cli.local_ip.to_string().clone(),
            local_port: cli.local_port.clone(),
            timeout: cli.timeout,
        }
    }

}