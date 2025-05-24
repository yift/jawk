use clap::Parser;
use jawk::Cli;
use jawk::go;
use std::cell::RefCell;
use std::rc::Rc;

fn main() {
    let cli = Cli::parse();
    let stdout = Rc::new(RefCell::new(std::io::stdout()));
    let stdin = Box::new(std::io::stdin);
    let stderr = Rc::new(RefCell::new(std::io::stdout()));

    if let Err(err) = go(cli, stdout, stderr, stdin) {
        eprintln!("{err}");
        std::process::exit(-1);
    }
}
