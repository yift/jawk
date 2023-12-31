use std::{collections::HashMap, rc::Rc, str::FromStr};

use thiserror::Error;

use crate::{
    json_value::JsonValue,
    processor::{Context, Process, ProcessDesision, Result as ProcessResult, Titles},
    reader::from_string,
    selection::{read_getter, Get, SelectionParseError},
};

#[derive(Clone)]
enum Value {
    Macro(Rc<dyn Get>),
    Calculated(JsonValue),
}
#[derive(Clone)]
pub struct PreSet {
    key: String,
    value: Value,
}
#[derive(Debug, Error)]
pub enum PreSetParserError {
    #[error("{0}")]
    ParserError(#[from] SelectionParseError),
    #[error("invalid KEY=value: no `=` found in `{0}`")]
    NoEqualsError(String),
    #[error("Empty name in `{0}`")]
    EmptyName(String),
    #[error("Empty value in `{0}`")]
    EmptyValue(String),
    #[error("Duplicate keys defined: `{0}`")]
    DuplicateKeys(String),
}

impl FromStr for PreSet {
    type Err = PreSetParserError;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let pos = s
            .find('=')
            .ok_or_else(|| PreSetParserError::NoEqualsError(s.to_string()))?;
        let key = s[..pos].to_string();
        let key = key.trim();
        let key = key.to_string();
        let value = s[pos + 1..].to_string();
        let mut reader = from_string(&value);
        let value = read_getter(&mut reader)?;
        if let Some(macro_name) = key.strip_prefix('@') {
            if macro_name.is_empty() {
                Err(PreSetParserError::EmptyName(s.to_owned()))
            } else {
                Ok(PreSet {
                    key: macro_name.to_string(),
                    value: Value::Macro(value),
                })
            }
        } else {
            let context = Context::new_empty();
            let value = value
                .get(&context)
                .ok_or_else(|| PreSetParserError::EmptyValue(s.to_string()))?;
            if key.is_empty() {
                Err(PreSetParserError::EmptyName(s.to_owned()))
            } else {
                Ok(PreSet {
                    key,
                    value: Value::Calculated(value),
                })
            }
        }
    }
}

pub trait PreSetCollection {
    fn create_process(&self, next: Box<dyn Process>)
        -> Result<Box<dyn Process>, PreSetParserError>;
}

struct PreSetProcessor {
    variables: Rc<HashMap<String, JsonValue>>,
    macros: Rc<HashMap<String, Rc<dyn Get>>>,
    next: Box<dyn Process>,
}

impl PreSetCollection for Vec<String> {
    fn create_process(
        &self,
        next: Box<dyn Process>,
    ) -> Result<Box<dyn Process>, PreSetParserError> {
        if self.is_empty() {
            return Ok(next);
        }
        let mut variables = HashMap::new();
        let mut macros = HashMap::new();
        for p in self {
            let p = PreSet::from_str(p)?;
            match &p.value {
                Value::Calculated(value) => {
                    if variables.insert(p.key.clone(), value.clone()).is_some() {
                        return Err(PreSetParserError::DuplicateKeys(p.key.clone()));
                    }
                }
                Value::Macro(getter) => {
                    if macros.insert(p.key.clone(), getter.clone()).is_some() {
                        return Err(PreSetParserError::DuplicateKeys(p.key.clone()));
                    }
                }
            }
        }
        Ok(Box::new(PreSetProcessor {
            variables: Rc::new(variables),
            macros: Rc::new(macros),
            next,
        }))
    }
}

impl Process for PreSetProcessor {
    fn complete(&mut self) -> ProcessResult<()> {
        self.next.complete()
    }
    fn process(&mut self, context: Context) -> ProcessResult<ProcessDesision> {
        let new_context = context
            .with_variables(&self.variables)
            .with_definitions(&self.macros);
        self.next.process(new_context)
    }
    fn start(&mut self, titles_so_far: Titles) -> ProcessResult<()> {
        self.next.start(titles_so_far)
    }
}

#[cfg(test)]
mod tests {
    use std::cell::RefCell;
    use std::ops::Deref;
    use std::rc::Rc;

    use super::*;
    use crate::json_value::JsonValue;
    use crate::processor::{Context, Titles};

