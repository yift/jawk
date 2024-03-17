use std::borrow::Cow;
use std::collections::HashMap;
use std::rc::Rc;

use crate::functions::basic::group as get_basic_functions;
use crate::functions::boolean::group as get_boolean_functions;
use crate::functions::list::group as get_list_functions;
use crate::functions::number::group as get_number_functions;
use crate::functions::object::group as get_object_functions;
use crate::functions::proccess::group as get_exec_functions;
use crate::functions::string::group as get_string_functions;
use crate::functions::time::group as get_time_functions;
use crate::functions::type_group::group as get_type_functions;
use crate::functions::variables::group as get_variable_functions;
use crate::json_parser::JsonParser;
use crate::processor::Context;
use crate::{
    json_value::JsonValue,
    reader::from_string,
    selection::{Get, Selection},
};
use clap::builder::PossibleValue;
use lazy_static::lazy_static;
use std::str::FromStr;
use thiserror::Error;

type Factory = fn(args: Vec<Rc<dyn Get>>) -> Rc<dyn Get>;

#[derive(Debug, Error)]
pub enum FunctionDefinitionsError {
    #[error("Function '{0}' is unknwon")]
    UnknownFunction(String),
    #[error("Mising argument for {0}, got only {1} needed {2}")]
    MissingArgument(String, usize, usize),
    #[error("Too many arguments for {0}, got only {1} needed {2}")]
    TooManyArgument(String, usize, usize),
}

