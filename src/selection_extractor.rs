use crate::{
    json_parser::JsonParserError, json_value::JsonValue, processor::Context, reader::Reader,
    selection::Get, selection::Result,
};
use std::{io::Read, rc::Rc};

struct SelectionExtructor {
    name: String,
}

impl Get for SelectionExtructor {
    fn get(&self, value: &Context) -> Option<JsonValue> {
        value.get_selected(&self.name)
    }
}

pub fn parse_get_selection<R: Read>(reader: &mut Reader<R>) -> Result<Rc<dyn Get>> {
    if reader.peek()? != Some(b'\'') {
        return Err(JsonParserError::UnexpectedEof(reader.where_am_i()).into());
    }
    let mut name = Vec::new();
    loop {
        match reader.next()? {
            None => return Err(JsonParserError::UnexpectedEof(reader.where_am_i()).into()),
            Some(b'\'') => break,
            Some(ch) => name.push(ch),
        };
    }
    reader.next()?;
    if name.is_empty() {
        return Err(JsonParserError::UnexpectedEof(reader.where_am_i()).into());
    }
    let name = String::from_utf8(name)?;
    let name = name.trim().into();
    Ok(Rc::new(SelectionExtructor { name }))
}

#[cfg(test)]
mod tests {

    use super::*;
    use crate::{reader::from_string, selection::SelectionParseError};

    #[test]
    fn parse_return_error_for_nothing() -> Result<()> {
        let text = String::new();
        let mut reader = from_string(&text);
        let error = parse_get_selection(&mut reader).err().unwrap();

        assert_eq!(
            matches!(
                error,
                SelectionParseError::JsonError(JsonParserError::UnexpectedEof(_))
            ),
            true
        );

        Ok(())
    }

    #[test]
    fn parse_return_error_for_unexpected_char() -> Result<()> {
        let text = "hi".to_string();
        let mut reader = from_string(&text);
        let error = parse_get_selection(&mut reader).err().unwrap();

        assert_eq!(
            matches!(
                error,
                SelectionParseError::JsonError(JsonParserError::UnexpectedEof(_))
            ),
            true
        );

        Ok(())
    }

    #[test]
    fn parse_return_error_for_empty_name() -> Result<()> {
        let text = "''".to_string();
        let mut reader = from_string(&text);
        let error = parse_get_selection(&mut reader).err().unwrap();

        assert_eq!(
            matches!(
                error,
                SelectionParseError::JsonError(JsonParserError::UnexpectedEof(_))
            ),
            true
        );

        Ok(())
    }

    #[test]
    fn get_return_the_correct_selection() -> Result<()> {
        let text = "'test'".to_string();
        let mut reader = from_string(&text);
        let selection = parse_get_selection(&mut reader).unwrap();

        let result = Some(1.into());
        let input = Context::new_empty().with_result(&Rc::new("test".to_string()), result.clone());

        let value = selection.get(&input);

        assert_eq!(result, value);

        Ok(())
    }

    #[test]
    fn get_return_the_correct_selection_when_not_found() -> Result<()> {
        let text = "'test-1'".to_string();
        let mut reader = from_string(&text);
        let selection = parse_get_selection(&mut reader).unwrap();

        let result = Some(1.into());
        let input =
            Context::new_empty().with_result(&Rc::new("test-2".to_string()), result.clone());

        let value = selection.get(&input);

        assert_eq!(None, value);

        Ok(())
    }
}
