use std::collections::HashMap;

use crate::basic_functions::get_basic_functions;
use crate::boolean_functions::get_boolean_functions;
use crate::json_parser::JsonParser;
use crate::list_functions::get_list_functions;
use crate::number_functions::get_number_functions;
use crate::object_functions::get_object_functions;
use crate::string_functions::get_string_functions;
use crate::type_functions::get_type_functions;
use crate::{
    json_value::JsonValue,
    reader::from_string,
    selection::{Get, Selection},
};
use lazy_static::lazy_static;
use std::str::FromStr;
use thiserror::Error;

type Factory = fn(args: Vec<Box<dyn Get>>) -> Box<dyn Get>;

#[derive(Debug, Error)]
pub enum FunctionDefinitionsError {
    #[error("Function '{0}' is unknwon")]
    UnknownFunction(String),
    #[error("Mising argument for {0}, got only {1} needed {2}")]
    MissingArgument(String, usize, usize),
    #[error("Too many arguments for {0}, got only {1} needed {2}")]
    TooManyArgument(String, usize, usize),
}

pub struct Example {
    pub input: Option<&'static str>,
    pub arguments: Vec<&'static str>,
    // For test only
    pub output: Option<&'static str>,
}

pub struct FunctionDefinitions {
    pub name: &'static str,
    pub aliases: Vec<&'static str>,
    pub min_args_count: usize,
    pub max_args_count: usize,
    pub build_extractor: Factory,
    pub description: Vec<&'static str>,
    pub examples: Vec<Example>,
}

pub struct FunctionsGroup {
    pub name: &'static str,
    pub functions: Vec<FunctionDefinitions>,
}

impl FunctionDefinitions {
    fn name(&self) -> String {
        self.name.into()
    }
    fn names(&self) -> Vec<&'static str> {
        let mut vec = vec![];
        vec.push(self.name);
        for alias in &self.aliases {
            vec.push(alias);
        }
        vec
    }
    pub fn create(
        &self,
        args: Vec<Box<dyn Get>>,
    ) -> Result<Box<dyn Get>, FunctionDefinitionsError> {
        if args.len() < self.min_args_count {
            return Err(FunctionDefinitionsError::MissingArgument(
                self.name(),
                self.min_args_count,
                args.len(),
            ));
        }
        if args.len() > self.max_args_count {
            return Err(FunctionDefinitionsError::TooManyArgument(
                self.name(),
                args.len(),
                self.min_args_count,
            ));
        }
        Ok((self.build_extractor)(args))
    }
}

pub struct Arguments {
    pub args: Vec<Box<dyn Get>>,
}
impl Arguments {
    pub fn new(args: Vec<Box<dyn Get>>) -> Self {
        Arguments { args }
    }

    pub fn apply(&self, value: &Option<JsonValue>, index: usize) -> Option<JsonValue> {
        if let Some(arg) = self.args.get(index) {
            arg.get(value)
        } else {
            None
        }
    }
}

lazy_static! {
    static ref BASIC_FUNCTIONS: FunctionsGroup = get_basic_functions();
    static ref TYPES_FUNCTIONS: FunctionsGroup = get_type_functions();
    static ref NUMBER_FUNCTIONS: FunctionsGroup = get_number_functions();
    static ref OBJECT_FUNCTIONS: FunctionsGroup = get_object_functions();
    static ref LIST_FUNCTIONS: FunctionsGroup = get_list_functions();
    static ref STRING_FUNCTIONS: FunctionsGroup = get_string_functions();
    static ref BOOLEAN_FUNCTIONS: FunctionsGroup = get_boolean_functions();
    static ref ALL_FUNCTIONS: Vec<&'static FunctionsGroup> = vec![
        &BASIC_FUNCTIONS,
        &TYPES_FUNCTIONS,
        &LIST_FUNCTIONS,
        &OBJECT_FUNCTIONS,
        &NUMBER_FUNCTIONS,
        &STRING_FUNCTIONS,
        &BOOLEAN_FUNCTIONS,
    ];
    static ref NAME_TO_FUNCTION: HashMap<&'static str, &'static FunctionDefinitions> =
        ALL_FUNCTIONS
            .iter()
            .flat_map(|l| l.functions.iter())
            .flat_map(|f| f.names().iter().map(move |n| (*n, f)).collect::<Vec<_>>())
            .collect();
}

pub fn find_function(name: &str) -> Result<&'static FunctionDefinitions, FunctionDefinitionsError> {
    NAME_TO_FUNCTION
        .get(name)
        .ok_or(FunctionDefinitionsError::UnknownFunction(name.to_string()))
        .map(|f| *f)
}

pub fn print_help() {
    for group in ALL_FUNCTIONS.iter() {
        println!("--- {} ---", group.name);
        for func in group.functions.iter() {
            let name = func.name;
            println!("  {} function:", name);
            for alias in &func.aliases {
                println!("    * Can also be called as '{}'", alias);
            }
            for description in &func.description {
                println!("    {}", description);
            }
            println!("    For example:");
            for example in &func.examples {
                let json = if let Some(input) = example.input {
                    let input = input.to_string();
                    let mut reader = from_string(&input);
                    let json = reader.next_json_value().unwrap().unwrap();
                    println!("      for input: \"{}\"", json);
                    Some(json)
                } else {
                    None
                };
                let args = example.arguments.join(", ");
                let run = format!("({} {})", name, args);
                println!("        running: \"{}\"", run);
                let selection = Selection::from_str(&run).unwrap();
                match selection.get(&json) {
                    None => println!("        will return nothing"),
                    Some(result) => println!("        will give: \"{}\"", result),
                };
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::selection;

    use super::*;

    #[test]
    fn test_functions() -> selection::Result<()> {
        for group in ALL_FUNCTIONS.iter() {
            println!("Running group: {}", group.name);
            for func in group.functions.iter() {
                println!("\tRunning function: {}", func.name);
                for example in &func.examples {
                    let json = if let Some(input) = example.input {
                        let input = input.to_string();
                        let mut reader = from_string(&input);
                        let json = reader.next_json_value()?;
                        Some(json.unwrap())
                    } else {
                        None
                    };
                    let args = example.arguments.join(", ");
                    println!("\t\tRunning examplt: {}...", args);
                    let run = format!("({} {})", func.name, args);
                    let selection = Selection::from_str(&run)?;
                    let expected = example.output.map(|input| {
                        let input = input.to_string();
                        let mut reader = from_string(&input);
                        reader.next_json_value().unwrap().unwrap()
                    });
                    let result = selection.get(&json);
                    match &result {
                        Some(result) => println!("\t\t\tgot: {}", result),
                        None => println!("\t\t\tgot nothing"),
                    }
                    assert_eq!(result, expected);
                    println!("\t\tPassed");
                }
                println!("\tPassed");
            }
        }
        Ok(())
    }
}
