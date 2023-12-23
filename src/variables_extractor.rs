use crate::{
    json_parser::JsonParserError, json_value::JsonValue, processor::Context, reader::Reader,
    selection::Get, selection::Result,
};
use std::{io::Read, rc::Rc};

struct VariableExtructor {
    name: String,
}

impl Get for VariableExtructor {
    fn get(&self, value: &Context) -> Option<JsonValue> {
        value.get_variable_value(&self.name).cloned()
    }
}

pub fn parse_get_variable<R: Read>(reader: &mut Reader<R>) -> Result<Rc<dyn Get>> {
    match reader.peek()? {
        None => {
            return Err(JsonParserError::UnexpectedEof(reader.where_am_i()).into());
        }
        Some(b':') => {}
        Some(ch) => {
            return Err(JsonParserError::UnexpectedCharacter(
                reader.where_am_i(),
                ch as char,
                ":".into(),
            )
            .into());
        }
    }
    let mut name = Vec::new();
    loop {
        match reader.next()? {
            None | Some(b' ' | b'\n' | b'\t' | b'\r') => {
                break;
            }
            Some(ch) => name.push(ch),
        };
    }
    if name.is_empty() {
        return Err(JsonParserError::UnexpectedEof(reader.where_am_i()).into());
    }
    let name = String::from_utf8(name)?;
    Ok(Rc::new(VariableExtructor { name }))
}
