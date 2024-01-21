mod additional_help;
mod const_getter;
mod duplication_remover;
mod extractor;
mod filter;
mod functions;
mod functions_definitions;
mod grouper;
mod input_context_extractor;
mod json_parser;
mod json_value;
mod limits;
mod merger;
mod output;
mod pre_sets;
mod printer;
mod processor;
mod reader;
mod regex_cache;
mod selection;
mod selection_extractor;
mod selection_help;
mod sorters;
mod splitter;
mod variables_extractor;

use additional_help::display_additional_help;
use clap::Parser;
use duplication_remover::Uniquness;
use filter::Filter;
use grouper::Grouper;
use json_parser::JsonParserError;
use json_value::JsonValue;
use limits::Limiter;
use merger::Merger;
use pre_sets::PreSetCollection;
use pre_sets::PreSetParserError;
use processor::ProcessDesision;
use processor::{Context, Process, ProcessError, Titles};
use regex_cache::RegexCache;
use selection::Selection;
use selection::SelectionParseError;
use sorters::Sorter;
use sorters::SorterParserError;
use splitter::Splitter;
use std::cell::RefCell;
use std::fmt::Error as FormatError;
use std::fs::read_dir;
use std::io::Error as IoEror;
use std::io::Read;
use std::path::PathBuf;
use std::rc::Rc;
use std::str::FromStr;
use thiserror::Error;

use crate::additional_help::create_possible_values;
use crate::json_parser::JsonParser;
use crate::output::OutputStyle;
use crate::reader::{from_file, from_std_in, Reader};

/// An AWK like toold for JSON input.
///
/// This tool should allow one to manipulate an input file that contains JSON values into CSV or JSON output.
#[derive(Parser)]
#[command(author, version, about)]
pub struct Cli {
    /// Input files.
    ///
    /// If ommited the standart in will be used.
    /// If any of the files is a directory, all it's files will be used.
    files: Vec<PathBuf>,

    /// What to do on error
    #[arg(long, default_value_t = OnError::Ignore)]
    #[clap(value_enum)]
    on_error: OnError,

    /// How to display the output
    #[arg(long, short, default_value_t = OutputStyle::OneLineJson)]
    #[clap(value_enum)]
    output_style: OutputStyle,

    /// What to output.
    ///
    /// Can be multpile selection. The expected format is `<selection>[=name]`.
    /// If name is ommited, the selection will be the name.
    /// See selection additional help for available selections format.
    ///
    /// For example: `--select .name.first=First Name`
    #[arg(long, short, visible_alias = "select")]
    choose: Vec<String>,

    /// Filter the output. The expected format is `<selection>`.
    ///
    /// If the filter is not true, the input will ignored.
    /// See selection additional help for available selections format.
    ///
    /// For example: `--filter=(or (= .name.first "John") (= .name.first "Jane"))`.
    #[arg(long, short, visible_alias = "where")]
    filter: Option<String>,

    /// Split the input by.
    ///
    /// The expected format is `<selection>`.
    /// If the output is not an array, the data will be ignored
    /// See selection additional help for available selections format.
    /// This will run before the filter.
    ///
    /// For example: `--split-by=.results`.
    #[arg(long, short, visible_alias = "split-by")]
    break_by: Option<String>,

    /// Group the output by.
    ///
    /// Be careful, the grouping is done in memory.
    /// The expected format is `<selection>`.
    /// If the output is not a string, the data will be ignored. One can use the `stringify` function if needed.
    /// See selection additional help for available selections format.
    /// Ommiting the selection will produce a list instead of an object.
    ///
    /// For example: `--group-by=.name.first`.
    #[arg(long, short, visible_alias = "combine", visible_alias = "merge")]
    group_by: Option<Option<String>>,

    /// How to order the output. Allow muttiploe sorting.
    ///
    /// Be careful, the sorting is done in memory.
    /// The expected format is `<selection>[=DESC]` or `<selection>[=ASC]`. If the directioon is ommited, `ASC` is assumed.
    /// The order is null, false, true, strings, numbers, objects, arrays.
    /// See selection additional help for available selections format.
    ///
    /// For example: `--sort-by=.name.last --sort-by=.name.first`.
    #[arg(long, short, visible_alias = "order-by")]
    sort_by: Vec<String>,

    /// How many results to skip (if any).
    ///
    #[arg(long, short = 'k', default_value_t = 0)]
    skip: u64,

    /// Maximal number of result to process.
    ///
    #[arg(long, short, visible_alias = "limit")]
    take: Option<u64>,

    /// Row seperator.
    ///
    /// How to seperate between each row. The default is new line, but one can use something like `--row_seperator="---\n" to use yaml style seperation.
    #[arg(long, short, default_value = "\n")]
    row_seperator: String,

    /// Additional help.
    ///
    /// Display additional help. Use the function name to get additional help on a specific function.
    #[arg(long, short, default_value = None, value_parser = create_possible_values())]
    additional_help: Option<String>,

    /// Avoid posting the same output more than once.
    ///
    /// Be careful, the data is kept in memory.
    #[arg(long, short)]
    unique: bool,

    /// Predefine variables and macros.
    ///
    /// One can define multiple variables and macros.
    /// The expected format is `key=value` for variables or `@key=value` for macros.
    ///
    /// For example: `--set one=1 --set pi=3.14 --set name="Name" --set @pirsquare=(* :pi . .)`.
    #[arg(long, short = 'e')]
    set: Vec<String>,

