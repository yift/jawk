use thiserror::Error;

use crate::{
    json_value::{JsonValue, NumberValue},
    processor::Context,
    reader::Reader,
    selection::{Get, Result as SelectionResult},
};
use std::{io::Read, rc::Rc};

#[derive(Debug, Error)]
pub enum InputContextExtractorParseError {
    #[error("Input context type: '{0}' is unknonw")]
    UnknonwType(String),
}

#[derive(PartialEq, Debug)]
enum Type {
    Index,
    IndexInFile,
    FileName,
    StartedAtLineNumber,
    EndsAtLineNumber,
    StartedAtCharNumber,
    EndAtCharNumber,
}
struct InputContextExtractor {
    extration: Type,
}

impl InputContextExtractor {
    fn from_name(name: String) -> Result<Self, InputContextExtractorParseError> {
        let extration = match name.as_str() {
            "index" => Type::Index,
            "index-in-file" => Type::IndexInFile,
            "started-at-line-number" => Type::StartedAtLineNumber,
            "started-at-char-number" => Type::StartedAtCharNumber,
            "ended-at-line-number" => Type::EndsAtLineNumber,
            "ended-at-char-number" => Type::EndAtCharNumber,
            "file-name" => Type::FileName,
            _ => {
                return Err(InputContextExtractorParseError::UnknonwType(name));
            }
        };
        Ok(Self { extration })
    }
}
impl Get for InputContextExtractor {
    fn get(&self, value: &Context) -> Option<JsonValue> {
        if let Some(context) = value.input_context() {
            match self.extration {
                Type::Index => Some(JsonValue::Number(NumberValue::Positive(context.index))),
                Type::IndexInFile => {
                    Some(JsonValue::Number(NumberValue::Positive(context.file_index)))
                }
                Type::StartedAtLineNumber => Some(context.start_location.line_number.into()),
                Type::EndsAtLineNumber => Some(context.end_location.line_number.into()),
                Type::StartedAtCharNumber => Some(context.start_location.char_number.into()),
                Type::EndAtCharNumber => Some(context.end_location.char_number.into()),
                Type::FileName => context
                    .start_location
                    .input
                    .as_ref()
                    .map(|str| str.clone().into()),
            }
        } else {
            None
        }
    }
}

pub fn parse_input_context<R: Read>(reader: &mut Reader<R>) -> SelectionResult<Rc<dyn Get>> {
    let mut name = Vec::new();
    while let Some(ch) = reader.next()? {
        if ch.is_ascii_lowercase() {
            name.push(ch)
        } else if ch.is_ascii_uppercase() {
            name.push(ch.to_ascii_lowercase())
        } else if ch == b'_' || ch == b'-' {
            name.push(b'-')
        } else {
            break;
        };
    }
    let name = String::from_utf8(name)?;
    let getter = InputContextExtractor::from_name(name)?;
    Ok(Rc::new(getter))
}

#[cfg(test)]
mod tests {
    use crate::reader::{from_string, Location};

    use super::*;

    #[test]
    fn from_name_return_the_correct_name_index() -> SelectionResult<()> {
        from_name_return_the_correct_name("index", Type::Index)
    }

    #[test]
    fn from_name_return_the_correct_name_index_in_file() -> SelectionResult<()> {
        from_name_return_the_correct_name("index-in-file", Type::IndexInFile)
    }

    #[test]
    fn from_name_return_the_correct_name_started_line() -> SelectionResult<()> {
        from_name_return_the_correct_name("started-at-line-number", Type::StartedAtLineNumber)
    }

    #[test]
    fn from_name_return_the_correct_name_ended_line() -> SelectionResult<()> {
        from_name_return_the_correct_name("ended-at-line-number", Type::EndsAtLineNumber)
    }

    #[test]
    fn from_name_return_the_correct_name_started_char() -> SelectionResult<()> {
        from_name_return_the_correct_name("started-at-char-number", Type::StartedAtCharNumber)
    }

    #[test]
    fn from_name_return_the_correct_name_ended_char() -> SelectionResult<()> {
        from_name_return_the_correct_name("ended-at-char-number", Type::EndAtCharNumber)
    }

    #[test]
    fn from_name_return_the_correct_name_file_name() -> SelectionResult<()> {
        from_name_return_the_correct_name("file-name", Type::FileName)
    }

    fn from_name_return_the_correct_name(name: &str, expected: Type) -> SelectionResult<()> {
        let got = InputContextExtractor::from_name(name.to_string())?.extration;

        assert_eq!(got, expected);

        Ok(())
    }

    #[test]
    fn from_name_return_error_for_unknown_name() -> SelectionResult<()> {
        let err = InputContextExtractor::from_name("nop".to_string()).err();

        assert_eq!(err.is_some(), true);

        Ok(())
    }

    #[test]
    fn get_return_the_correct_values_index() -> SelectionResult<()> {
        let ext = InputContextExtractor::from_name("index".to_string())?;
        let conext = Context::new_with_input(
            JsonValue::Null,
            Location {
                input: None,
                line_number: 40,
                char_number: 10,
            },
            Location {
                input: None,
                line_number: 40,
                char_number: 10,
            },
            10,
            61,
        );

        assert_eq!(ext.get(&conext), Some(61.into()));

        Ok(())
    }

