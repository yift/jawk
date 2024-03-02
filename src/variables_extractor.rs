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

#[cfg(test)]
mod tests {
    use std::{cell::RefCell, str::FromStr};

    use super::*;
    use crate::{json_value::JsonValue, reader::from_string, selection::SelectionParseError};

    #[test]
    fn parse_return_error_for_nothing() {
        let text = String::new();
        let mut reader = from_string(&text);
        let error = parse_get_variable(&mut reader).err().unwrap();

        assert!(matches!(
            error,
            SelectionParseError::JsonError(JsonParserError::UnexpectedEof(_))
        ));
    }

    #[test]
    fn parse_return_error_for_unexpected_char() {
        let text = "hi".to_string();
        let mut reader = from_string(&text);
        let error = parse_get_variable(&mut reader).err().unwrap();

        assert!(matches!(
            error,
            SelectionParseError::JsonError(JsonParserError::UnexpectedCharacter(_, _, _))
        ));
    }

    #[test]
    fn parse_return_error_for_empty_name() {
        let text = ":".to_string();
        let mut reader = from_string(&text);
        let error = parse_get_variable(&mut reader).err().unwrap();

        assert!(matches!(
            error,
            SelectionParseError::JsonError(JsonParserError::UnexpectedEof(_))
        ));
    }

    #[test]
    fn macro_will_apply_getter() -> Result<()> {
        let text = "@name".to_string();
        let mut reader = from_string(&text);
        let getter = parse_get_variable(&mut reader)?;

        struct Caller(Rc<RefCell<usize>>);
        let data = Rc::new(RefCell::new(0));
        impl Get for Caller {
            fn get(&self, _: &Context) -> Option<JsonValue> {
                *self.0.borrow_mut() += 1;
                JsonValue::from_str("12").ok()
            }
        }
        let caller: Rc<dyn Get> = Rc::new(Caller(data.clone()));
        let context = Context::new_empty().with_definition("name".to_string(), &caller);

        let value = getter.get(&context);

        {
            assert_eq!(value, JsonValue::from_str("12").ok());
            let binding = data.borrow();
            let data = &*binding;
            assert_eq!(data, &1);
        }

        {
            let value = getter.get(&context);
            assert_eq!(value, JsonValue::from_str("12").ok());
            let binding = data.borrow();
            let data = &*binding;
            assert_eq!(data, &2);
        }

        Ok(())
    }

    #[test]
    fn macro_will_get_variable() -> Result<()> {
        let text = ":name".to_string();
        let mut reader = from_string(&text);
        let getter = parse_get_variable(&mut reader)?;

        let context = Context::new_empty().with_variable("name".to_string(), (22).into());

        let value = getter.get(&context);

        assert_eq!(value, JsonValue::from_str("22").ok());

        Ok(())
    }
}
