mod json_parser;

use clap::Parser;
use std::fs::{read_dir, File};
use std::io::BufReader;
use std::io::{stdin, Read, Result as IoResult};
use std::path::PathBuf;

use crate::json_parser::JsonReader;

#[derive(Parser)]
#[command(author, version, about, long_about = Some("JSONs AWK"))]
struct Cli {
    /// Input files
    files: Vec<PathBuf>,
}

fn main() -> IoResult<()> {
    let cli = Cli::parse();
    if cli.files.is_empty() {
        read_input(stdin())?;
    } else {
        for file in cli.files {
            read_file(&file)?;
        }
    }
    Ok(())
}

fn read_file(file: &PathBuf) -> IoResult<()> {
    if !file.exists() {
        panic!("File {:?} not exists", file);
    }
    if file.is_dir() {
        for entry in read_dir(file)? {
            let path = entry?.path();
            read_file(&path)?;
        }
    } else {
        println!("QQQ reading file: {:?}", file);
        let reader = File::open(file)?;
        let reader = BufReader::new(reader);
        read_input(reader)?;
    }
    Ok(())
}

fn read_input<R: Read>(input: R) -> IoResult<()> {
    let mut reader = JsonReader::new(input);
    loop {
        let val = reader.next_value()?;
        println!("val: {:?}", val);
        if val.is_none() {
            return Ok(());
        }
    }
}