    #[test]
    fn get_return_the_correct_values_index_in_file() -> SelectionResult<()> {
        let ext = InputContextExtractor::from_name("index-in-file".to_string())?;
        let conext = Context::new_with_input(
            JsonValue::Null,
            Location {
                input: None,
                line_number: 40,
                char_number: 10,
            },
            Location {
                input: None,
                line_number: 40,
                char_number: 10,
            },
            10,
            61,
        );

        assert_eq!(ext.get(&conext), Some(10.into()));

        Ok(())
    }

    #[test]
    fn get_return_the_correct_values_line_started() -> SelectionResult<()> {
        let ext = InputContextExtractor::from_name("started-at-line-number".to_string())?;
        let conext = Context::new_with_input(
            JsonValue::Null,
            Location {
                input: None,
                line_number: 41,
                char_number: 10,
            },
            Location {
                input: None,
                line_number: 42,
                char_number: 10,
            },
            10,
            61,
        );

        assert_eq!(ext.get(&conext), Some(41.into()));

        Ok(())
    }

    #[test]
    fn get_return_the_correct_values_line_ended() -> SelectionResult<()> {
        let ext = InputContextExtractor::from_name("ended-at-line-number".to_string())?;
        let conext = Context::new_with_input(
            JsonValue::Null,
            Location {
                input: None,
                line_number: 41,
                char_number: 10,
            },
            Location {
                input: None,
                line_number: 42,
                char_number: 10,
            },
            10,
            61,
        );

        assert_eq!(ext.get(&conext), Some(42.into()));

        Ok(())
    }

    #[test]
    fn get_return_the_correct_values_char_started() -> SelectionResult<()> {
        let ext = InputContextExtractor::from_name("started-at-char-number".to_string())?;
        let conext = Context::new_with_input(
            JsonValue::Null,
            Location {
                input: None,
                line_number: 41,
                char_number: 11,
            },
            Location {
                input: None,
                line_number: 42,
                char_number: 20,
            },
            10,
            61,
        );

        assert_eq!(ext.get(&conext), Some(11.into()));

        Ok(())
    }

    #[test]
    fn get_return_the_correct_values_char_ended() -> SelectionResult<()> {
        let ext = InputContextExtractor::from_name("ended-at-char-number".to_string())?;
        let conext = Context::new_with_input(
            JsonValue::Null,
            Location {
                input: None,
                line_number: 41,
                char_number: 11,
            },
            Location {
                input: None,
                line_number: 42,
                char_number: 20,
            },
            10,
            61,
        );

        assert_eq!(ext.get(&conext), Some(20.into()));

        Ok(())
    }

    #[test]
    fn get_return_the_correct_values_file_name() -> SelectionResult<()> {
        let ext = InputContextExtractor::from_name("file-name".to_string())?;
        let conext = Context::new_with_input(
            JsonValue::Null,
            Location {
                input: Some("test".into()),
                line_number: 41,
                char_number: 11,
            },
            Location {
                input: None,
                line_number: 42,
                char_number: 20,
            },
            10,
            61,
        );

        assert_eq!(ext.get(&conext), Some("test".into()));

        Ok(())
    }

    #[test]
    fn get_return_the_correct_values_file_name_none() -> SelectionResult<()> {
        let ext = InputContextExtractor::from_name("file-name".to_string())?;
        let conext = Context::new_with_input(
            JsonValue::Null,
            Location {
                input: None,
                line_number: 41,
                char_number: 11,
            },
            Location {
                input: None,
                line_number: 42,
                char_number: 20,
            },
            10,
            61,
        );

        assert_eq!(ext.get(&conext), None);

        Ok(())
    }

    #[test]
    fn get_return_notging_without_context() -> SelectionResult<()> {
        let ext = InputContextExtractor::from_name("index".to_string())?;
        let conext = Context::new_with_no_context(JsonValue::Null);

        assert_eq!(ext.get(&conext), None);

        Ok(())
    }

    #[test]
    fn parse_will_be_case_insensetive() -> SelectionResult<()> {
        let text = "EndEd-at_char-number 3".into();
        let mut reader = from_string(&text);

        let selection = parse_input_context(&mut reader)?;
        let conext = Context::new_with_input(
            JsonValue::Null,
            Location {
                input: None,
                line_number: 41,
                char_number: 11,
            },
            Location {
                input: None,
                line_number: 42,
                char_number: 20,
            },
            10,
            61,
        );

        assert_eq!(selection.get(&conext), Some(20.into()));

        Ok(())
    }

    #[test]
    fn parse_will_create_exception_for_unknwon_name() -> SelectionResult<()> {
        let text = "test".into();
        let mut reader = from_string(&text);

        let err = parse_input_context(&mut reader).err();

        assert_eq!(err.is_some(), true);

        Ok(())
    }
}
