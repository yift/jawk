use std::collections::HashMap;
use std::ops::Deref;
use std::rc::Rc;

use indexmap::IndexMap;
use std::fmt::Error as FormatError;
use std::io::Error as IoEror;
use thiserror::Error;

use crate::json_value::JsonValue;
use crate::reader::Location;
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
    titles: Vec<Rc<String>>,
}
impl Titles {
    pub fn with_title(&self, title: &Rc<String>) -> Self {
        let mut titles = self.titles.clone();
        titles.push(title.clone());
        Titles { titles }
    }

    pub fn len(&self) -> usize {
        self.titles.len()
    }

    pub fn to_list(&self) -> Vec<Option<JsonValue>> {
        let mut lst = Vec::with_capacity(self.titles.len());
        for str in &self.titles {
            let value = Some(str.deref().clone().into());
            lst.push(value);
        }
        lst
    }
}
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum ContextKey {
    Value(JsonValue),
    Results(Vec<Option<JsonValue>>),
}

#[derive(Clone)]
pub struct InputContext {
    pub start_location: Location,
    pub end_location: Location,
    pub file_index: u64,
    pub index: u64,
}
pub struct Context {
    input: Rc<JsonValue>,
    results: Vec<(Rc<String>, Option<JsonValue>)>,
    parent_inputs: Vec<Rc<JsonValue>>,
    variables: Rc<HashMap<String, JsonValue>>,
    definitions: Rc<HashMap<String, Rc<dyn Get>>>,
    input_context: Option<Rc<InputContext>>,
}
impl Context {
    pub fn new_empty() -> Self {
        Context {
            input: Rc::new(JsonValue::Null),
            results: Vec::new(),
            parent_inputs: Vec::new(),
            variables: Rc::new(HashMap::new()),
            definitions: Rc::new(HashMap::new()),
            input_context: None,
        }
    }
    pub fn new_with_no_context(input: JsonValue) -> Self {
        Context {
            input: Rc::new(input),
            results: Vec::new(),
            parent_inputs: Vec::new(),
            variables: Rc::new(HashMap::new()),
            definitions: Rc::new(HashMap::new()),
            input_context: None,
        }
    }
    pub fn new_with_input(
        input: JsonValue,
        start_location: Location,
        end_location: Location,
        file_index: u64,
        index: u64,
    ) -> Self {
        let input_context = InputContext {
            start_location,
            end_location,
            file_index,
            index,
        };
        Context {
            input: Rc::new(input),
            results: Vec::new(),
            parent_inputs: Vec::new(),
            variables: Rc::new(HashMap::new()),
            definitions: Rc::new(HashMap::new()),
            input_context: Some(Rc::new(input_context)),
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
            input_context: self.input_context.clone(),
        }
    }
    pub fn with_result(&self, title: &Rc<String>, result: Option<JsonValue>) -> Self {
        let mut results = self.results.clone();
        results.push((title.clone(), result));
        Context {
            input: self.input().clone(),
            results,
            parent_inputs: Vec::new(),
            variables: self.variables.clone(),
            definitions: self.definitions.clone(),
            input_context: self.input_context.clone(),
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
            input_context: self.input_context.clone(),
        }
    }
    pub fn with_variables(&self, variables: &Rc<HashMap<String, JsonValue>>) -> Self {
        Context {
            input: self.input().clone(),
            results: self.results.clone(),
            parent_inputs: Vec::new(),
            variables: variables.clone(),
            definitions: self.definitions.clone(),
            input_context: self.input_context.clone(),
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
            input_context: self.input_context.clone(),
        }
    }
    pub fn with_definitions(&self, definitions: &Rc<HashMap<String, Rc<dyn Get>>>) -> Self {
        Context {
            input: self.input().clone(),
            results: self.results.clone(),
            parent_inputs: Vec::new(),
            variables: self.variables.clone(),
            definitions: definitions.clone(),
            input_context: self.input_context.clone(),
        }
    }
    pub fn build(&self) -> Option<JsonValue> {
        if self.results.is_empty() {
            Some(self.input().deref().clone())
        } else {
            let mut mp = IndexMap::new();
            for (title, value) in &self.results {
                match value {
                    Some(value) => {
                        mp.insert(title.deref().clone(), value.clone());
                    }
                    None => {}
                }
            }
            Some(JsonValue::Object(mp))
        }
    }
    pub fn to_list(&self) -> Vec<Option<JsonValue>> {
        self.results.iter().map(|i| i.1.clone()).collect()
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
            ContextKey::Results(self.to_list())
        }
    }

    pub fn input_context(&self) -> Option<Rc<InputContext>> {
        self.input_context.clone()
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
