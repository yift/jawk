mod const_getter;
mod duplication_remover;
mod extractor;
mod filter;
mod functions;
mod functions_definitions;
mod grouper;
mod json_parser;
mod json_value;
mod output;
mod pre_sets;
mod printer;
mod processor;
mod reader;
mod selection;
mod sorters;
mod variables_extractor;

use clap::Parser;
use duplication_remover::Uniquness;
use filter::Filter;
use functions_definitions::print_help;
use grouper::Grouper;
use json_parser::JsonParserError;
use pre_sets::PreSetCollection;
use pre_sets::PreSetParserError;
use processor::{Context, Process, ProcessError, Titles};
use selection::Selection;
use selection::SelectionParseError;
use sorters::Sorter;
use sorters::SorterParserError;
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
    #[arg(long, short, visible_alias = "select")]
    choose: Vec<String>,

    /// Filter the output
    #[arg(long, short, visible_alias = "where")]
    filter: Option<String>,

    /// Group the output by.
    /// Be careful, the grouping is done in memory
    #[arg(long, short, visible_alias = "groupBy")]
    group_by: Option<String>,

    /// How to order the output
    /// Be careful, the sorting is done in memory
    #[arg(
        long,
        short,
        visible_alias = "sortBy",
        visible_alias = "order-by",
        visible_alias = "orderBy"
    )]
    sort_by: Vec<String>,

    /// Row seperator
    #[arg(long, short, default_value = "\n")]
    row_seperator: String,

    /// List of available functions
    #[arg(long, short, default_value_t = false)]
    available_functions: bool,

    /// Avoid posting the same output more than once.
    /// Be careful, the data is kept in memory.
    #[arg(long, short)]
    unique: bool,

    /// Predefine variables.
    /// Use key=value format. That is, `name="hello"`.
    #[arg(long)]
    set: Vec<String>,
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
        let mut process = self.output_style.get_processor(self.row_seperator.clone());
        if let Some(group_by) = &self.group_by {
            let group_by = Grouper::from_str(group_by)?;
            process = group_by.create_process(process);
        }
        for sorter in &self.sort_by {
            let sorter = Sorter::from_str(sorter)?;
            process = sorter.create_processor(process);
        }
        if self.unique {
            process = Uniquness::create_process(process);
        }
        for selection in &self.choose {
            let selection = Selection::from_str(selection)?;
            process = selection.create_process(process);
        }
        if let Some(filter) = &self.filter {
            let filter = Filter::from_str(filter)?;
            process = filter.create_process(process);
        }
        process = self.set.create_process(process)?;
        process.start(Titles::default())?;

        if self.files.is_empty() {
            let mut reader = from_std_in();
            self.read_input(&mut reader, process.as_mut())?;
        } else {
            for file in self.files.clone() {
                self.read_file(&file, process.as_mut())?;
            }
        }
        process.complete()?;
        Ok(())
    }

    fn read_file(&self, file: &PathBuf, process: &mut dyn Process) -> Result<()> {
        if !file.exists() {
            panic!("File {:?} not exists", file);
        }
        if file.is_dir() {
            for entry in read_dir(file)? {
                let path = entry?.path();
                self.read_file(&path, process)?;
            }
        } else {
            let mut reader = from_file(file)?;
            self.read_input(&mut reader, process)?;
        }
        Ok(())
    }
    fn read_input<R: Read>(&self, reader: &mut Reader<R>, process: &mut dyn Process) -> Result<()> {
        loop {
            match reader.next_json_value() {
                Ok(Some(val)) => {
                    let context = Context::new_with_input(val);
                    process.process(context)?;
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
    SelectionParse(#[from] SelectionParseError),
    #[error("{0}")]
    SorterParse(#[from] SorterParserError),
    #[error("{0}")]
    Io(#[from] IoEror),
    #[error("{0}")]
    Processor(#[from] ProcessError),
    #[error("{0}")]
    PreSet(#[from] PreSetParserError),
}
