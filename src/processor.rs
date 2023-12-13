use std::ops::Deref;
use std::rc::Rc;

use indexmap::IndexMap;
use std::fmt::Error as FormatError;
use std::io::Error as IoEror;
use thiserror::Error;

use crate::json_value::JsonValue;

#[derive(Debug, Error)]
pub enum ProcessError {
    #[error("{0}")]
    Format(#[from] FormatError),
    #[error("{0}")]
    Io(#[from] IoEror),
    #[error("{0}")]
    InvalidInputError(&'static str),
}

#[derive(Default)]
pub struct Titles {
    titles: Vec<String>,
}
impl Titles {
    pub fn with_title(&self, title: String) -> Self {
        let mut titles = self.titles.clone();
        titles.push(title);
        Titles { titles }
    }

    pub fn len(&self) -> usize {
        self.titles.len()
    }

    pub fn get(&self, index: usize) -> Option<&String> {
        self.titles.get(index)
    }
    pub fn as_context(&self) -> Context {
        let mut headers = Context::new(JsonValue::null());
        for str in &self.titles {
            let value = Some(str.into());
            headers = headers.with_result(value);
        }
        headers
    }
}
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum ContextKey {
    Value(JsonValue),
    Results(Vec<Option<JsonValue>>),
}
pub struct Context {
    input: Rc<JsonValue>,
    results: Vec<Option<JsonValue>>,
}
impl Context {
    pub fn new(input: JsonValue) -> Self {
        Context {
            input: Rc::new(input),
            results: Vec::new(),
        }
    }
    pub fn with_result(&self, result: Option<JsonValue>) -> Self {
        let mut results = self.results.clone();
        results.push(result);
        Context {
            input: self.input.clone(),
            results,
        }
    }
    pub fn build(&self, titles: &Titles) -> Option<JsonValue> {
        if self.results.is_empty() {
            Some(self.input.as_ref().clone())
        } else {
            let mut mp = IndexMap::new();
            for (title, value) in titles.titles.iter().zip(&self.results) {
                match value {
                    Some(value) => {
                        mp.insert(title.clone(), value.clone());
                    }
                    None => {}
                }
            }
            Some(JsonValue::Object(mp))
        }
    }
    pub fn get(&self, index: usize) -> &Option<JsonValue> {
        match self.results.get(index) {
            None => &None,
            Some(t) => t,
        }
    }

    pub fn input(&self) -> &Rc<JsonValue> {
        &self.input
    }

    pub fn key(&self) -> ContextKey {
        if self.results.is_empty() {
            ContextKey::Value(self.input.deref().clone())
        } else {
            ContextKey::Results(self.results.clone())
        }
    }
}

pub type Result = std::result::Result<(), ProcessError>;

pub trait Process {
    fn start(&mut self, titles_so_far: Titles) -> Result;
    fn process(&mut self, context: Context) -> Result;
    fn complete(&mut self) -> Result;
}
