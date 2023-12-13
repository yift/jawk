use crate::functions_definitions::find_function;
use crate::functions_definitions::FunctionDefinitionsError;
use crate::json_parser::JsonParser;
use crate::json_parser::JsonParserError;
use crate::json_value::JsonValue;
use crate::processor::Context;
use crate::processor::Process;
use crate::processor::Titles;
use crate::reader::from_string;
use crate::reader::Location;
use crate::reader::Reader;
use std::io::Error as IoError;
use std::io::Read;
use std::num::ParseIntError;
use std::ops::Deref;
use std::str::FromStr;
use std::string::FromUtf8Error;
use std::sync::Arc;
use std::vec;
use thiserror::Error;

pub trait Get: Sync + Send {
    fn get(&self, value: &Option<JsonValue>) -> Option<JsonValue>;
}

#[derive(Clone)]
pub struct Selection {
    getter: Arc<Box<dyn Get>>,
    name: String,
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
        let getter = Arc::new(extractors);
        Ok(Selection { name, getter })
    }
}

struct SelectionProcess {
    name: String,
    getter: Arc<Box<dyn Get>>,
    next: Box<dyn Process>,
}
impl Process for SelectionProcess {
    fn start(&mut self, titles_so_far: Titles) -> crate::processor::Result {
        let next_titles = titles_so_far.with_title(self.name.clone());
        self.next.start(next_titles)
    }
    fn complete(&mut self) -> crate::processor::Result {
        self.next.complete()
    }
    fn process(&mut self, context: Context) -> crate::processor::Result {
        let input = context.input().as_ref().map(|val| val.deref().clone());

        let result = self.getter.get(&input);
        let new_context = context.with_result(result);

        self.next.process(new_context)
    }
}

#[derive(Clone)]
pub struct UnnamedSelection {
    getter: Arc<Box<dyn Get>>,
}
impl FromStr for UnnamedSelection {
    type Err = SelectionParseError;
    fn from_str(s: &str) -> Result<Self> {
        let source = s.to_string();
        let mut reader = from_string(&source);
        reader.eat_whitespace()?;
        let extractors = read_getter(&mut reader)?;
        reader.eat_whitespace()?;
        if let Some(ch) = reader.next()? {
            return Err(SelectionParseError::ExpectingEof(
                reader.where_am_i(),
                ch as char,
            ));
        }
        Ok(UnnamedSelection {
            getter: Arc::new(extractors),
        })
    }
}
impl UnnamedSelection {
    pub fn pass(&self, value: &JsonValue) -> bool {
        let val = Some(value.clone());
        self.getter.get(&val) == Some(JsonValue::Boolean(true))
    }
    pub fn name(&self, value: &JsonValue) -> Option<String> {
        let val = Some(value.clone());
        if let Some(JsonValue::String(str)) = self.getter.get(&val) {
            Some(str)
        } else {
            None
        }
    }
}
enum SingleExtract {
    ByKey(String),
    ByIndex(usize),
}
enum Extract {
    Root,
    Element(Vec<SingleExtract>),
}
struct ConstGetters {
    value: JsonValue,
}
impl Get for ConstGetters {
    fn get(&self, _: &Option<JsonValue>) -> Option<JsonValue> {
        let val = self.value.clone();
        Some(val)
    }
}
impl Get for Extract {
    fn get(&self, value: &Option<JsonValue>) -> Option<JsonValue> {
        match self {
            Extract::Root => value.clone(),
            Extract::Element(es) => {
                let mut val = value.clone();
                for e in es {
                    match val {
                        None => {
                            break;
                        }
                        Some(_) => {
                            val = e.get(&val);
                        }
                    }
                }
                val
            }
        }
    }
}
impl Get for SingleExtract {
    fn get(&self, value: &Option<JsonValue>) -> Option<JsonValue> {
        match value {
            Some(JsonValue::Array(list)) => match self {
                SingleExtract::ByIndex(index) => list.get(*index).cloned(),
                _ => None,
            },
            Some(JsonValue::Object(map)) => match self {
                SingleExtract::ByKey(key) => map.get(key).cloned(),
                _ => None,
            },
            _ => None,
        }
    }
}
pub fn read_getter<R: Read>(reader: &mut Reader<R>) -> Result<Box<dyn Get>> {
    reader.eat_whitespace()?;
    match reader.peek()? {
        None => Err(SelectionParseError::UnexpectedEof),
        Some(b'.') | Some(b'#') => {
            let ext = Extract::parse(reader)?;
            Ok(Box::new(ext))
        }
        Some(b'(') => parse_function(reader),
        _ => match reader.next_json_value()? {
            None => Err(SelectionParseError::UnexpectedEof),
            Some(val) => Ok(Box::new(ConstGetters { value: val })),
        },
    }
}
impl Extract {
    fn parse<R: Read>(reader: &mut Reader<R>) -> Result<Self> {
        let mut ext = vec![];
        loop {
            match reader.peek()? {
                Some(b'.') => {
                    let key = Self::read_extract_key(reader)?;
                    if key.is_empty() {
                        if ext.is_empty() {
                            return Ok(Extract::Root);
                        } else {
                            return Err(SelectionParseError::MissingKey(reader.where_am_i()));
                        }
                    }
                    let es = SingleExtract::ByKey(key);
                    ext.push(es);
                }
                Some(b'#') => match Self::read_extract_index(reader)? {
                    None => {
                        if ext.is_empty() {
                            return Ok(Extract::Root);
                        } else {
                            return Err(SelectionParseError::MissingKey(reader.where_am_i()));
                        }
                    }
                    Some(index) => {
                        let es = SingleExtract::ByIndex(index);
                        ext.push(es);
                    }
                },
                _ => {
                    return Ok(Extract::Element(ext));
                }
            }
        }
    }
    fn read_extract_key<R: Read>(reader: &mut Reader<R>) -> Result<String> {
        let mut buf = Vec::new();
        loop {
            match reader.next()? {
                None => {
                    break;
                }
                Some(ch) => {
                    if ch.is_ascii_whitespace()
                        || ch == b'.'
                        || ch == b','
                        || ch == b'='
                        || ch == b'('
                        || ch == b')'
                        || ch.is_ascii_control()
                        || ch == b'\"'
                        || ch == b']'
                        || ch == b'['
                        || ch == b'{'
                        || ch == b'}'
                        || ch == b'#'
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
    fn read_extract_index<R: Read>(reader: &mut Reader<R>) -> Result<Option<usize>> {
        reader.next()?;
        let mut digits = Vec::new();
        reader.read_digits(&mut digits)?;
        let str = String::from_utf8(digits)?;
        if str.is_empty() {
            return Ok(None);
        }
        let number = str.parse::<usize>()?;
        Ok(Some(number))
    }
}

fn parse_function<R: Read>(reader: &mut Reader<R>) -> Result<Box<dyn Get>> {
    let mut name = read_function_name(reader)?;
    let mut args: Vec<Box<dyn Get>> = Vec::new();
    if name.starts_with('.') {
        args.push(Box::new(Extract::Root));
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
    fn get(&self, value: &Option<JsonValue>) -> Option<JsonValue> {
        self.getter.get(value)
    }
}
