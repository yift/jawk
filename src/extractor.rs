use std::{io::Read, rc::Rc};

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
enum ExtractFromInput {
    Root,
    Element(Vec<SingleExtract>),
}
struct Extract {
    number_of_parents: usize,
    extract_from_input: ExtractFromInput,
}
impl ExtractFromInput {
    fn extract(&self, input: &JsonValue) -> Option<JsonValue> {
        match self {
            ExtractFromInput::Root => Some(input.clone()),
            ExtractFromInput::Element(es) => {
                let mut val = Some(input.clone());
                for e in es {
                    match val {
                        None => {
                            break;
                        }
                        Some(value) => {
                            val = e.extract(&value);
                        }
                    }
                }
                val
            }
        }
    }
}
impl Get for Extract {
    fn get(&self, value: &Context) -> Option<JsonValue> {
        let input = value.parent_input(self.number_of_parents);
        self.extract_from_input.extract(input)
    }
}
pub fn parse_extractor<R: Read>(reader: &mut Reader<R>) -> Result<Rc<dyn Get>> {
    let number_of_parents = read_number_of_parents(reader)?;
    let extract_from_input = ExtractFromInput::parse(reader)?;
    Ok(Rc::new(Extract {
        number_of_parents,
        extract_from_input,
    }))
}
fn read_number_of_parents<R: Read>(reader: &mut Reader<R>) -> Result<usize> {
    let mut size = 0;
    loop {
        match reader.peek()? {
            Some(b'^') => {
                size += 1;
                reader.next()?;
            }
            _ => {
                return Ok(size);
            }
        }
    }
}
pub fn root() -> Rc<dyn Get> {
    Rc::new(Extract {
        extract_from_input: ExtractFromInput::Root,
        number_of_parents: 0,
    })
}
impl ExtractFromInput {
    fn parse<R: Read>(reader: &mut Reader<R>) -> Result<Self> {
        let mut ext = vec![];
        loop {
            match reader.peek()? {
                Some(b'.') => {
                    let key = Self::read_extract_key(reader)?;
                    if key.is_empty() {
                        if ext.is_empty() {
                            return Ok(ExtractFromInput::Root);
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
                            return Ok(ExtractFromInput::Root);
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
                    return Ok(ExtractFromInput::Element(ext));
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
impl SingleExtract {
    fn extract(&self, value: &JsonValue) -> Option<JsonValue> {
        match value {
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