    /// Regular expression cache size.
    ///
    /// Regular expression compilation can time time. If you have a few repeating complex regular expression, you can set
    /// a cahce of compiled expressions, this will make sure that the offten used regular expression are compiled only once.
    /// Ommiting this setting will re-compile the regular expressions before any use (i.e. size 0).
    ///
    #[arg(long, default_value_t = 0)]
    regular_expression_cache_size: usize,

    /// Only accept objects and array
    ///
    /// Use this to ignore numbers, Booleans and nulls in the input stream and only treat
    /// objects and arrays (that is, as defined in RFC 4627).
    #[arg(long)]
    only_objects_and_arrays: bool,
}

#[derive(clap::ValueEnum, Debug, Clone, PartialEq)]
#[clap(rename_all = "kebab_case")]
enum OnError {
    /// Do nothing with the error.
    Ignore,
    /// Exit the process on the first error.
    Panic,
    /// Output the errors to stderr.
    Stderr,
    /// Output the errors to stdout.
    Stdout,
}

pub struct Master<R: Read> {
    cli: Cli,
    stdout: Rc<RefCell<dyn std::io::Write + Send>>,
    stderr: Rc<RefCell<dyn std::io::Write + Send>>,
    stdin: Box<dyn Fn() -> R>,
    regular_expression_cache: RegexCache,
}
impl<S: Read> Master<S> {
    pub fn new(
        cli: Cli,
        stdout: Rc<RefCell<dyn std::io::Write + Send>>,
        stderr: Rc<RefCell<dyn std::io::Write + Send>>,
        stdin: Box<dyn Fn() -> S>,
    ) -> Self {
        let regular_expression_cache = RegexCache::new(cli.regular_expression_cache_size);
        Master {
            cli,
            stdin,
            stderr,
            stdout,
            regular_expression_cache,
        }
    }
    pub fn go(&self) -> Result<()> {
        match &self.cli.additional_help {
            Some(help_type) => {
                display_additional_help(help_type);
                return Ok(());
            }
            None => {}
        }
        let mut process = self
            .cli
            .output_style
            .get_processor(self.cli.row_seperator.clone(), self.stdout.clone());
        if let Some(group_by) = &self.cli.group_by {
            if let Some(group_by) = group_by {
                let group_by = Grouper::from_str(group_by)?;
                process = group_by.create_process(process);
            } else {
                process = Merger::create_process(process);
            }
        }
        process = Limiter::create_process(self.cli.skip, self.cli.take, process);
        for sorter in &self.cli.sort_by {
            let sorter = Sorter::from_str(sorter)?;
            let max_size = self.cli.take.map(|take| (self.cli.skip + take) as usize);
            process = sorter.create_processor(process, max_size);
        }
        if self.cli.unique {
            process = Uniquness::create_process(process);
        }
        for selection in self.cli.choose.iter().rev() {
            let selection = Selection::from_str(selection)?;
            process = selection.create_process(process);
        }
        if let Some(filter) = &self.cli.filter {
            let filter = Filter::from_str(filter)?;
            process = filter.create_process(process);
        }
        if let Some(splitter) = &self.cli.break_by {
            let splitter = Splitter::from_str(splitter)?;
            process = splitter.create_process(process);
        }
        process = self.cli.set.create_process(process)?;
        process.start(Titles::default())?;

        let mut index = 0;
        if self.cli.files.is_empty() {
            let mut reader = from_std_in((self.stdin)());
            self.read_input(&mut reader, &mut index, process.as_mut())?;
        } else {
            for file in self.cli.files.clone() {
                self.read_file(&file, &mut index, process.as_mut())?;
            }
        }
        process.complete()?;
        Ok(())
    }

    fn read_file(&self, file: &PathBuf, index: &mut u64, process: &mut dyn Process) -> Result<()> {
        if !file.exists() {
            panic!("File {:?} not exists", file);
        }
        if file.is_dir() {
            for entry in read_dir(file)? {
                let path = entry?.path();
                self.read_file(&path, index, process)?;
            }
        } else {
            let mut reader = from_file(file)?;
            self.read_input(&mut reader, index, process)?;
        }
        Ok(())
    }
    fn read_input<R: Read>(
        &self,
        reader: &mut Reader<R>,
        index: &mut u64,
        process: &mut dyn Process,
    ) -> Result<()> {
        let mut in_file_index: u64 = 0;
        loop {
            let started = reader.where_am_i();
            match reader.next_json_value() {
                Ok(Some(val)) => {
                    if self.cli.only_objects_and_arrays {
                        match val {
                            JsonValue::Object(_) | JsonValue::Array(_) => {}
                            _ => {
                                continue;
                            }
                        }
                    }
                    let ended = reader.where_am_i();
                    let context = Context::new_with_input(
                        val,
                        started,
                        ended,
                        in_file_index,
                        *index,
                        &self.regular_expression_cache,
                    );
                    match process.process(context)? {
                        ProcessDesision::Break => break Ok(()),
                        ProcessDesision::Continue => {
                            in_file_index += 1;
                            *index += 1;
                        }
                    }
                }
                Ok(None) => {
                    return Ok(());
                }
                Err(e) => {
                    if !e.can_recover() {
                        return Err(e.into());
                    }
                    match self.cli.on_error {
                        OnError::Ignore => {}
                        OnError::Panic => {
                            return Err(e.into());
                        }
                        OnError::Stdout => writeln!(self.stdout.borrow_mut(), "error:{}", e)?,
                        OnError::Stderr => writeln!(self.stderr.borrow_mut(), "error:{}", e)?,
                    }
                }
            };
        }
    }
}

pub type Result<T> = std::result::Result<T, MainError>;

#[derive(Debug, Error)]
pub enum MainError {
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
