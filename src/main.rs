mod utils;
mod local;
mod remote;
mod winservice;
mod cli;

use utils::*;
use local::*;
use winservice::{install_service, uninstall_service, syslog_service};
use cli::*;

#[allow(unused, unused_imports, unused_variables, dead_code)]

//the main function
fn main() -> Result<(), windows_service::Error> {
    let cli = Cli::parse();
    
    let options = LoggerOptions::from_cli(&cli);

    if cli.install_service {

        install_service::install(format!("--run-service {}", options).split_whitespace().map(std::ffi::OsString::from).collect())?;
    
    } else if cli.uninstall_service {

        uninstall_service::uninstall()?;

    } else if cli.run_service {

        println!("Trying to start the service...");
        syslog_service::run()?;
        
    } else {

        loop {
            detector(&options);
            std::thread::sleep(std::time::Duration::from_secs_f32(cli.timeout));
        }

    }

    Ok(())

}
