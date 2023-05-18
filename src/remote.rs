use std::io::Write;
use std::net::{UdpSocket, TcpStream};
use crate::utils::*;

pub fn warning(options: &LoggerOptions, message: String) {
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