use std::collections::HashMap;
use std::ops::Deref;
use std::rc::Rc;

use indexmap::IndexMap;
use std::fmt::Error as FormatError;
use std::io::Error as IoEror;
use thiserror::Error;

use crate::json_value::JsonValue;
use crate::selection::Get;

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

    pub fn as_context(&self) -> Context {
        let mut headers = Context::new_empty();
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
    parent_inputs: Vec<Rc<JsonValue>>,
    variables: Rc<HashMap<String, JsonValue>>,
    definitions: Rc<HashMap<String, Rc<dyn Get>>>,
}
impl Context {
    pub fn new_empty() -> Self {
        Context {
            input: Rc::new(JsonValue::Null),
            results: Vec::new(),
            parent_inputs: Vec::new(),
            variables: Rc::new(HashMap::new()),
            definitions: Rc::new(HashMap::new()),
        }
    }
    pub fn new_with_input(input: JsonValue) -> Self {
        Context {
            input: Rc::new(input),
            results: Vec::new(),
            parent_inputs: Vec::new(),
            variables: Rc::new(HashMap::new()),
            definitions: Rc::new(HashMap::new()),
        }
    }
    pub fn with_inupt(&self, value: JsonValue) -> Self {
        let input = Rc::new(value);
        let mut parent_inputs = Vec::with_capacity(self.parent_inputs.len() + 1);
        parent_inputs.push(self.input.clone());
        for i in &self.parent_inputs {
            parent_inputs.push(i.clone());
        }
        Context {
            input,
            results: Vec::new(),
            parent_inputs,
            variables: self.variables.clone(),
            definitions: self.definitions.clone(),
        }
    }
    pub fn with_result(&self, result: Option<JsonValue>) -> Self {
        let mut results = self.results.clone();
        results.push(result);
        Context {
            input: self.input().clone(),
            results,
            parent_inputs: Vec::new(),
            variables: self.variables.clone(),
            definitions: self.definitions.clone(),
        }
    }
    pub fn with_variable(&self, name: String, value: JsonValue) -> Self {
        let mut variables = HashMap::with_capacity(self.variables.len() + 1);
        for (k, v) in self.variables.deref() {
            variables.insert(k.clone(), v.clone());
        }
        variables.insert(name, value);
        Context {
            input: self.input().clone(),
            results: self.results.clone(),
            parent_inputs: Vec::new(),
            variables: Rc::new(variables),
            definitions: self.definitions.clone(),
        }
    }
    pub fn with_variables(&self, variables: &Rc<HashMap<String, JsonValue>>) -> Self {
        Context {
            input: self.input().clone(),
            results: self.results.clone(),
            parent_inputs: Vec::new(),
            variables: variables.clone(),
            definitions: self.definitions.clone(),
        }
    }
    pub fn with_definition(&self, name: String, definition: &Rc<dyn Get>) -> Self {
        let mut definitions = HashMap::with_capacity(self.definitions.len() + 1);
        for (k, d) in self.definitions.deref() {
            definitions.insert(k.clone(), d.clone());
        }
        definitions.insert(name, definition.clone());
        Context {
            input: self.input().clone(),
            results: self.results.clone(),
            parent_inputs: Vec::new(),
            variables: self.variables.clone(),
            definitions: Rc::new(definitions),
        }
    }
    pub fn with_definitions(&self, definitions: &Rc<HashMap<String, Rc<dyn Get>>>) -> Self {
        Context {
            input: self.input().clone(),
            results: self.results.clone(),
            parent_inputs: Vec::new(),
            variables: self.variables.clone(),
            definitions: definitions.clone(),
        }
    }
    pub fn build(&self, titles: &Titles) -> Option<JsonValue> {
        if self.results.is_empty() {
            Some(self.input().as_ref().clone())
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

    pub fn get_variable_value(&self, name: &String) -> Option<&JsonValue> {
        self.variables.get(name)
    }

    pub fn get_definition(&self, name: &String) -> Option<&Rc<dyn Get>> {
        self.definitions.get(name)
    }

    pub fn input(&self) -> &Rc<JsonValue> {
        &self.input
    }

    pub fn parent_input(&self, count: usize) -> &JsonValue {
        if count == 0 {
            self.input()
        } else {
            self.parent_inputs
                .get(count - 1)
                .unwrap_or_else(|| self.input())
        }
    }

    pub fn key(&self) -> ContextKey {
        if self.results.is_empty() {
            ContextKey::Value(self.input().deref().clone())
        } else {
            ContextKey::Results(self.results.clone())
        }
    }
}

#[derive(Debug, PartialEq)]
pub enum ProcessDesision {
    Continue,
    Break,
}
pub type Result<T> = std::result::Result<T, ProcessError>;

pub trait Process {
    fn start(&mut self, titles_so_far: Titles) -> Result<()>;
    fn process(&mut self, context: Context) -> Result<ProcessDesision>;
    fn complete(&mut self) -> Result<()>;
}
