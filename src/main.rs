use clap::Parser;
use jawk::Cli;
use jawk::{Master, Result};
use std::cell::RefCell;
use std::rc::Rc;

fn main() -> Result<()> {
    let cli = Cli::parse();
    let stdout = Rc::new(RefCell::new(std::io::stdout()));
    let stdin = Box::new(std::io::stdin);
    let stderr = Rc::new(RefCell::new(std::io::stdout()));

    let master = Master::new(cli, stdout, stderr, stdin);
    master.go()
}