enum ValidOutputForTest {
    String(Cow<'static, str>),
    Function(fn(&Option<JsonValue>) -> bool),
}

pub struct Example {
    input: Option<&'static str>,
    pub arguments: Vec<&'static str>,
    // For test only
    output: Option<ValidOutputForTest>,
    // For docs only
    explain: Option<Cow<'static, str>>,
    // For docs only
    acurate: bool,
}

impl Example {
    pub fn new() -> Self {
        Example {
            input: None,
            arguments: vec![],
            output: None,
            explain: None,
            acurate: true,
        }
    }
    pub fn input(mut self, input: &'static str) -> Self {
        self.input = Some(input);
        self
    }
    pub fn add_argument(mut self, arg: &'static str) -> Self {
        self.arguments.push(arg);
        self
    }
    pub fn expected_output(mut self, output: &'static str) -> Self {
        self.output = Some(ValidOutputForTest::String(output.into()));
        self
    }
    pub fn validate_output(mut self, compare: fn(&Option<JsonValue>) -> bool) -> Self {
        self.output = Some(ValidOutputForTest::Function(compare));
        self
    }
    pub fn expected_json(mut self, output: Option<JsonValue>) -> Self {
        self.output = output.map(|v| ValidOutputForTest::String(format!("{v}").into()));
        self
    }
    pub fn explain(mut self, explain: &'static str) -> Self {
        self.explain = Some(explain.into());
        self
    }
    pub fn more_or_less(mut self) -> Self {
        self.acurate = false;
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
    pub fn create(&self, args: Vec<Rc<dyn Get>>) -> Result<Rc<dyn Get>, FunctionDefinitionsError> {
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

impl Arguments for Vec<Rc<dyn Get>> {
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
    static ref VARIABLE_FUNCTIONS: FunctionsGroup = get_variable_functions();
    static ref EXEC_FUNCTIONS: FunctionsGroup = get_exec_functions();
    static ref ALL_FUNCTIONS: Vec<&'static FunctionsGroup> = vec![
        &BASIC_FUNCTIONS,
        &TYPES_FUNCTIONS,
        &LIST_FUNCTIONS,
        &OBJECT_FUNCTIONS,
        &NUMBER_FUNCTIONS,
        &STRING_FUNCTIONS,
        &BOOLEAN_FUNCTIONS,
        &TIME_FUNCTIONS,
        &VARIABLE_FUNCTIONS,
        &EXEC_FUNCTIONS,
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

pub fn create_possible_fn_help_types() -> Vec<PossibleValue> {
    let mut values = Vec::new();
    values.push(
        PossibleValue::new("functions").help("Additional help about the available functions"),
    );

    for &group in ALL_FUNCTIONS.iter() {
        values.push(PossibleValue::new(group.name).help(format!(
            "Additional help about the {} functions group",
            group.name
        )));
        for function in &group.functions {
            for alias in function.names() {
                values.push(PossibleValue::new(alias).hide(true));
            }
        }
    }

    values
}

pub fn get_fn_help(help_type: &str) -> Vec<String> {
    if help_type == "functions" {
        let mut help = Vec::new();
        help.push("# Functions".into());
        help.push(
            "Functions allow one to manipulate the input. The functions format is `(<function-name> <arg0> <arg1> ..)` where `<argN>` are functions or other types of selection.".into()
        );
        help.push("See additional help for selection for more details.".into());
        help.push(format!(
            "There are {} functions group available:",
            ALL_FUNCTIONS.len()
        ));
        for &group in ALL_FUNCTIONS.iter() {
            help.push(format!("* *{}* functions.", group.name));
        }
        help.push(String::new());
        help.push(
            "See additional help with the group name to see the list of available functions in that group.".into()
        );
        help
    } else {
        for &group in ALL_FUNCTIONS.iter() {
            if group.name == help_type {
                return get_group_help(group);
            }
        }
        let function = NAME_TO_FUNCTION.get(help_type);
        if let Some(&function) = function {
            get_function_help(function)
        } else {
            panic!("Can not find function {help_type}")
        }
    }
}

#[cfg(feature = "create-docs")]
pub fn get_groups_and_funs() -> Vec<(String, Vec<String>)> {
    ALL_FUNCTIONS
        .iter()
        .map(|g| {
            let funcs = g.functions.iter().map(|f| f.name.to_string()).collect();
            (g.name.to_string(), funcs)
        })
        .collect()
}

fn get_group_help(group: &FunctionsGroup) -> Vec<String> {
    let mut help = Vec::new();
    help.push(format!("# Function group {}", group.name));
    help.push(format!(
        "Function group {} has {} functions:",
        group.name,
        group.functions.len()
    ));
    for f in &group.functions {
        help.push(format!(
            "* `{}` - {}",
            f.name,
            f.description.first().unwrap_or(&"")
        ));
    }
    help.push(
        "Use additional help with a function name to see more details about the function.".into(),
    );
    help
}

fn get_function_help(func: &FunctionDefinitions) -> Vec<String> {
    let mut help = Vec::new();
    let name = func.name;
    help.push(format!("# `{name}` function:"));
    for alias in &func.aliases {
        help.push(format!("* Can also be called as `{alias}`\n"));
    }
    for description in &func.description {
        help.push((*description).to_string());
    }
    help.push(String::new());
    help.push("## Examples:".into());
    for example in &func.examples {
        let (json, input) = if let Some(input) = example.input {
            let input = input.to_string();
            let mut reader = from_string(&input);
            let json = reader.next_json_value().unwrap().unwrap();
            let input = format!(" for input: ```{}```", &json);
            (json, input)
        } else {
            (JsonValue::Null, String::new())
        };
        let args = example.arguments.join(", ");
        let run = format!("({name} {args})");
        help.push(format!("* running: `{run}`{input}"));
        let selection = Selection::from_str(&run).unwrap();
        match selection.get(&Context::new_with_no_context(json)) {
            None => help.push("  will return nothing".into()),
            Some(result) => {
                if example.acurate {
                    help.push(format!("  will give: `{result}`"));
                } else {
                    help.push(format!("  can give something like: `{result}`"));
                }
            }
        }
        match &example.explain {
            None => {}
            Some(explain) => help.push(format!("  Because {explain}")),
        }
        help.push("----".into());
        help.push(String::new());
    }
    help
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
            for func in &group.functions {
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
                    println!("\t\tRunning example: {args}...");
                    let run = format!("({} {})", func.name, args);
                    let selection = Selection::from_str(&run)?;
                    let result = selection.get(&Context::new_with_no_context(json));
                    match &result {
                        Some(result) => println!("\t\t\tgot: {result}"),
                        None => println!("\t\t\tgot nothing"),
                    }
                    match &example.output {
                        None => {
                            assert_eq!(result, None);
                        }
                        Some(ValidOutputForTest::String(str)) => {
                            let input = str.to_string();
                            let mut reader = from_string(&input);
                            let json = reader.next_json_value().unwrap().unwrap();
                            assert_eq!(result, Some(json));
                        }
                        Some(ValidOutputForTest::Function(fun)) => {
                            assert!(fun(&result));
                        }
                    }
                    println!("\t\tPassed");
                }
                println!("\tPassed");
            }
        }
        Ok(())
    }

    #[test]
    fn test_no_dulicates() {
        let mut names = HashSet::new();
        for group in ALL_FUNCTIONS.iter() {
            println!("Looking at group: {}", group.name);
            for func in &group.functions {
                println!("\t looking at function: {}", func.name);
                assert!(names.insert(func.name.to_string()));
                for alias in &func.aliases {
                    println!("\t\t looking at alias: {alias}");
                    assert!(names.insert((*alias).to_string()));
                }
            }
        }
    }

    #[test]
    fn test_create_with_missing_of_arguments() -> selection::Result<()> {
        let func = find_function("?")?;
        let error = func.create(Vec::new()).err().unwrap();

        assert!(matches!(
            error,
            FunctionDefinitionsError::MissingArgument(_, _, _)
        ));
        Ok(())
    }

    #[test]
    fn test_create_with_too_many_of_arguments() -> selection::Result<()> {
        let now = find_function("now")?.create(Vec::new()).unwrap();
        let func = find_function("?")?;
        let error = func
            .create(vec![
                now.clone(),
                now.clone(),
                now.clone(),
                now.clone(),
                now.clone(),
            ])
            .err()
            .unwrap();

        assert!(matches!(
            error,
            FunctionDefinitionsError::TooManyArgument(_, _, _)
        ));
        Ok(())
    }

    #[test]
    fn name_return_the_function_name() -> selection::Result<()> {
        let func = find_function("if")?;

        assert_eq!(func.name(), "?".to_string());
        Ok(())
    }
}
