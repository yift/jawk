use std::env::var;
use std::str::FromStr;

use const_format::formatcp;
use regex::Regex;

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
            FunctionDefinitions {
                name: "concat",
                aliases: vec![],
                min_args_count: 2,
                max_args_count: usize::MAX,
                build_extractor: |args| {
                    struct Impl(Arguments);
                    impl Get for Impl {
                        fn get(&self, value: &Option<JsonValue>) -> Option<JsonValue> {
                            let mut all = String::new();
                            for s in &self.0.args {
                                if let Some(JsonValue::String(str)) = s.get(value) {
                                    all.push_str(str.as_str());
                                } else {
                                    return None;
                                }
                            }
                            Some(all.into())
                        }
                    }
                    Box::new(Impl(Arguments::new(args)))
                },
                description: vec!["Concat all string arguments."],
                examples: vec![Example {
                    input: None,
                    arguments: vec!["\"one\"", "\" \"", "\"two\""],
                    output: Some("\"one two\""),
                },Example {
                    input: None,
                    arguments: vec!["\"one\"", "\" \"", "2"],
                    output: None,
                },],
            },
            FunctionDefinitions {
                name: "split",
                aliases: vec![],
                min_args_count: 2,
                max_args_count: 2,
                build_extractor: |args| {
                    struct Impl(Arguments);
                    impl Get for Impl {
                        fn get(&self, value: &Option<JsonValue>) -> Option<JsonValue> {
                            if let (
                                Some(JsonValue::String(str)),
                                Some(JsonValue::String(splitter))
                            ) = (self.0.apply(value, 0), self.0.apply(value, 1)) {
                                Some(
                                    str.split(splitter.as_str())
                                    .map(|f| JsonValue::String(f.to_string()))
                                    .collect::<Vec<_>>()
                                    .into()
                                )
                            } else {
                                None
                            }
                        }
                    }
                    Box::new(Impl(Arguments::new(args)))
                },
                description: vec![
                    "Split the string into array of strings.",
                ],
                examples: vec![
                    Example {
                        input: None,
                        arguments: vec!["\"one, two, three\"", "\", \""],
                        output: Some("[\"one\", \"two\", \"three\"]"),
                    },
                    Example {
                        input: None,
                        arguments: vec!["\"a|b|c\"", "\"|\""],
                        output: Some("[\"a\", \"b\", \"c\"]"),
                    },
                ]
            },
                FunctionDefinitions {
                name: "match",
                aliases: vec!["match_regex"],
                min_args_count: 2,
                max_args_count: 2,
                build_extractor: |args| {
                    struct Impl(Arguments);
                    impl Get for Impl {
                        fn get(&self, value: &Option<JsonValue>) -> Option<JsonValue> {
                            if let (Some(JsonValue::String(str)), Some(JsonValue::String(regex))) = (self.0.apply(value, 0), self.0.apply(value, 1)) {
                                if let Ok(regex) = Regex::new(regex.as_str()) {
                                    Some(regex.is_match(&str).into())
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
                description: vec!["Return true if the first string argument match the second regular expression argument."],
                examples: vec![Example {
                    input: None,
                    arguments: vec!["\"test\"", "\"[a-z]+\""],
                    output: Some("true"),
                },Example {
                    input: None,
                    arguments: vec!["\"test\"", "\"[0-9]+\""],
                    output: Some("false"),
                },Example {
                    input: None,
                    arguments: vec!["\"test\"", "\"[0-9\""],
                    output: None,
                }],
            },
            FunctionDefinitions {
                name: "extract_regex_group",
                aliases: vec![],
                min_args_count: 3,
                max_args_count: 3,
                build_extractor: |args| {
                    struct Impl(Arguments);
                    impl Get for Impl {
                        fn get(&self, value: &Option<JsonValue>) -> Option<JsonValue> {
                            if let (
                                Some(JsonValue::String(str)),
                                Some(JsonValue::String(regex)),
                                Some(JsonValue::Number(index))
                            ) = (
                                self.0.apply(value, 0),
                                self.0.apply(value, 1),
                                self.0.apply(value, 2),
                            ) {
                                if let (Ok(regex), Ok(index)) = (Regex::new(regex.as_str()), TryInto::<usize>::try_into(index)) {
                                    if index > regex.captures_len() {
                                        return None;
                                    }
                                    if let Some(captures) = regex.captures(&str) {
                                        captures.get(index).map(|s| s.as_str().to_string().into())
                                    } else {
                                        None
                                    }
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
                description: vec!["Return the capture group within the string."],
                examples: vec![Example {
                    input: None,
                    arguments: vec!["\"hello 200 world\"", "\"[a-z ]+([0-9]+)[a-z ]+\"", "1"],
                    output: Some("\"200\""),
                },Example {
                    input: None,
                    arguments: vec!["\"hello 200 world\"", "\"[a-z ]+([0-9]+)[a-z ]+\"", "20"],
                    output: None,
                },Example {
                    input: None,
                    arguments: vec!["\"hello 200 world\"", "\"[a-z ]+([0-9]+)[a-z ]+\"", "0"],
                    output: Some("\"hello 200 world\""),
                },Example {
                    input: None,
                    arguments: vec!["\"test\"", "\"[0-9\"", "10"],
                    output: None,
                }],
            },
        ],
    }
}
