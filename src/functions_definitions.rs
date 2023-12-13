use std::collections::HashMap;

use crate::functions::basic_functions::get_basic_functions;
use crate::functions::boolean_functions::get_boolean_functions;
use crate::functions::list_functions::get_list_functions;
use crate::functions::number_functions::get_number_functions;
use crate::functions::object_functions::get_object_functions;
use crate::functions::string_functions::get_string_functions;
use crate::functions::time_functions::get_time_functions;
use crate::functions::type_functions::get_type_functions;
use crate::json_parser::JsonParser;
use crate::processor::Context;
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
    input: Option<&'static str>,
    pub arguments: Vec<&'static str>,
    // For test only
    output: Option<&'static str>,
}

impl Example {
    pub fn new() -> Self {
        Example {
            input: None,
            arguments: vec![],
            output: None,
        }
    }
    pub fn input(mut self, inpput: &'static str) -> Self {
        self.input = Some(inpput);
        self
    }
    pub fn add_argument(mut self, arg: &'static str) -> Self {
        self.arguments.push(arg);
        self
    }
    pub fn expected_output(mut self, output: &'static str) -> Self {
        self.output = Some(output);
        self
    }
}

pub struct FunctionDefinitions {
    name: &'static str,
    aliases: Vec<&'static str>,
    min_args_count: usize,
    max_args_count: usize,
    build_extractor: Factory,
    description: Vec<&'static str>,
    examples: Vec<Example>,
}

impl FunctionDefinitions {
    pub fn new(
        name: &'static str,
        min_args_count: usize,
        max_args_count: usize,
        build_extractor: Factory,
    ) -> Self {
        FunctionDefinitions {
            name,
            aliases: vec![],
            min_args_count,
            max_args_count,
            build_extractor,
            description: vec![],
            examples: vec![],
        }
    }

    pub fn add_alias(mut self, alias: &'static str) -> Self {
        self.aliases.push(alias);
        self
    }
    pub fn add_description_line(mut self, line: &'static str) -> Self {
        self.description.push(line);
        self
    }
    pub fn add_example(mut self, example: Example) -> Self {
        self.examples.push(example);
        self
    }
    pub fn name(&self) -> String {
        self.name.into()
    }
    pub fn names(&self) -> Vec<&'static str> {
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

pub struct FunctionsGroup {
    name: &'static str,
    functions: Vec<FunctionDefinitions>,
}

impl FunctionsGroup {
    pub fn new(name: &'static str) -> Self {
        FunctionsGroup {
            name,
            functions: vec![],
        }
    }
    pub fn add_function(mut self, function: FunctionDefinitions) -> Self {
        self.functions.push(function);
        self
    }
}

pub trait Arguments {
    fn apply(&self, value: &Context, index: usize) -> Option<JsonValue>;
}
impl Arguments for Vec<Box<dyn Get>> {
    fn apply(&self, context: &Context, index: usize) -> Option<JsonValue> {
        if let Some(arg) = self.get(index) {
            arg.get(context)
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
    static ref TIME_FUNCTIONS: FunctionsGroup = get_time_functions();
    static ref ALL_FUNCTIONS: Vec<&'static FunctionsGroup> = vec![
        &BASIC_FUNCTIONS,
        &TYPES_FUNCTIONS,
        &LIST_FUNCTIONS,
        &OBJECT_FUNCTIONS,
        &NUMBER_FUNCTIONS,
        &STRING_FUNCTIONS,
        &BOOLEAN_FUNCTIONS,
        &TIME_FUNCTIONS,
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
                    json
                } else {
                    JsonValue::null()
                };
                let args = example.arguments.join(", ");
                let run = format!("({} {})", name, args);
                println!("        running: \"{}\"", run);
                let selection = Selection::from_str(&run).unwrap();
                match selection.get(&Context::new(json)) {
                    None => println!("        will return nothing"),
                    Some(result) => println!("        will give: \"{}\"", result),
                };
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashSet;

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
                        json.unwrap()
                    } else {
                        JsonValue::Null
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
                    let result = selection.get(&Context::new(json));
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
    #[test]
    fn test_no_dulicates() -> selection::Result<()> {
        let mut names = HashSet::new();
        for group in ALL_FUNCTIONS.iter() {
            println!("Looking at group: {}", group.name);
            for func in group.functions.iter() {
                println!("\t looking at function: {}", func.name);
                assert_eq!(names.insert(func.name.to_string()), true);
                for alias in &func.aliases {
                    println!("\t\t looking at alias: {}", alias);
                    assert_eq!(names.insert(alias.to_string()), true);
                }
            }
        }
        Ok(())
    }
}
