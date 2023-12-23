use std::{collections::HashMap, rc::Rc, str::FromStr};

use thiserror::Error;

use crate::{
    json_value::JsonValue,
    processor::{Context, Process, Titles},
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
    fn complete(&mut self) -> crate::processor::Result {
        self.next.complete()
    }
    fn process(&mut self, context: Context) -> crate::processor::Result {
        let new_context = context
            .with_variables(&self.variables)
            .with_definitions(&self.macros);
        self.next.process(new_context)
    }
    fn start(&mut self, titles_so_far: Titles) -> crate::processor::Result {
        self.next.start(titles_so_far)
    }
}
