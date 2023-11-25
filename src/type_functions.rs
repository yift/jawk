use crate::{
    functions_definitions::{Arguments, Example, FunctionDefinitions, FunctionsGroup},
    json_value::JsonValue,
    selection::Get,
};

pub fn get_type_functions() -> FunctionsGroup {
    FunctionsGroup {
        name: "Type functions",
        functions: vec![
            FunctionDefinitions {
                name: "array?",
                aliases: vec!["list?"],
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
                        arguments: vec!["[1, 2, 3, 4]"],
                    },
                    Example {
                        input: None,
                        arguments: vec!["312"],
                    },
                ],
            },
            FunctionDefinitions {
                name: "object?",
                aliases: vec!["map?", "hash?"],
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
                        arguments: vec!["[1, 2, 3, 4]"],
                    },
                    Example {
                        input: None,
                        arguments: vec!["{\"key\": 12}"],
                    },
                ],
            },
            FunctionDefinitions {
                name: "null?",
                aliases: vec!["nil?"],
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
                        arguments: vec!["null"],
                    },
                    Example {
                        input: None,
                        arguments: vec!["1"],
                    },
                ],
            },
            FunctionDefinitions {
                name: "bool?",
                aliases: vec!["boolean?"],
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
                        arguments: vec!["false"],
                    },
                    Example {
                        input: None,
                        arguments: vec!["\"false\""],
                    },
                ],
            },
            FunctionDefinitions {
                name: "number?",
                aliases: vec![],
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
                        arguments: vec!["\"str\""],
                    },
                    Example {
                        input: None,
                        arguments: vec!["1.32"],
                    },
                ],
            },
            FunctionDefinitions {
                name: "string?",
                aliases: vec![],
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
                        arguments: vec!["\"one\""],
                    },
                    Example {
                        input: None,
                        arguments: vec!["1.32"],
                    },
                ],
            },
        ],
    }
}
