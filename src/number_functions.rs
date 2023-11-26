use crate::{
    functions_definitions::{Arguments, Example, FunctionDefinitions, FunctionsGroup},
    json_value::JsonValue,
    selection::Get,
};

pub fn get_number_functions() -> FunctionsGroup {
    FunctionsGroup{ name: "Number functions", functions: vec![
        FunctionDefinitions {
            name: "+",
            aliases: vec!["add", "plus"],
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
                    output: Some("4"),
                },
                Example {
                    input: None,
                    arguments: vec!["1", "10", "-4.1", "0.1"],
                    output: Some("7"),
                },
                Example {
                    input: None,
                    arguments: vec!["1", "3", "false"],
                    output: None,
                },
            ],
        },
        FunctionDefinitions {
            name: "-",
            aliases: vec!["take_away", "minus", "substruct"],
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
                    output: Some("97"),
                },
                Example {
                    input: None,
                    arguments: vec!["10", "3.2"],
                    output: Some("6.8"),
                },
                Example {
                    input: None,
                    arguments: vec!["10", "\"text\""],
                    output: None,
                },
                Example {
                    input: None,
                    arguments: vec!["null", "6"],
                    output: None,
                },
            ],
        },
        FunctionDefinitions {
            name: "*",
            aliases: vec!["times", "multiple"],
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
                    output: Some("6"),
                },
                Example {
                    input: None,
                    arguments: vec!["2", "15", "0.1"],
                    output: Some("3"),
                },
                Example {
                    input: None,
                    arguments: vec!["2", "true"],
                    output: None,
                },
            ],
        },
        FunctionDefinitions {
            name: "/",
            aliases: vec!["divide"],
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
                    output: Some("4"),
                },
                Example {
                    input: None,
                    arguments: vec!["7", "2"],
                    output: Some("3.5"),
                },
                Example {
                    input: None,
                    arguments: vec!["7", "[]"],
                    output: None,
                },
                Example {
                    input: None,
                    arguments: vec!["{}", "5"],
                    output: None,
                },
            ],
        },
        FunctionDefinitions {
            name: "%",
            aliases: vec!["mod", "modulu"],
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
                    output: Some("2"),
                },
                Example {
                    input: None,
                    arguments: vec!["7", "2"],
                    output: Some("1"),
                },
                Example {
                    input: None,
                    arguments: vec!["7", "false"],
                    output: None,
                },
                Example {
                    input: None,
                    arguments: vec!["[1]", "4"],
                    output: None,
                },
            ],
        },

        ]}
}
