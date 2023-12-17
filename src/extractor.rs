use std::{io::Read, ops::Deref};

use crate::{
    json_value::JsonValue,
    processor::Context,
    reader::Reader,
    selection::{Get, Result, SelectionParseError},
};

enum SingleExtract {
    ByKey(String),
    ByIndex(usize),
}
enum Extract {
    Root,
    Element(Vec<SingleExtract>),
}
impl Get for Extract {
    fn get(&self, context: &Context) -> Option<JsonValue> {
        let value = context.input();
        match self {
            Extract::Root => Some(value.deref().clone()),
            Extract::Element(es) => {
                let mut val = Some(value.deref().clone());
                for e in es {
                    match val {
                        None => {
                            break;
                        }
                        Some(value) => {
                            val = e.get(&Context::new(value));
                        }
                    }
                }
                val
            }
        }
    }
}
impl Get for SingleExtract {
    fn get(&self, value: &Context) -> Option<JsonValue> {
        match value.input().deref() {
            JsonValue::Array(list) => match self {
                SingleExtract::ByIndex(index) => list.get(*index).cloned(),
                _ => None,
            },
            JsonValue::Object(map) => match self {
                SingleExtract::ByKey(key) => map.get(key).cloned(),
                _ => None,
            },
            _ => None,
        }
    }
}
pub fn parse_extractor<R: Read>(reader: &mut Reader<R>) -> Result<Box<dyn Get>> {
    let extractor = Extract::parse(reader)?;
    Ok(Box::new(extractor))
}
pub fn root() -> Box<dyn Get> {
    Box::new(Extract::Root)
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
