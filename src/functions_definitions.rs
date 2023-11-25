use std::collections::HashMap;

use crate::json_parser::JsonParser;
use crate::{
    json_value::{JsonValue, NumberValue},
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

struct Example {
    input: Option<&'static str>,
    arguments: Vec<&'static str>,
}

pub struct FunctionDefinitions {
    names: Vec<&'static str>,
    min_args_count: usize,
    max_args_count: usize,
    build_extractor: Factory,
    description: Vec<&'static str>,
    examples: Vec<Example>,
}

pub struct FunctionsGroup {
    name: &'static str,
    functions: Vec<FunctionDefinitions>,
}

impl FunctionDefinitions {
    fn name(&self) -> String {
        self.names.first().unwrap().to_string()
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

struct Arguments {
    args: Vec<Box<dyn Get>>,
}
impl Arguments {
    fn new(args: Vec<Box<dyn Get>>) -> Self {
        Arguments { args }
    }

    fn apply(&self, value: &Option<JsonValue>, index: usize) -> Option<JsonValue> {
        if let Some(arg) = self.args.get(index) {
            arg.get(value)
        } else {
            None
        }
    }
}

lazy_static! {

static ref BASIC_FUNCTIONS: FunctionsGroup = FunctionsGroup{ name: "Basic functions", functions: vec![
        FunctionDefinitions {
            names: vec!["get","[]"],
            min_args_count: 2,
            max_args_count: 2,
            build_extractor: |args| {
                struct Impl(Arguments);
                impl Get for Impl {
                    fn get(&self, value: &Option<JsonValue>) -> Option<JsonValue> {
                        match self.0.apply(value, 0) {
                            Some(JsonValue::Object(map)) => {
                                if let Some(JsonValue::String(key)) = self.0.apply(value, 1) {
                                    map.get(&key).cloned()
                                } else {
                                    None
                                }
                            },
                            Some(JsonValue::Array(array)) => {
                                if let Some(JsonValue::Number(NumberValue::Positive(index))) = self.0.apply(value, 1) {
                                    array.get(index as usize).cloned()
                                } else {
                                    None
                                }
                            },
                             _ => None

                        }
                    }
                }
                Box::new(Impl(Arguments::new(args)))
            },
            description: vec!["Get an item from an array by index or from a map by key."],
            examples: vec![
                Example {
                    input: None,
                    arguments: vec!["[\"a\", \"b\", \"c\"]", "1"]
                },
                Example {
                    input: None,
                    arguments: vec!["{\"key-1\": 12, \"key-2\": 32}", "\"key-1\""]
                },
            ]
        },
        FunctionDefinitions {
            names: vec!["|"],
            min_args_count: 2,
            max_args_count: usize::MAX,
            build_extractor: combine_extractors,
            description: vec!["Pipe the output of one function to the next function."],
            examples: vec![Example {
                input: Some("{\"key\": [20, 40, 60, {\"key-2\": 100}]}"),
                arguments: vec!["(get . \"key\")", "(get . 3)", "(get . \"key-2\")"]
            },]
        },
        FunctionDefinitions {
            names: vec!["size", "count", "length", "len"],
            min_args_count: 1,
            max_args_count: 1,
            build_extractor: |args| {
                struct Impl(Arguments);
                impl Get for Impl {
                    fn get(&self, value: &Option<JsonValue>) -> Option<JsonValue> {
                        match self.0.apply(value, 0) {
                            Some(JsonValue::Object(map)) => {
                                Some(map.len().into())
                            },
                            Some(JsonValue::Array(map)) => {
                                Some(map.len().into())
                            },
                            Some(JsonValue::String(str)) => {
                                Some(str.len().into())
                            },
                             _ => None

                        }
                    }
                }
                Box::new(Impl(Arguments::new(args)))
            },
            description: vec![
                "Get the number of element in an array,",
                "the number of keys in an object or the number of characters",
                "in a string."
            ],
            examples: vec![
                Example {
                    input: None,
                    arguments: vec!["[1, 2, 3, 4]"]
                },
                Example {
                    input: None,
                    arguments: vec!["{\"key-1\": 1, \"key-2\": false}"]
                },
                Example {
                    input: None,
                    arguments: vec!["\"123\""]
                },
            ]
        },
        FunctionDefinitions {
            names: vec!["default", "defaults", "or_else"],
            min_args_count: 1,
            max_args_count: usize::MAX,
            build_extractor: |args| {
                struct Impl(Vec<Box<dyn Get>>);
                impl Get for Impl {
                    fn get(&self, value: &Option<JsonValue>) -> Option<JsonValue> {
                        for e in &self.0 {
                            let val = e.get(value);
                            if val.is_some() {
                                return val;
                            }
                        }
                        None
                    }
                }
                Box::new(Impl(args))
            },
            description: vec!["Get the first non empty value."],
            examples: vec![
                Example {
                    input: Some("{\"key-1\": 1, \"key-2\": false}"),
                    arguments: vec!["(get . 1)", "(get . \"key-1\")", "22"]
                },
                Example {
                    input: Some("{\"key-1\": 1, \"key-2\": false}"),
                    arguments: vec!["(get . 1)", "(get . \"key-3\")", "22"]
                },
            ]
        },
        FunctionDefinitions {
            names: vec!["?", "if"],
            min_args_count: 3,
            max_args_count: 3,
            build_extractor: |args| {
                struct Impl(Arguments);
                impl Get for Impl {
                    fn get(&self, value: &Option<JsonValue>) -> Option<JsonValue> {
                        match self.0.apply(value, 0) {
                            Some(JsonValue::Boolean(true)) => self.0.apply(value, 1),
                            Some(JsonValue::Boolean(false)) => self.0.apply(value, 2),
                            _ => None,
                        }
                    }
                }
                Box::new(Impl(Arguments::new(args)))
            },
            description: vec![
                "Return the second argument if the first argument is true. Return the third argument",
                "if the first is false. Return nothing if the first argument is not Boolean"
            ],
            examples: vec![
                Example {
                    input: None,
                    arguments: vec!["true", "12", "22"]
                },
                Example {
                    input: None,
                    arguments: vec!["false", "12", "22"]
                },
                Example {
                    input: Some("[1, 2, 3]"),
                    arguments: vec!["(array? .)", "#1", "#2"]
                },
                Example {
                    input: Some("[1, 2, 3]"),
                    arguments: vec!["(null? .)", "#1", "#2"]
                },
            ]
        },
    ]};
    static ref TYPES_FUNCTIONS : FunctionsGroup = FunctionsGroup{ name: "Type functions", functions: vec![
        FunctionDefinitions {
            names: vec!["array?", "list?"],
            min_args_count: 1,
            max_args_count: 1,
            build_extractor: |args| {
                struct Impl(Arguments);
                impl Get for Impl {
                    fn get(&self, value: &Option<JsonValue>) -> Option<JsonValue> {
                        match self.0.apply(value, 0) {
                            Some(JsonValue::Array(_)) => Some(true.into()),
                            _ => Some(false.into()),
                        }
                    }
                }
                Box::new(Impl(Arguments::new(args)))
            },
            description: vec!["return true if the argument is an array."],
            examples: vec![
                Example {
                    input: None,
                    arguments: vec!["[1, 2, 3, 4]"]
                },
                Example {
                    input: None,
                    arguments: vec!["312"]
                },
            ]
        },
        FunctionDefinitions {
            names: vec!["object?", "map?", "hash?"],
            min_args_count: 1,
            max_args_count: 1,
            build_extractor: |args| {
                struct Impl(Arguments);
                impl Get for Impl {
                    fn get(&self, value: &Option<JsonValue>) -> Option<JsonValue> {
                        match self.0.apply(value, 0) {
                            Some(JsonValue::Object(_)) => Some(true.into()),
                            _ => Some(false.into()),
                        }
                    }
                }
                Box::new(Impl(Arguments::new(args)))
            },
            description: vec!["return true if the argument is an object."],
            examples: vec![
                Example {
                    input: None,
                    arguments: vec!["[1, 2, 3, 4]"]
                },
                Example {
                    input: None,
                    arguments: vec!["{\"key\": 12}"]
                },
            ]
        },
        FunctionDefinitions {
            names: vec!["null?", "nil?"],
            min_args_count: 1,
            max_args_count: 1,
            build_extractor: |args| {
                struct Impl(Arguments);
                impl Get for Impl {
                    fn get(&self, value: &Option<JsonValue>) -> Option<JsonValue> {
                        match self.0.apply(value, 0) {
                            Some(JsonValue::Null) => Some(true.into()),
                            _ => Some(false.into()),
                        }
                    }
                }
                Box::new(Impl(Arguments::new(args)))
            },
            description: vec!["return true if the argument is a null."],
            examples: vec![
                Example {
                    input: None,
                    arguments: vec!["null"]
                },
                Example {
                    input: None,
                    arguments: vec!["1"]
                },
            ]
        },
        FunctionDefinitions {
            names: vec!["bool?", "boolean?"],
            min_args_count: 1,
            max_args_count: 1,
            build_extractor: |args| {
                struct Impl(Arguments);
                impl Get for Impl {
                    fn get(&self, value: &Option<JsonValue>) -> Option<JsonValue> {
                        match self.0.apply(value, 0) {
                            Some(JsonValue::Boolean(_)) => Some(true.into()),
                            _ => Some(false.into()),
                        }
                    }
                }
                Box::new(Impl(Arguments::new(args)))
            },
            description: vec!["return true if the argument is a boolean."],
            examples: vec![
                Example {
                    input: None,
                    arguments: vec!["false"]
                },
                Example {
                    input: None,
                    arguments: vec!["\"false\""]
                },
            ]
        },
        FunctionDefinitions {
            names: vec!["number?"],
            min_args_count: 1,
            max_args_count: 1,
            build_extractor: |args| {
                struct Impl(Arguments);
                impl Get for Impl {
                    fn get(&self, value: &Option<JsonValue>) -> Option<JsonValue> {
                        match self.0.apply(value, 0) {
                            Some(JsonValue::Number(_)) => Some(true.into()),
                            _ => Some(false.into()),
                        }
                    }
                }
                Box::new(Impl(Arguments::new(args)))
            },
            description: vec!["return true if the argument is a number."],
            examples: vec![
                Example {
                    input: None,
                    arguments: vec!["\"str\""]
                },
                Example {
                    input: None,
                    arguments: vec!["1.32"]
                },
            ]
        },
        FunctionDefinitions {
            names: vec!["string?"],
            min_args_count: 1,
            max_args_count: 1,
            build_extractor: |args| {
                struct Impl(Arguments);
                impl Get for Impl {
                    fn get(&self, value: &Option<JsonValue>) -> Option<JsonValue> {
                        match self.0.apply(value, 0) {
                            Some(JsonValue::String(_)) => Some(true.into()),
                            _ => Some(false.into()),
                        }
                    }
                }
                Box::new(Impl(Arguments::new(args)))
            },
            description: vec!["return true if the argument is a string."],
            examples: vec![
                Example {
                    input: None,
                    arguments: vec!["\"one\""]
                },
                Example {
                    input: None,
                    arguments: vec!["1.32"]
                },
            ]
        },
    ]};
    static ref NUMBER_FUNCTIONS : FunctionsGroup = FunctionsGroup{ name: "Number functions", functions: vec![
        FunctionDefinitions {
            names: vec!["+", "add", "plus"],
            min_args_count: 2,
            max_args_count: usize::MAX,
            build_extractor: |args| {
                struct Impl(Arguments);
                impl Get for Impl {
                    fn get(&self, value: &Option<JsonValue>) -> Option<JsonValue> {
                        let mut sum = 0.0;
                        for s in &self.0.args {
                            if let Some(JsonValue::Number(num)) = s.get(value) {
                                let num: f64 = num.into();
                                sum += num;
                            } else {
                                return None;
                            }
                        }
                        Some(sum.into())
                    }
                }
                Box::new(Impl(Arguments::new(args)))
            },
            description: vec![
                "If all the arguments are number, add them.",
            ],
            examples: vec![
                Example {
                    input: None,
                    arguments: vec!["1", "3"],
                },
                Example {
                    input: None,
                    arguments: vec!["1", "10", "-4.1", "0.1"],
                },
            ],
        },
        FunctionDefinitions {
            names: vec!["-", "take_away", "minus", "substruct"],
            min_args_count: 2,
            max_args_count: 2,
            build_extractor: |args| {
                struct Impl(Arguments);
                impl Get for Impl {
                    fn get(&self, value: &Option<JsonValue>) -> Option<JsonValue> {
                        if let (Some(JsonValue::Number(num1)), Some(JsonValue::Number(num2))) = (self.0.apply(value, 0), self.0.apply(value, 1)) {
                            let num1: f64 = num1.into();
                            let num2: f64 = num2.into();
                            Some((num1 - num2).into())
                        } else {
                            None
                        }
                    }
                }
                Box::new(Impl(Arguments::new(args)))
            },
            description: vec![
                "Substract the second argument from the first one if both are number.",
            ],
            examples: vec![
                Example {
                    input: None,
                    arguments: vec!["100", "3"],
                },
                Example {
                    input: None,
                    arguments: vec!["10", "3.2"],
                },
            ],
        },
        FunctionDefinitions {
            names: vec!["*", "times", "multiple"],
            min_args_count: 2,
            max_args_count: usize::MAX,
            build_extractor: |args| {
                struct Impl(Arguments);
                impl Get for Impl {
                    fn get(&self, value: &Option<JsonValue>) -> Option<JsonValue> {
                        let mut sum = 1.0;
                        for s in &self.0.args {
                            if let Some(JsonValue::Number(num)) = s.get(value) {
                                let num: f64 = num.into();
                                sum *= num;
                            } else {
                                return None;
                            }
                        }
                        Some(sum.into())
                    }
                }
                Box::new(Impl(Arguments::new(args)))
            },
            description: vec![
                "If all the arguments are number, multiply them.",
            ],
            examples: vec![
                Example {
                    input: None,
                    arguments: vec!["2", "3"],
                },
                Example {
                    input: None,
                    arguments: vec!["2", "15", "0.1"],
                },
            ],
        },
        FunctionDefinitions {
            names: vec!["/", "divide"],
            min_args_count: 2,
            max_args_count: 2,
            build_extractor: |args| {
                struct Impl(Arguments);
                impl Get for Impl {
                    fn get(&self, value: &Option<JsonValue>) -> Option<JsonValue> {
                        if let (Some(JsonValue::Number(num1)), Some(JsonValue::Number(num2))) = (self.0.apply(value, 0), self.0.apply(value, 1)) {
                            let num1: f64 = num1.into();
                            let num2: f64 = num2.into();
                            if num2 != 0.0 {
                                Some((num1/ num2).into())
                            } else {
                                None
                            }
                        } else {
                            None
                        }
                    }
                }
                Box::new(Impl(Arguments::new(args)))
            },
            description: vec![
                "Divide the firs argument by the second argument. If the second argument is 0 will return nothing",
            ],
            examples: vec![
                Example {
                    input: None,
                    arguments: vec!["100", "25"],
                },
                Example {
                    input: None,
                    arguments: vec!["7", "2"],
                },
            ],
        },
        FunctionDefinitions {
            names: vec!["%", "mod", "modulu"],
            min_args_count: 2,
            max_args_count: 2,
            build_extractor: |args| {
                struct Impl(Arguments);
                impl Get for Impl {
                    fn get(&self, value: &Option<JsonValue>) -> Option<JsonValue> {
                        if let (Some(JsonValue::Number(num1)), Some(JsonValue::Number(num2))) = (self.0.apply(value, 0), self.0.apply(value, 1)) {
                            let num1: f64 = num1.into();
                            let num2: f64 = num2.into();
                            if num2 != 0.0 {
                                Some((num1 % num2).into())
                            } else {
                                None
                            }
                        } else {
                            None
                        }
                    }
                }
                Box::new(Impl(Arguments::new(args)))
            },
            description: vec![
                "Find the modulu of the division of the firs argument by the second argument. If the second argument is 0 will return nothing",
            ],
            examples: vec![
                Example {
                    input: None,
                    arguments: vec!["5", "3"],
                },
                Example {
                    input: None,
                    arguments: vec!["7", "2"],
                },
            ],
        },

        ]};
    static ref OBJECT_FUNCTIONS : FunctionsGroup =    FunctionsGroup { name: "Object functions", functions:  vec![
        FunctionDefinitions {
            names: vec!["keys"],
            min_args_count: 1,
            max_args_count: 1,
            build_extractor: |args| {
                struct Impl(Arguments);
                impl Get for Impl {
                    fn get(&self, value: &Option<JsonValue>) -> Option<JsonValue> {
                        if let Some(JsonValue::Object(map)) = self.0.apply(value, 0) {
                            Some(JsonValue::Array(
                                map.keys().cloned().map(JsonValue::String).collect(),
                            ))
                        } else {
                            None
                        }
                    }
                }
                Box::new(Impl(Arguments::new(args)))
            },
            description: vec!["Get the list of keys from an object."],
            examples: vec![Example {
                input: None,
                arguments: vec!["{\"key-1\": 1, \"key-2\": false}"]
            },]
        },
        FunctionDefinitions {
            names: vec!["values", "vals"],
            min_args_count: 1,
            max_args_count: 1,
            build_extractor: |args| {
                struct Impl(Arguments);
                impl Get for Impl {
                    fn get(&self, value: &Option<JsonValue>) -> Option<JsonValue> {
                        if let Some(JsonValue::Object(map)) = self.0.apply(value, 0) {
                            Some(JsonValue::Array(map.values().cloned().collect()))
                        } else {
                            None
                        }
                    }
                }
                Box::new(Impl(Arguments::new(args)))
            },
            description: vec!["Get the list of values from an object."],
            examples: vec![Example {
                input: None,
                arguments: vec!["{\"key-1\": 1, \"key-2\": false}"]
            },]
        },
    ]};
    static ref LIST_FUNCTIONS : FunctionsGroup =    FunctionsGroup { name: "List functions", functions:  vec![
        FunctionDefinitions {
            names: vec!["filter"],
            min_args_count: 2,
            max_args_count: 2,
            build_extractor: |args| {
                struct Impl(Arguments);
                impl Get for Impl {
                    fn get(&self, value: &Option<JsonValue>) -> Option<JsonValue> {
                        match self.0.apply(value, 0) {
                            Some(JsonValue::Array(list)) => {
                                let list: Vec<_> = list.into_iter()

                                .filter(|v| {
                                    let v = Some(v.clone());
                                    matches!(self.0.apply(&v, 1), Some(JsonValue::Boolean(true)))
                                }
                                ).collect();
                                Some(list.into())
                            },
                            _ => None,
                        }
                    }
                }
                Box::new(Impl(Arguments::new(args)))
            },
            description: vec![
                "Filter a list.",
                "If the first argument is a list, return all the values for which the second argument is a list."
            ],
            examples: vec![
                Example {
                    input: None,
                    arguments: vec!["[1, 2, 3, 4]", "true"]
                },
                Example {
                    input: None,
                    arguments: vec!["[1, 2, 3, 4]", "null"]
                },
                Example {
                    input: None,
                    arguments: vec!["[1, 2, 3, 4, \"one\", null]", "(string? .)"]
                },
                Example {
                    input: Some("[1, 2, null, \"a\", 4]"),
                    arguments: vec![".", "(number? .)"]
                },
            ]
        },
        FunctionDefinitions {
            names: vec!["sort", "order"],
            min_args_count: 1,
            max_args_count: 1,
            build_extractor: |args| {
                struct Impl(Arguments);
                impl Get for Impl {
                    fn get(&self, value: &Option<JsonValue>) -> Option<JsonValue> {
                        match self.0.apply(value, 0) {
                            Some(JsonValue::Array(list)) => {
                                let mut list: Vec<JsonValue> = list.clone();
                                list.sort();

                                Some(list.into())
                            },
                            _ => None,
                        }
                    }
                }
                Box::new(Impl(Arguments::new(args)))
            },
            description: vec![
                "Sort a list.",
                "If the first argument is a list, return list sorted."
            ],
            examples: vec![
                Example {
                    input: None,
                    arguments: vec!["[1, -2, 3.01, 3.05, -544, 100]"]
                },
                Example {
                    input: None,
                    arguments: vec!["[null, true, false, {}, [1, 2, 3], \"abc\", \"cde\", {\"key\": 12}]"]
                },
            ]
        },
        FunctionDefinitions {
            names: vec!["sort_by", "order_by"],
            min_args_count: 2,
            max_args_count: 2,
            build_extractor: |args| {
                struct Impl(Arguments);
                impl Get for Impl {
                    fn get(&self, value: &Option<JsonValue>) -> Option<JsonValue> {
                        match self.0.apply(value, 0) {
                            Some(JsonValue::Array(list)) => {
                                let mut list: Vec<JsonValue> = list.clone();
                                list.sort_by(|v1, v2| {
                                    let v1 = Some(v1.clone());
                                    let v1 = self.0.apply(&v1,1 );
                                    let v2 = Some(v2.clone());
                                    let v2 = self.0.apply(&v2,1 );
                                    v1.cmp(&v2)
                                });

                                Some(list.into())
                            },
                            _ => None,
                        }
                    }
                }
                Box::new(Impl(Arguments::new(args)))
            },
            description: vec![
                "Filter a list.",
                "If the first argument is a list, return list sorted by the second argument."
            ],
            examples: vec![
                Example {
                    input: None,
                    arguments: vec!["[\"12345\", \"5\", \"23\", \"abc\", \"-1-2\", \"\"]", "(len .)"]
                },
            ]
        },
    ]};
    static ref ALL_FUNCTIONS: Vec<&'static FunctionsGroup> = vec![
        &BASIC_FUNCTIONS,
        &TYPES_FUNCTIONS,
        &LIST_FUNCTIONS,
        &OBJECT_FUNCTIONS,
        &NUMBER_FUNCTIONS,
    ];
    static ref NAME_TO_FUNCTION: HashMap<&'static str, &'static FunctionDefinitions> = ALL_FUNCTIONS
        .iter()
        .flat_map(|l| l.functions.iter())
        .flat_map(|f| f.names.iter().map(move |n| (*n, f)))
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
            let name = func.names.first().unwrap();
            println!("  {} function:", name);
            for alias in &func.names {
                println!("    * Can be called as '{}'", alias);
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
                    println!("      * for input: \"{}\"", json);
                    Some(json)
                } else {
                    None
                };
                let args = example.arguments.join(", ");
                let run = format!("({} {})", name, args);
                let selection = Selection::from_str(&run).unwrap();
                let result = selection.get(&json).unwrap();
                println!("        running: \"{}\"", run);
                println!("        will give: \"{}\"", result);
            }
        }
    }
}

fn combine_extractors(args: Vec<Box<dyn Get>>) -> Box<dyn Get> {
    struct Impl(Vec<Box<dyn Get>>);
    impl Get for Impl {
        fn get(&self, value: &Option<JsonValue>) -> Option<JsonValue> {
            let mut value = value.clone();
            for e in &self.0 {
                value = e.get(&value);
            }
            value
        }
    }
    Box::new(Impl(args))
}
