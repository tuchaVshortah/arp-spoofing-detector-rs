# arp-spoofing-detector-rs

A program that detects ARP spoofing attack in local network and reports it to the specified log server.


## Installation


Install arp-spoofing-detector-rs with git

```powershell
  git clone https://github.com/tuchaVshortah/arp-spoofing-detector-rs.git
```

Install Rust from the official site

```powershell
    https://www.rust-lang.org/tools/install
```

Compile

```powershell
    cargo build --release
```

Enjoy!

```powershell
    arp-spoofing-detector.exe
```


## Usage

```powershell
    .\arp-spoofing-detector.exe -h
ARP spoofing detector program

Usage: arp-spoofing-detector.exe [OPTIONS]

Options:
  -i, --install-service            Installs ArpSpoofDetectService with launch arguments
  -u, --uninstall-service          Uninstalls the service
  -r, --run-service                Runs the service
  -p, --proto <PROTO>              Specifies which protocol to use. Can be tcp or udp (case sensitive) [default: tcp]
      --syslog-ip <SYSLOG_IP>      Takes IP address of the Syslog server [default: 127.0.0.1]
      --syslog-port <SYSLOG_PORT>  Specifies the server port to connect to [default: 1468]
      --local-ip <LOCAL_IP>        Takes IP address of the local machine. Required when udp is used [default: 127.0.0.1]
      --local-port <LOCAL_PORT>    Specifies the local port to use. Required when udp is used [default: 9999]
      --timeout <TIMEOUT>          [default: 3]
  -h, --help                       Print help
  -V, --version                    Print version
```


## Examples

### CLI

Send logs to the remote using TCP:

```powershell
    .\arp-spoofing-detector.exe -p tcp --syslog-ip <remote-syslog-ip> --syslog-port <remote-syslog-port>
```

Send logs to the remote using UDP:

```powershell
    .\arp-spoofing-detector.exe -p udp --local-ip <local-machine-ip> --local-port <local-machine-port> --syslog-ip <remote-syslog-ip> --syslog-port <remote-syslog-port>
```

### Windows service

Install ArpSpoofDetectService with the following launch arguments

```powershell
    .\arp-spoofing-detector.exe -i -p tcp --syslog-ip 127.0.0.1 --syslog-port 1469 --timeout 2
```

Start and stop ArpSpoofDetectService using the Services application


![image](https://github.com/tuchaVshortah/arp-spoofing-detector-rs/assets/71591558/c8443a0a-c27f-42d7-a432-4b79a062fd78)


Uninstall ArpSpoofDetectService

```powershell
    .\arp-spoofing-detector.exe -u
```
