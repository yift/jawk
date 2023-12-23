use crate::{
    json_parser::JsonParserError, json_value::JsonValue, processor::Context, reader::Reader,
    selection::Get, selection::Result,
};
use std::{io::Read, rc::Rc};

enum Type {
    Variable,
    Macro,
}
struct VariableExtructor {
    name: String,
    variable_type: Type,
}

impl Get for VariableExtructor {
    fn get(&self, value: &Context) -> Option<JsonValue> {
        match self.variable_type {
            Type::Macro => value.get_definition(&self.name).and_then(|f| f.get(value)),
            Type::Variable => value.get_variable_value(&self.name).cloned(),
        }
    }
}

pub fn parse_get_variable<R: Read>(reader: &mut Reader<R>) -> Result<Rc<dyn Get>> {
    let variable_type = match reader.peek()? {
        None => {
            return Err(JsonParserError::UnexpectedEof(reader.where_am_i()).into());
        }
        Some(b':') => Type::Variable,
        Some(b'@') => Type::Macro,
        Some(ch) => {
            return Err(JsonParserError::UnexpectedCharacter(
                reader.where_am_i(),
                ch as char,
                ":@".into(),
            )
            .into());
        }
    };
    let mut name = Vec::new();
    loop {
        match reader.next()? {
            None | Some(b' ' | b'\n' | b'\t' | b'\r' | b')') => {
                break;
            }
            Some(ch) => name.push(ch),
        };
    }
    if name.is_empty() {
        return Err(JsonParserError::UnexpectedEof(reader.where_am_i()).into());
    }
    let name = String::from_utf8(name)?;
    Ok(Rc::new(VariableExtructor {
        name,
        variable_type,
    }))
}
