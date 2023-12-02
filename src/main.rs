mod functions;
mod functions_definitions;
mod json_parser;
mod json_value;
mod output;
mod printer;
mod reader;
mod selection;

use clap::Parser;
use functions_definitions::print_help;
use json_parser::JsonParserError;
use output::{get_output, Output};
use selection::{Get, Selection};
use std::fmt::Error as FormatError;
use std::fs::read_dir;
use std::io::Error as IoEror;
use std::io::Read;
use std::path::PathBuf;
use std::str::FromStr;
use thiserror::Error;

use crate::json_parser::JsonParser;
use crate::reader::{from_file, from_std_in, Reader};

#[derive(Parser)]
#[command(author, version, about, long_about = Some("JSONs AWK"))]
struct Cli {
    /// Input files
    files: Vec<PathBuf>,

    /// What to do on error
    #[arg(long, default_value_t = OnError::Ignore)]
    #[clap(value_enum)]
    on_error: OnError,

    /// How to display the output
    #[arg(long, short, default_value_t = OutputStyle::OneLineJson)]
    #[clap(value_enum)]
    output_style: OutputStyle,

    /// What to output
    #[arg(long, short, value_parser = Selection::from_str)]
    select: Vec<Selection>,

    /// Row seperator
    #[arg(long, short, default_value = "\n")]
    row_seperator: String,

    /// List of available functions
    #[arg(long, short, default_value_t = false)]
    available_functions: bool,
}

#[derive(clap::ValueEnum, Debug, Clone, PartialEq)]
#[clap(rename_all = "kebab_case")]
enum OnError {
    Ignore,
    Panic,
    Stderr,
    Stdout,
}

#[derive(clap::ValueEnum, Debug, Clone, PartialEq, Copy)]
#[clap(rename_all = "kebab_case")]
pub enum OutputStyle {
    Json,
    OneLineJson,
    ConsiseJson,
    Csv,
    Text,
}

fn main() -> Result<()> {
    let cli = Cli::parse();
    cli.go()
}

impl Cli {
    fn go(&self) -> Result<()> {
        if self.available_functions {
            print_help();
            return Ok(());
        }
        let rows_titles = self
            .select
            .iter()
            .map(|select| select.name().clone())
            .collect();
        let output = get_output(self.output_style, rows_titles, self.row_seperator.clone());
        if self.files.is_empty() {
            let mut reader = from_std_in();
            self.read_input(&mut reader, output.as_ref())?;
        } else {
            for file in self.files.clone() {
                self.read_file(&file, output.as_ref())?;
            }
        }
        Ok(())
    }

    fn read_file(&self, file: &PathBuf, output: &dyn Output) -> Result<()> {
        if !file.exists() {
            panic!("File {:?} not exists", file);
        }
        if file.is_dir() {
            for entry in read_dir(file)? {
                let path = entry?.path();
                self.read_file(&path, output)?;
            }
        } else {
            let mut reader = from_file(file)?;
            self.read_input(&mut reader, output)?;
        }
        Ok(())
    }
    fn read_input<R: Read>(&self, reader: &mut Reader<R>, output: &dyn Output) -> Result<()> {
        loop {
            match reader.next_json_value() {
                Ok(Some(val)) => {
                    if self.select.is_empty() {
                        output.output_row(vec![Some(val)])?;
                    } else {
                        let val = Some(val);
                        let row = self.select.iter().map(|v| v.get(&val)).collect();
                        output.output_row(row)?;
                    }
                }
                Ok(None) => {
                    return Ok(());
                }
                Err(e) => {
                    if !e.can_recover() {
                        return Err(e.into());
                    }
                    match self.on_error {
                        OnError::Ignore => {}
                        OnError::Panic => {
                            return Err(e.into());
                        }
                        OnError::Stdout => println!("error:{}", e),
                        OnError::Stderr => eprintln!("error:{}", e),
                    }
                }
            };
        }
    }
}

type Result<T> = std::result::Result<T, MainError>;

#[derive(Debug, Error)]
enum MainError {
    #[error("{0}")]
    Json(#[from] JsonParserError),
    #[error("{0}")]
    Format(#[from] FormatError),
    #[error("{0}")]
    Io(#[from] IoEror),
}
