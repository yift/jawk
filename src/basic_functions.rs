use crate::{
    functions_definitions::{Arguments, Example, FunctionDefinitions, FunctionsGroup},
    json_value::JsonValue,
    selection::Get,
};

pub fn get_basic_functions() -> FunctionsGroup {
    FunctionsGroup{ name: "Basic functions", functions: vec![
        FunctionDefinitions {
            name: "get",
            aliases: vec!["[]"],
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
                                if let Some(JsonValue::Number(n)) = self.0.apply(value, 1) {
                                    if let Ok(index) = TryInto::<usize>::try_into(n) {
                                        array.get(index).cloned()
                                    } else {
                                        None
                                    }
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
            name: "|",
            aliases: vec![],
            min_args_count: 2,
            max_args_count: usize::MAX,
            build_extractor: |args| {
                struct Impl(Arguments);
                impl Get for Impl {
                    fn get(&self, value: &Option<JsonValue>) -> Option<JsonValue> {
                        let mut value = value.clone();
                        for e in &self.0.args {
                            value = e.get(&value);
                        }
                        value
                    }
                }
                Box::new(Impl(Arguments::new(args)))
            },
            description: vec!["Pipe the output of one function to the next function."],
            examples: vec![Example {
                input: Some("{\"key\": [20, 40, 60, {\"key-2\": 100}]}"),
                arguments: vec!["(get . \"key\")", "(get . 3)", "(get . \"key-2\")"]
            },]
        },
        FunctionDefinitions {
            name: "size",
            aliases: vec!["count", "length", "len"],
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
            name: "default",
            aliases: vec!["defaults", "or_else"],
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
            name: "?",
            aliases: vec!["if"],
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
        FunctionDefinitions {
            name: "stringify",
            aliases: vec![],
            min_args_count: 1,
            max_args_count: 1,
            build_extractor: |args| {
                struct Impl(Arguments);
                impl Get for Impl {
                    fn get(&self, value: &Option<JsonValue>) -> Option<JsonValue> {
                        self.0.apply(value, 0).map(|val| format!("{}", val).into())
                    }
                }
                Box::new(Impl(Arguments::new(args)))
            },
            description: vec![
                "Return the JSON represantation of the object."
            ],
            examples: vec![
                Example {
                    input: None,
                    arguments: vec!["true"]
                },
                Example {
                    input: None,
                    arguments: vec!["1e2"]
                },
                Example {
                    input: None,
                    arguments: vec!["{\"key\": [1, 2, \"3\"]}"]
                },
            ]
        },
    ]}
}
