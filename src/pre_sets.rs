use std::{collections::HashMap, rc::Rc, str::FromStr};

use thiserror::Error;

use crate::{
    json_parser::{JsonParser, JsonParserError},
    json_value::JsonValue,
    processor::{Context, Process, Titles},
    reader::from_string,
};

#[derive(Clone)]
pub struct PreSet {
    key: String,
    value: JsonValue,
}
#[derive(Debug, Error)]
pub enum PreSetParserError {
    #[error("{0}")]
    JsonParserError(#[from] JsonParserError),
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
        if key.is_empty() {
            return Err(PreSetParserError::EmptyName(s.to_owned()));
        }
        let key = key.to_string();
        let value = s[pos + 1..].to_string();
        let mut reader = from_string(&value);
        let value = reader.next_json_value()?;
        let value = value.ok_or_else(|| PreSetParserError::EmptyValue(s.to_string()))?;

        Ok(PreSet { key, value })
    }
}

pub trait PreSetCollection {
    fn create_process(&self, next: Box<dyn Process>)
        -> Result<Box<dyn Process>, PreSetParserError>;
}

struct PreSetProcessor {
    variables: Rc<HashMap<String, JsonValue>>,
    next: Box<dyn Process>,
}

impl PreSetCollection for Vec<PreSet> {
    fn create_process(
        &self,
        next: Box<dyn Process>,
    ) -> Result<Box<dyn Process>, PreSetParserError> {
        if self.is_empty() {
            return Ok(next);
        }
        let mut mp = HashMap::new();
        for p in self {
            if mp.insert(p.key.clone(), p.value.clone()).is_some() {
                return Err(PreSetParserError::DuplicateKeys(p.key.clone()));
            }
        }
        Ok(Box::new(PreSetProcessor {
            variables: Rc::new(mp),
            next,
        }))
    }
}

impl Process for PreSetProcessor {
    fn complete(&mut self) -> crate::processor::Result {
        self.next.complete()
    }
    fn process(&mut self, context: Context) -> crate::processor::Result {
        let new_context = context.with_variables(&self.variables);
        self.next.process(new_context)
    }
    fn start(&mut self, titles_so_far: Titles) -> crate::processor::Result {
        self.next.start(titles_so_far)
    }
}
