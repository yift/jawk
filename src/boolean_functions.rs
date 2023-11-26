use crate::{
    functions_definitions::{Arguments, Example, FunctionDefinitions, FunctionsGroup},
    json_value::JsonValue,
    selection::Get,
};

pub fn get_boolean_functions() -> FunctionsGroup {
    FunctionsGroup {
        name: "Boolean functions",
        functions: vec![
            FunctionDefinitions {
                name: "=",
                aliases: vec![],
                min_args_count: 2,
                max_args_count: 2,
                build_extractor: |args| {
                    struct Impl(Arguments);
                    impl Get for Impl {
                        fn get(&self, value: &Option<JsonValue>) -> Option<JsonValue> {
                            if let (Some(val1), Some(val2)) =
                                (self.0.apply(value, 0), self.0.apply(value, 1))
                            {
                                let eq = val1.eq(&val2);
                                Some(eq.into())
                            } else {
                                None
                            }
                        }
                    }
                    Box::new(Impl(Arguments::new(args)))
                },
                description: vec!["Compare two value and return true if both are equals."],
                examples: vec![
                    Example {
                        input: None,
                        arguments: vec!["1", "3"],
                        output: Some("false"),
                    },
                    Example {
                        input: None,
                        arguments: vec!["1", "1"],
                        output: Some("true"),
                    },
                    Example {
                        input: None,
                        arguments: vec!["\"1\"", "1"],
                        output: Some("false"),
                    },
                    Example {
                        input: None,
                        arguments: vec!["\"abc\"", "\"abc\""],
                        output: Some("true"),
                    },
                ],
            },
            FunctionDefinitions {
                name: "!=",
                aliases: vec!["<>"],
                min_args_count: 2,
                max_args_count: 2,
                build_extractor: |args| {
                    struct Impl(Arguments);
                    impl Get for Impl {
                        fn get(&self, value: &Option<JsonValue>) -> Option<JsonValue> {
                            if let (Some(val1), Some(val2)) =
                                (self.0.apply(value, 0), self.0.apply(value, 1))
                            {
                                let eq = !val1.eq(&val2);
                                Some(eq.into())
                            } else {
                                None
                            }
                        }
                    }
                    Box::new(Impl(Arguments::new(args)))
                },
                description: vec!["Compare two value and return true if both are not equals."],
                examples: vec![
                    Example {
                        input: None,
                        arguments: vec!["1", "3"],
                        output: Some("true"),
                    },
                    Example {
                        input: None,
                        arguments: vec!["1", "1"],
                        output: Some("false"),
                    },
                    Example {
                        input: None,
                        arguments: vec!["\"1\"", "1"],
                        output: Some("true"),
                    },
                    Example {
                        input: None,
                        arguments: vec!["\"abc\"", "\"abc\""],
                        output: Some("false"),
                    },
                ],
            },
            FunctionDefinitions {
                name: "<",
                aliases: vec![],
                min_args_count: 2,
                max_args_count: 2,
                build_extractor: |args| {
                    struct Impl(Arguments);
                    impl Get for Impl {
                        fn get(&self, value: &Option<JsonValue>) -> Option<JsonValue> {
                            if let (Some(val1), Some(val2)) =
                                (self.0.apply(value, 0), self.0.apply(value, 1))
                            {
                                let eq = val1 < val2;
                                Some(eq.into())
                            } else {
                                None
                            }
                        }
                    }
                    Box::new(Impl(Arguments::new(args)))
                },
                description: vec![
                    "Compare two value and return true if the first is smaller than the second.",
                ],
                examples: vec![
                    Example {
                        input: None,
                        arguments: vec!["1", "3"],
                        output: Some("true"),
                    },
                    Example {
                        input: None,
                        arguments: vec!["1", "1"],
                        output: Some("false"),
                    },
                    Example {
                        input: None,
                        arguments: vec!["31", "1"],
                        output: Some("false"),
                    },
                ],
            },
            FunctionDefinitions {
                name: "<=",
                aliases: vec![],
                min_args_count: 2,
                max_args_count: 2,
                build_extractor: |args| {
                    struct Impl(Arguments);
                    impl Get for Impl {
                        fn get(&self, value: &Option<JsonValue>) -> Option<JsonValue> {
                            if let (Some(val1), Some(val2)) =
                                (self.0.apply(value, 0), self.0.apply(value, 1))
                            {
                                let eq = val1 <= val2;
                                Some(eq.into())
                            } else {
                                None
                            }
                        }
                    }
                    Box::new(Impl(Arguments::new(args)))
                },
                description: vec![
                    "Compare two value and return true if the first is smaller or eqauls than the second.",
                ],
                examples: vec![
                    Example {
                        input: None,
                        arguments: vec!["1", "3"],
                        output: Some("true"),
                    },
                    Example {
                        input: None,
                        arguments: vec!["1", "1"],
                        output: Some("true"),
                    },
                    Example {
                        input: None,
                        arguments: vec!["31", "1"],
                        output: Some("false"),
                    },
                ],
            },
            FunctionDefinitions {
                name: ">=",
                aliases: vec![],
                min_args_count: 2,
                max_args_count: 2,
                build_extractor: |args| {
                    struct Impl(Arguments);
                    impl Get for Impl {
                        fn get(&self, value: &Option<JsonValue>) -> Option<JsonValue> {
                            if let (Some(val1), Some(val2)) =
                                (self.0.apply(value, 0), self.0.apply(value, 1))
                            {
                                let eq = val1 >= val2;
                                Some(eq.into())
                            } else {
                                None
                            }
                        }
                    }
                    Box::new(Impl(Arguments::new(args)))
                },
                description: vec![
                    "Compare two value and return true if the first is greater or eqauls than the second.",
                ],
                examples: vec![
                    Example {
                        input: None,
                        arguments: vec!["1", "3"],
                        output: Some("false"),
                    },
                    Example {
                        input: None,
                        arguments: vec!["1", "1"],
                        output: Some("true"),
                    },
                    Example {
                        input: None,
                        arguments: vec!["31", "1"],
                        output: Some("true"),
                    },
                ],
            },
            FunctionDefinitions {
                name: ">",
                aliases: vec![],
                min_args_count: 2,
                max_args_count: 2,
                build_extractor: |args| {
                    struct Impl(Arguments);
                    impl Get for Impl {
                        fn get(&self, value: &Option<JsonValue>) -> Option<JsonValue> {
                            if let (Some(val1), Some(val2)) =
                                (self.0.apply(value, 0), self.0.apply(value, 1))
                            {
                                let eq = val1 > val2;
                                Some(eq.into())
                            } else {
                                None
                            }
                        }
                    }
                    Box::new(Impl(Arguments::new(args)))
                },
                description: vec![
                    "Compare two value and return true if the first is greater than the second.",
                ],
                examples: vec![
                    Example {
                        input: None,
                        arguments: vec!["1", "3"],
                        output: Some("false"),
                    },
                    Example {
                        input: None,
                        arguments: vec!["1", "1"],
                        output: Some("false"),
                    },
                    Example {
                        input: None,
                        arguments: vec!["31", "1"],
                        output: Some("true"),
                    },
                ],
            },
            FunctionDefinitions {
                name: "and",
                aliases: vec!["&&"],
                min_args_count: 2,
                max_args_count: usize::MAX,
                build_extractor: |args| {
                    struct Impl(Arguments);
                    impl Get for Impl {
                        fn get(&self, value: &Option<JsonValue>) -> Option<JsonValue> {
                            for s in &self.0.args {
                                match s.get(value) {
                                    Some(JsonValue::Boolean(true)) => {},
                                    Some(JsonValue::Boolean(false)) => {return Some(false.into());},
                                    _ => return None,
                                }
                            }
                            Some(true.into())
                        }
                    }
                    Box::new(Impl(Arguments::new(args)))
                },
                description: vec![
                    "Return true if all the arguments are true, nothing if there is a non boolean argument and false if there is a false argument.",
                ],
                examples: vec![
                    Example {
                        input: None,
                        arguments: vec!["true", "true", "true", "true"],
                        output: Some("true"),
                    },
                    Example {
                        input: None,
                        arguments: vec!["true", "true", "false", "true"],
                        output: Some("false"),
                    },
                    Example {
                        input: None,
                        arguments: vec!["true", "true", "12", "true"],
                        output: None,
                    },
                ],
            },
            FunctionDefinitions {
                name: "or",
                aliases: vec!["||"],
                min_args_count: 2,
                max_args_count: usize::MAX,
                build_extractor: |args| {
                    struct Impl(Arguments);
                    impl Get for Impl {
                        fn get(&self, value: &Option<JsonValue>) -> Option<JsonValue> {
                            for s in &self.0.args {
                                match s.get(value) {
                                    Some(JsonValue::Boolean(false)) => {},
                                    Some(JsonValue::Boolean(true)) => {return Some(true.into());},
                                    _ => return None,
                                }
                            }
                            Some(false.into())
                        }
                    }
                    Box::new(Impl(Arguments::new(args)))
                },
                description: vec![
                    "Return true if any of the arguments are true, nothing if there is a non boolean argument and false if all the arguments are false.",
                ],
                examples: vec![
                    Example {
                        input: None,
                        arguments: vec!["true", "true", "true", "true"],
                        output: Some("true"),
                    },
                    Example {
                        input: None,
                        arguments: vec!["false","false","false","true", "true", "false", "true"],
                        output: Some("true"),
                    },
                    Example {
                        input: None,
                        arguments: vec!["false", "false"],
                        output: Some("false"),
                    },
                    Example {
                        input: None,
                        arguments: vec!["121", "true", "true"],
                        output: None,
                    },
                ],
            },
            FunctionDefinitions {
                name: "xor",
                aliases: vec!["^"],
                min_args_count: 2,
                max_args_count: 2,
                build_extractor: |args| {
                    struct Impl(Arguments);
                    impl Get for Impl {
                        fn get(&self, value: &Option<JsonValue>) -> Option<JsonValue> {
                            if let (Some(JsonValue::Boolean(val1)), Some(JsonValue::Boolean(val2))) =
                                (self.0.apply(value, 0), self.0.apply(value, 1))
                            {
                                let eq = val1 ^ val2;
                                Some(eq.into())
                            } else {
                                None
                            }
                        }
                    }
                    Box::new(Impl(Arguments::new(args)))
                },
                description: vec![
                    "Return true if one, and only one, of the argument is true.",
                ],
                examples: vec![
                    Example {
                        input: None,
                        arguments: vec!["true", "true"],
                        output: Some("false"),
                    },
                    Example {
                        input: None,
                        arguments: vec!["true", "false"],
                        output: Some("true"),
                    },
                    Example {
                        input: None,
                        arguments: vec!["false", "true"],
                        output: Some("true"),
                    },
                    Example {
                        input: None,
                        arguments: vec!["false", "false"],
                        output: Some("false"),
                    },
                    Example {
                        input: None,
                        arguments: vec!["null", "false"],
                        output: None,
                    },
                    Example {
                        input: None,
                        arguments: vec!["true", "12"],
                        output: None,
                    },
                ],
            },
            FunctionDefinitions {
                name: "not",
                aliases: vec!["!"],
                min_args_count: 1,
                max_args_count: 1,
                build_extractor: |args| {
                    struct Impl(Arguments);
                    impl Get for Impl {
                        fn get(&self, value: &Option<JsonValue>) -> Option<JsonValue> {
                            if let Some(JsonValue::Boolean(val1)) =
                                self.0.apply(value, 0)
                            {
                                let eq = !val1;
                                Some(eq.into())
                            } else {
                                None
                            }
                        }
                    }
                    Box::new(Impl(Arguments::new(args)))
                },
                description: vec![
                    "Return false if the argument is true and true if the argument is false.",
                ],
                examples: vec![
                    Example {
                        input: None,
                        arguments: vec!["true"],
                        output: Some("false"),
                    },
                    Example {
                        input: None,
                        arguments: vec!["false"],
                        output: Some("true"),
                    },
                    Example {
                        input: None,
                        arguments: vec!["(string? 12)"],
                        output: Some("true"),
                    },
                    Example {
                        input: None,
                        arguments: vec!["12"],
                        output: None,
                    },
                ],
            },
        ],
    }
}
