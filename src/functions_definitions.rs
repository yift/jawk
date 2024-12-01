use std::borrow::Cow;
use std::collections::HashMap;
use std::iter;
use std::rc::Rc;
use std::sync::LazyLock;

use crate::functions::all::group;
#[cfg(feature = "create-docs")]
use crate::json_parser::JsonParser;
use crate::processor::Context;
#[cfg(feature = "create-docs")]
use crate::{reader::from_string, selection::Selection};

use crate::{json_value::JsonValue, selection::Get};
use clap::builder::PossibleValue;
#[cfg(feature = "create-docs")]
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

#[allow(dead_code)]
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

    pub fn file_name(&self) -> String {
        let replacer = vec![
            ("?", "__qm__"),
            ("/", "__sl__"),
            (">", "__gt__"),
            ("<", "__st__"),
            ("%", "__pc__"),
            ("\"", "__qt__"),
            ("|", "__pp__"),
        ];
        let mut function_file_name = self.name();
        for (replace, by) in replacer {
            function_file_name = function_file_name.replace(replace, by);
        }
        function_file_name
    }
}

pub struct FunctionsGroup {
    pub group_name: &'static str,
    description: Vec<&'static str>,
    sub_groups: Vec<FunctionsGroup>,
    group_functions: Vec<FunctionDefinitions>,
    root: bool,
}

impl FunctionsGroup {
    pub fn new(group_name: &'static str) -> Self {
        FunctionsGroup {
            group_name,
            description: vec![],
            sub_groups: vec![],
            group_functions: vec![],
            root: false,
        }
    }
    pub fn add_function(mut self, function: FunctionDefinitions) -> Self {
        self.group_functions.push(function);
        self
    }
    pub fn add_description_line(mut self, line: &'static str) -> Self {
        self.description.push(line);
        self
    }
    pub fn add_sub_group(mut self, sub_group: FunctionsGroup) -> Self {
        self.sub_groups.push(sub_group);
        self
    }
    pub fn root(mut self) -> Self {
        self.root = true;
        self
    }

    fn all_sub_group_iter(&'static self) -> Box<dyn Iterator<Item = &'static FunctionsGroup>> {
        Box::new(
            iter::once(self).chain(self.sub_groups.iter().flat_map(|f| f.all_sub_group_iter())),
        )
    }

    fn all_functions_iter(&'static self) -> impl Iterator<Item = &'static FunctionDefinitions> {
        self.all_sub_group_iter()
            .flat_map(|f| f.group_functions.iter())
    }

    #[cfg(feature = "create-docs")]
    pub fn functions(&self) -> impl Iterator<Item = &FunctionDefinitions> {
        self.group_functions.iter()
    }

    pub fn is_root(&self) -> bool {
        self.root
    }
    #[cfg(feature = "create-docs")]
    pub fn subgroups(&self) -> impl Iterator<Item = &FunctionsGroup> {
        self.sub_groups.iter()
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
static ALL_GROUPS: LazyLock<FunctionsGroup> = LazyLock::new(group);
static NAME_TO_FUNCTION: LazyLock<HashMap<&'static str, &'static FunctionDefinitions>> =
    LazyLock::new(|| {
        ALL_GROUPS
            .all_functions_iter()
            .flat_map(|f| f.names().iter().map(move |n| (*n, f)).collect::<Vec<_>>())
            .collect()
    });

pub fn find_function(name: &str) -> Result<&'static FunctionDefinitions, FunctionDefinitionsError> {
    NAME_TO_FUNCTION
        .get(name)
        .ok_or(FunctionDefinitionsError::UnknownFunction(name.to_string()))
        .copied()
}

pub fn create_possible_fn_help_types() -> Vec<PossibleValue> {
    let mut values = Vec::new();
    values.push(
        PossibleValue::new("functions").help("Additional help about the available functions"),
    );

    for group in ALL_GROUPS.all_sub_group_iter() {
        if !group.is_root() {
            values.push(PossibleValue::new(group.group_name).help(format!(
                "Additional help about the {} functions group",
                group.group_name
            )));
        }
        for function in &group.group_functions {
            for alias in function.names() {
                values.push(PossibleValue::new(alias).hide(true));
            }
        }
    }

    values
}
#[cfg(feature = "create-docs")]
pub fn get_fn_help(help_type: &str) -> Vec<String> {
    for group in ALL_GROUPS.all_sub_group_iter() {
        if group.group_name == help_type {
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

pub fn get_fn_help_name(help_type: &str) -> String {
    for group in ALL_GROUPS.all_sub_group_iter() {
        if group.group_name == help_type {
            return help_type.into();
        }
    }
    let function = NAME_TO_FUNCTION.get(help_type);
    if let Some(&function) = function {
        function.file_name()
    } else {
        panic!("Can not find function {help_type}")
    }
}

#[cfg(feature = "create-docs")]
pub fn get_groups() -> &'static FunctionsGroup {
    &ALL_GROUPS
}
#[cfg(feature = "create-docs")]
fn create_group_detailed_help(group: &FunctionsGroup, indentation: String) -> Vec<String> {
    let mut help = Vec::new();
    for f in &group.group_functions {
        help.push(format!(
            "{} Function [`{}`]({}.md) - {}",
            indentation,
            f.name,
            f.file_name(),
            f.description.first().unwrap_or(&"")
        ));
    }
    #[cfg(feature = "create-docs")]
    for g in group.subgroups() {
        help.push(format!(
            "{} Group [`{}`]({}.md) - {}",
            indentation,
            g.group_name,
            g.group_name,
            g.description.first().unwrap_or(&"")
        ));

        let group_items = create_group_detailed_help(g, format!("  {}", indentation));

        help.extend(group_items);
    }
    help
}
#[cfg(feature = "create-docs")]
fn get_group_help(group: &FunctionsGroup) -> Vec<String> {
    let mut help = Vec::new();
    if group.is_root() {
        help.push("# Functions".to_string());
    } else {
        help.push(format!("# Function group {}", group.group_name));
    }
    for line in &group.description {
        help.push((*line).to_string());
    }

    help.push("Function group has those functions and groups:".into());
    help.extend(create_group_detailed_help(group, "*".into()));

    help.push("".into());
    help.push(
        "Use additional help with a function name to see more details about the function.".into(),
    );
    help
}
#[cfg(feature = "create-docs")]
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
    use std::{collections::HashSet, str::FromStr};

    use crate::{
        json_parser::JsonParser,
        reader::from_string,
        selection::{self, Selection},
    };

    use super::*;

    #[test]
    fn test_functions() -> selection::Result<()> {
        for group in ALL_GROUPS.all_sub_group_iter() {
            println!("Running group: {}", group.group_name);
            for func in &group.group_functions {
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
        for group in ALL_GROUPS.all_sub_group_iter() {
            println!("Looking at group: {}", group.group_name);
            assert!(names.insert(group.group_name.to_string()));
            for func in &group.group_functions {
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
