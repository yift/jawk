use crate::const_getter::ConstGetters;
use crate::extractor::parse_extractor;
use crate::extractor::root;
use crate::functions_definitions::find_function;
use crate::functions_definitions::FunctionDefinitionsError;
use crate::input_context_extractor::parse_input_context;
use crate::input_context_extractor::InputContextExtractorParseError;
use crate::json_parser::JsonParserError;
use crate::json_value::JsonValue;
use crate::processor::Context;
use crate::processor::Process;
use crate::processor::ProcessDesision;
use crate::processor::Result as ProcessResult;
use crate::processor::Titles;
use crate::reader::from_string;
use crate::reader::Location;
use crate::reader::Reader;
use crate::variables_extractor::parse_get_variable;
use std::io::Error as IoError;
use std::io::Read;
use std::num::ParseIntError;
use std::rc::Rc;
use std::str::FromStr;
use std::string::FromUtf8Error;
use std::vec;
use thiserror::Error;

pub trait Get {
    fn get(&self, value: &Context) -> Option<JsonValue>;
}

#[derive(Clone)]
pub struct Selection {
    getter: Rc<dyn Get>,
    name: Rc<String>,
}

pub type Result<T> = std::result::Result<T, SelectionParseError>;
#[derive(Debug, Error)]
pub enum SelectionParseError {
    #[error("{0}")]
    IoError(#[from] IoError),
    #[error("{0}")]
    JsonError(#[from] JsonParserError),
    #[error("{0}")]
    StringUtfError(#[from] FromUtf8Error),
    #[error("{0}")]
    NumberParseError(#[from] ParseIntError),
    #[error("{0}")]
    Function(#[from] FunctionDefinitionsError),
    #[error("{0}")]
    InputContext(#[from] InputContextExtractorParseError),
    #[error("{0}, Missing key name")]
    MissingKey(Location),
    #[error("{0}: Expecting equals, got {1}")]
    ExpectingEquals(Location, char),
    #[error("{0}: Expecting EOF, got {1}")]
    ExpectingEof(Location, char),
    #[error("Unexpected end of string")]
    UnexpectedEof,
}

impl FromStr for Selection {
    type Err = SelectionParseError;
    fn from_str(s: &str) -> Result<Self> {
        let source = s.to_string();
        let mut reader = from_string(&source);
        reader.eat_whitespace()?;
        let extractors = read_getter(&mut reader)?;
        reader.eat_whitespace()?;
        let name = match reader.peek()? {
            Some(b'=') => {
                reader.next()?;
                reader.eat_whitespace()?;
                let mut buf = vec![];
                while let Some(ch) = reader.peek()? {
                    buf.push(ch);
                    reader.next()?;
                }
                String::from_utf8(buf)?
            }
            Some(ch) => {
                return Err(SelectionParseError::ExpectingEquals(
                    reader.where_am_i(),
                    ch as char,
                ));
            }
            None => source.clone(),
        };
        let getter = extractors;
        let name = Rc::new(name);
        Ok(Selection { name, getter })
    }
}

struct SelectionProcess {
    name: Rc<String>,
    getter: Rc<dyn Get>,
    next: Box<dyn Process>,
}
impl Process for SelectionProcess {
    fn start(&mut self, titles_so_far: Titles) -> ProcessResult<()> {
        let next_titles = titles_so_far.with_title(&self.name);
        self.next.start(next_titles)
    }
    fn complete(&mut self) -> ProcessResult<()> {
        self.next.complete()
    }
    fn process(&mut self, context: Context) -> ProcessResult<ProcessDesision> {
        let result = self.getter.get(&context);
        let new_context = context.with_result(&self.name, result);

        self.next.process(new_context)?;
        Ok(ProcessDesision::Continue)
    }
}

pub fn read_getter<R: Read>(reader: &mut Reader<R>) -> Result<Rc<dyn Get>> {
    reader.eat_whitespace()?;
    match reader.peek()? {
        None => Err(SelectionParseError::UnexpectedEof),
        Some(b'.') | Some(b'#') | Some(b'^') => parse_extractor(reader),
        Some(b'(') => parse_function(reader),
        Some(b':' | b'@') => parse_get_variable(reader),
        Some(b'&') => parse_input_context(reader),
        _ => match ConstGetters::parse(reader)? {
            None => Err(SelectionParseError::UnexpectedEof),
            Some(getter) => Ok(Rc::new(getter)),
        },
    }
}

fn parse_function<R: Read>(reader: &mut Reader<R>) -> Result<Rc<dyn Get>> {
    let mut name = read_function_name(reader)?;
    let mut args: Vec<Rc<dyn Get>> = Vec::new();
    if name.starts_with('.') {
        args.push(root());
        name = name[1..].to_string();
    }
    let function = find_function(&name)?;
    loop {
        reader.eat_whitespace()?;
        match reader.peek()? {
            Some(b',') => {
                reader.next()?;
            }
            Some(b')') => {
                break;
            }
            None => {
                return Err(SelectionParseError::UnexpectedEof);
            }
            _ => {
                let arg = read_getter(reader)?;
                args.push(arg);
            }
        }
    }
    reader.next()?;
    let fun = function.create(args)?;
    Ok(fun)
}

fn read_function_name<R: Read>(reader: &mut Reader<R>) -> Result<String> {
    reader.eat_whitespace()?;
    let mut buf = Vec::new();
    loop {
        match reader.next()? {
            None => {
                break;
            }
            Some(ch) => {
                if ch.is_ascii_whitespace()
                    || ch == b','
                    || ch == b'('
                    || ch == b')'
                    || ch.is_ascii_control()
                {
                    break;
                } else {
                    buf.push(ch);
                }
            }
        }
    }
    let str = String::from_utf8(buf)?;
    Ok(str)
}

impl Selection {
    pub fn create_process(&self, next: Box<dyn Process>) -> Box<dyn Process> {
        let process = SelectionProcess {
            name: self.name.clone(),
            getter: self.getter.clone(),
            next,
        };
        Box::new(process)
    }
}
impl Get for Selection {
    fn get(&self, context: &Context) -> Option<JsonValue> {
        self.getter.get(context)
    }
}
