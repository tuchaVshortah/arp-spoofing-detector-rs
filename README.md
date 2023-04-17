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
    -i, --install-service            Installs a service that allows the program to run as a background process
    -c, --check-service              Checks if service is installed
    -d, --delete-service             Deletes the service only if it has already been installed
    -x, --start-service              Starts the program in background
    -s, --stop-service               Stops the background process
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

Install a service that is required to run the program in background:

```powershell
    .\arp-spoofing-detector.exe --install-service
```

Check if the service has been installed successfully:

```powershell
    .\arp-spoofing-detector.exe --check-service
```

Start the service:

```powershell
    .\arp-spoofing-detector.exe --start-service
```

Stop the service:

```powershell
    .\arp-spoofing-detector.exe --stop-service
```

Delete the service:

```powershell
    .\arp-spoofing-detector.exe --delete-service
```

Connect to the local syslog server at port 1468 using TCP (using default options):

```powershell
    .\arp-spoofing-detector.exe
```

Do the same, but use UDP:

```powershell
    .\arp-spoofing-detector.exe -p udp
```

Connect to a remote syslog server using a specific port with TCP:

```powershell
    .\arp-spoofing-detector.exe -p tcp --syslog-ip <remote-syslog-ip> --syslog-port <remote-syslog-port>
```

Do the same, but use UDP:

```powershell
    .\arp-spoofing-detector.exe -p udp --local-ip <local-machine-ip> --local-port <local-machine-port> --syslog-ip <remote-syslog-ip> --syslog-port <remote-syslog-port>
```
