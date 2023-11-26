use std::env::var;
use std::str::FromStr;

use const_format::formatcp;

use crate::json_parser::JsonParser;
use crate::selection::Selection;
use crate::{
    functions_definitions::{Arguments, Example, FunctionDefinitions, FunctionsGroup},
    json_value::JsonValue,
    reader::from_string,
    selection::Get,
};

pub fn get_string_functions() -> FunctionsGroup {
    FunctionsGroup {
        name: "String functions",
        functions: vec![
            FunctionDefinitions {
                name: "parse",
                aliases: vec!["parse_json"],
                min_args_count: 1,
                max_args_count: 1,
                build_extractor: |args| {
                    struct Impl(Arguments);
                    impl Get for Impl {
                        fn get(&self, value: &Option<JsonValue>) -> Option<JsonValue> {
                            match self.0.apply(value, 0) {
                                Some(JsonValue::String(str)) => {
                                    let mut reader = from_string(&str);
                                    match reader.next_json_value() {
                                        Ok(Some(first_value)) => match reader.next_json_value() {
                                            Ok(None) => Some(first_value),
                                            _ => None,
                                        },
                                        _ => None,
                                    }
                                }
                                _ => None,
                            }
                        }
                    }
                    Box::new(Impl(Arguments::new(args)))
                },
                description: vec!["Parse a string into JSON value."],
                examples: vec![
                    Example {
                        input: None,
                        arguments: vec!["\"[1, 2, 3, 4]\""],
                        output: Some("[1, 2, 3, 4]"),
                    },
                    Example {
                        input: None,
                        arguments: vec!["\"312\""],
                        output: Some("312"),
                    },
                    Example {
                        input: None,
                        arguments: vec!["\"{}\""],
                        output: Some("{}"),
                    },
                    Example {
                        input: None,
                        arguments: vec!["400"],
                        output: None,
                    },
                ],
            },
            FunctionDefinitions {
                name: "parse_selection",
                aliases: vec![],
                min_args_count: 1,
                max_args_count: 1,
                build_extractor: |args| {
                    struct Impl(Arguments);
                    impl Get for Impl {
                        fn get(&self, value: &Option<JsonValue>) -> Option<JsonValue> {
                            match self.0.apply(value, 0) {
                                Some(JsonValue::String(str)) => {
                                    match Selection::from_str(str.as_str()) {
                                        Ok(selection) => selection.get(value),
                                        _ => None,
                                    }
                                }
                                _ => None,
                            }
                        }
                    }
                    Box::new(Impl(Arguments::new(args)))
                },
                description: vec!["Parse a string into a new selection."],
                examples: vec![
                    Example {
                        input: None,
                        arguments: vec!["\"(+ 10 11)\""],
                        output: Some("21"),
                    },
                    Example {
                        input: None,
                        arguments: vec!["false"],
                        output: None,
                    },
                ],
            },
            FunctionDefinitions {
                name: "env",
                aliases: vec!["$"],
                min_args_count: 1,
                max_args_count: 1,
                build_extractor: |args| {
                    struct Impl(Arguments);
                    impl Get for Impl {
                        fn get(&self, value: &Option<JsonValue>) -> Option<JsonValue> {
                            if let Some(JsonValue::String(str)) = self.0.apply(value, 0) {
                                if let Ok(value) = var(str) {
                                    Some(value.into())
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
                description: vec!["Get enviornment variable."],
                examples: vec![Example {
                    input: None,
                    arguments: vec!["\"PATH\""],
                    output: Some(formatcp!("\"{}\"", env!("PATH"))),
                }],
            },
        ],
    }
}
