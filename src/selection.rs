use crate::functions_definitions::find_function;
use crate::functions_definitions::FunctionDefinitionsError;
use crate::json_parser::JsonParser;
use crate::json_parser::JsonParserError;
use crate::json_value::JsonValue;
use crate::reader::from_string;
use crate::reader::Location;
use crate::reader::Reader;
use std::io::Error as IoError;
use std::io::Read;
use std::num::ParseIntError;
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

enum SingleExtract {
    ByKey(String),
    ByIndex(usize),
}
enum Extract {
    Root,
    Element(Vec<SingleExtract>),
}
enum BasicGetters {
    Const(JsonValue),
}
impl Get for BasicGetters {
    fn get(&self, _: &Option<JsonValue>) -> Option<JsonValue> {
        match self {
            BasicGetters::Const(val) => {
                let val = val.clone();
                Some(val)
            }
        }
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
                        None => break,
                        Some(_) => val = e.get(&val),
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
fn read_getter<R: Read>(reader: &mut Reader<R>) -> Result<Box<dyn Get>> {
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
            Some(val) => Ok(Box::new(BasicGetters::Const(val))),
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
                None => break,
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
                        buf.push(ch)
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
            Some(b')') => break,
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
            None => break,
            Some(ch) => {
                if ch.is_ascii_whitespace()
                    || ch == b','
                    || ch == b'('
                    || ch == b')'
                    || ch.is_ascii_control()
                {
                    break;
                } else {
                    buf.push(ch)
                }
            }
        }
    }
    let str = String::from_utf8(buf)?;
    Ok(str)
}

impl Selection {
    pub fn name(&self) -> &String {
        &self.name
    }
}
impl Get for Selection {
    fn get(&self, value: &Option<JsonValue>) -> Option<JsonValue> {
        self.getter.get(value)
    }
}
