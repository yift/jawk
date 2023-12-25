use clap::Parser;
use jawk::Cli;
use jawk::{Master, Result};
use std::sync::Arc;
use std::sync::Mutex;

fn main() -> Result<()> {
    let cli = Cli::parse();
    let stdout = Arc::new(Mutex::new(std::io::stdout()));
    let stdin = Box::new(std::io::stdin);
    let stderr = Arc::new(Mutex::new(std::io::stdout()));

    let master = Master::new(cli, stdout, stderr, stdin);
    master.go()
}