    #[test]
    fn parse_parse_correctly() -> ProcessResult<()> {
        let list = vec!["ten=10".to_string(), "@eleven=11".to_string()];
        struct Next(Rc<RefCell<bool>>);
        let data = Rc::new(RefCell::new(false));
        impl Process for Next {
            fn complete(&mut self) -> ProcessResult<()> {
                Ok(())
            }
            fn start(&mut self, _: Titles) -> ProcessResult<()> {
                Ok(())
            }
            fn process(&mut self, context: Context) -> ProcessResult<ProcessDesision> {
                assert_eq!(
                    context.get_variable_value(&"ten".to_string()).cloned(),
                    JsonValue::from_str("10").ok()
                );
                let mac = context.get_definition(&"eleven".to_string()).unwrap();
                assert_eq!(mac.get(&context), JsonValue::from_str("11").ok());
                *self.0.borrow_mut() = true;
                Ok(ProcessDesision::Continue)
            }
        }

        let next = Box::new(Next(data.clone()));
        let mut preseters = list.create_process(next).unwrap();
        let context = Context::new_with_no_context(JsonValue::Null);

        preseters.process(context)?;

        let binding = data.borrow();
        let data = binding.deref();
        assert_eq!(data, &true);

        Ok(())
    }

    #[test]
    fn no_equal_return_error() -> ProcessResult<()> {
        let list = vec!["name".to_string()];
        struct Next;
        impl Process for Next {
            fn complete(&mut self) -> ProcessResult<()> {
                Ok(())
            }
            fn start(&mut self, _: Titles) -> ProcessResult<()> {
                Ok(())
            }
            fn process(&mut self, _: Context) -> ProcessResult<ProcessDesision> {
                Ok(ProcessDesision::Continue)
            }
        }

        let next = Box::new(Next);
        let error = list.create_process(next).err().unwrap();

        assert_eq!(matches!(error, PreSetParserError::NoEqualsError(_)), true);

        Ok(())
    }

    #[test]
    fn no_name_variable_return_error() -> ProcessResult<()> {
        let list = vec!["=1".to_string()];
        struct Next;
        impl Process for Next {
            fn complete(&mut self) -> ProcessResult<()> {
                Ok(())
            }
            fn start(&mut self, _: Titles) -> ProcessResult<()> {
                Ok(())
            }
            fn process(&mut self, _: Context) -> ProcessResult<ProcessDesision> {
                Ok(ProcessDesision::Continue)
            }
        }

        let next = Box::new(Next);
        let error = list.create_process(next).err().unwrap();

        assert_eq!(matches!(error, PreSetParserError::EmptyName(_)), true);

        Ok(())
    }

    #[test]
    fn no_name_def_return_error() -> ProcessResult<()> {
        let list = vec!["@=1".to_string()];
        struct Next;
        impl Process for Next {
            fn complete(&mut self) -> ProcessResult<()> {
                Ok(())
            }
            fn start(&mut self, _: Titles) -> ProcessResult<()> {
                Ok(())
            }
            fn process(&mut self, _: Context) -> ProcessResult<ProcessDesision> {
                Ok(ProcessDesision::Continue)
            }
        }

        let next = Box::new(Next);
        let error = list.create_process(next).err().unwrap();

        assert_eq!(matches!(error, PreSetParserError::EmptyName(_)), true);

        Ok(())
    }
    #[test]
    fn duplicate_name_return_error() -> ProcessResult<()> {
        let list = vec!["name=1".to_string(), "name=2".to_string()];
        struct Next;
        impl Process for Next {
            fn complete(&mut self) -> ProcessResult<()> {
                Ok(())
            }
            fn start(&mut self, _: Titles) -> ProcessResult<()> {
                Ok(())
            }
            fn process(&mut self, _: Context) -> ProcessResult<ProcessDesision> {
                Ok(ProcessDesision::Continue)
            }
        }

        let next = Box::new(Next);
        let error = list.create_process(next).err().unwrap();

        assert_eq!(matches!(error, PreSetParserError::DuplicateKeys(_)), true);

        Ok(())
    }
    #[test]
    fn duplicate_def_name_return_error() -> ProcessResult<()> {
        let list = vec!["@name=1".to_string(), "@name=2".to_string()];
        struct Next;
        impl Process for Next {
            fn complete(&mut self) -> ProcessResult<()> {
                Ok(())
            }
            fn start(&mut self, _: Titles) -> ProcessResult<()> {
                Ok(())
            }
            fn process(&mut self, _: Context) -> ProcessResult<ProcessDesision> {
                Ok(ProcessDesision::Continue)
            }
        }

        let next = Box::new(Next);
        let error = list.create_process(next).err().unwrap();

        assert_eq!(matches!(error, PreSetParserError::DuplicateKeys(_)), true);

        Ok(())
    }
}
