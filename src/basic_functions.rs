use crate::{
    functions_definitions::{Arguments, Example, FunctionDefinitions, FunctionsGroup},
    json_value::JsonValue,
    selection::Get,
};

use indexmap::IndexMap;
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
                    arguments: vec!["[\"a\", \"b\", \"c\"]", "1"],
                    output: Some("\"b\""),
                },
                Example {
                    input: None,
                    arguments: vec!["{\"key-1\": 12, \"key-2\": 32}", "\"key-1\""],
                    output: Some("12"),
                },
                Example {
                    input: None,
                    arguments: vec!["[\"a\", \"b\", \"c\"]", "100"],
                    output: None,
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
                arguments: vec!["(get . \"key\")", "(get . 3)", "(get . \"key-2\")"],
                output: Some("100"),
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
                            Some(JsonValue::Array(list)) => {
                                Some(list.len().into())
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
                    arguments: vec!["[1, 2, 3, 4]"],
                    output: Some("4"),
                },
                Example {
                    input: None,
                    arguments: vec!["{\"key-1\": 1, \"key-2\": false}"],
                    output: Some("2"),
                },
                Example {
                    input: None,
                    arguments: vec!["\"123\""],
                    output: Some("3"),
                },
                Example {
                    input: None,
                    arguments: vec!["50"],
                    output: None,
                },
            ]
        },
        FunctionDefinitions {
            name: "take",
            aliases: vec!["take_first"],
            min_args_count: 2,
            max_args_count: 2,
            build_extractor: |args| {
                struct Impl(Arguments);
                impl Get for Impl {
                    fn get(&self, value: &Option<JsonValue>) -> Option<JsonValue> {
                        if let Some(JsonValue::Number(n)) = self.0.apply(value, 1) {
                            if let Ok(size) = TryInto::<usize>::try_into(n) {
                                match self.0.apply(value, 0) {
                                    Some(JsonValue::Object(map)) => {
                                        let map = if size > map.len() {
                                            map
                                        } else {
                                            let mut new_map = IndexMap::with_capacity(size);
                                            for (k, v) in map {
                                                new_map.insert(k, v);
                                                if new_map.len() == size {
                                                    break
                                                }
                                            }
                                            new_map
                                        };
                                        Some(map.into())
                                    },
                                    Some(JsonValue::Array(vec)) => {
                                        let vec = if size > vec.len() {
                                            vec
                                        } else {
                                            let mut new_vec = Vec::with_capacity(size);
                                            for i in vec {
                                                new_vec.push(i);
                                                if new_vec.len() == size {
                                                    break
                                                }
                                            }
                                            new_vec
                                        };
                                        Some(vec.into())
                                    },
                                    Some(JsonValue::String(str)) => {
                                        let str = if size > str.len() {
                                            str
                                        } else {
                                            str[..size].into()
                                        };
                                        Some(str.into())
                                    },
                                     _ => None
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
            description: vec![
                "Take the first N of element in an array, object of string",
            ],
            examples: vec![
                Example {
                    input: None,
                    arguments: vec!["[1, 2, 3, 4]", "2"],
                    output: Some("[1, 2]"),
                },
                Example {
                    input: None,
                    arguments: vec!["[1, 2, 3, 4]", "6"],
                    output: Some("[1, 2, 3, 4]"),
                },
                Example {
                    input: None,
                    arguments: vec!["{\"key-1\": 1, \"key-2\": false}", "1"],
                    output: Some("{\"key-1\": 1}"),
                },
                Example {
                    input: None,
                    arguments: vec!["{\"key-1\": 1, \"key-2\": false}", "3"],
                    output: Some("{\"key-1\": 1, \"key-2\": false}"),
                },
                Example {
                    input: None,
                    arguments: vec!["\"123\"", "2"],
                    output: Some("\"12\""),
                },
                Example {
                    input: None,
                    arguments: vec!["\"123\"", "20"],
                    output: Some("\"123\""),
                },
                Example {
                    input: None,
                    arguments: vec!["50", "10"],
                    output: None,
                },
                Example {
                    input: None,
                    arguments: vec!["\"123\"", "false"],
                    output: None,
                },
            ]
        },
        FunctionDefinitions {
            name: "take_last",
            aliases: vec![],
            min_args_count: 2,
            max_args_count: 2,
            build_extractor: |args| {
                struct Impl(Arguments);
                impl Get for Impl {
                    fn get(&self, value: &Option<JsonValue>) -> Option<JsonValue> {
                        if let Some(JsonValue::Number(n)) = self.0.apply(value, 1) {
                            if let Ok(size) = TryInto::<usize>::try_into(n) {
                                match self.0.apply(value, 0) {
                                    Some(JsonValue::Object(map)) => {
                                        let map = if size > map.len() {
                                            map
                                        } else {
                                            let mut new_map = IndexMap::with_capacity(size);
                                            let mut index = map.len();
                                            for (k, v) in map {
                                                index-=1;
                                                if index < size {
                                                    new_map.insert(k, v);
                                                }
                                            }
                                            new_map
                                        };
                                        Some(map.into())
                                    },
                                    Some(JsonValue::Array(vec)) => {
                                        let vec = if size > vec.len() {
                                            vec
                                        } else {
                                            let mut new_vec = Vec::with_capacity(size);
                                            let mut index = vec.len();
                                            for i in vec {
                                                index-=1;
                                                if index < size {
                                                    new_vec.push(i);
                                                }
                                            }
                                            new_vec
                                        };
                                        Some(vec.into())
                                    },
                                    Some(JsonValue::String(str)) => {
                                        let str = if size > str.len() {
                                            str
                                        } else {
                                            str[(size-1)..].into()
                                        };
                                        Some(str.into())
                                    },
                                     _ => None
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
            description: vec![
                "Take the last N of element in an array, object of string",
            ],
            examples: vec![
                Example {
                    input: None,
                    arguments: vec!["[1, 2, 3, 4]", "2"],
                    output: Some("[3, 4]"),
                },
                Example {
                    input: None,
                    arguments: vec!["[1, 2, 3, 4]", "6"],
                    output: Some("[1, 2, 3, 4]"),
                },
                Example {
                    input: None,
                    arguments: vec!["{\"key-1\": 1, \"key-2\": false}", "1"],
                    output: Some("{\"key-2\": false}"),
                },
                Example {
                    input: None,
                    arguments: vec!["{\"key-1\": 1, \"key-2\": false}", "3"],
                    output: Some("{\"key-1\": 1, \"key-2\": false}"),
                },
                Example {
                    input: None,
                    arguments: vec!["\"123\"", "2"],
                    output: Some("\"23\""),
                },
                Example {
                    input: None,
                    arguments: vec!["\"123\"", "20"],
                    output: Some("\"123\""),
                },
                Example {
                    input: None,
                    arguments: vec!["50", "10"],
                    output: None,
                },
                Example {
                    input: None,
                    arguments: vec!["\"123\"", "false"],
                    output: None,
                },
            ]
        },
        FunctionDefinitions {
            name: "sub",
            aliases: vec![],
            min_args_count: 3,
            max_args_count: 3,
            build_extractor: |args| {
                struct Impl(Arguments);
                impl Get for Impl {
                    fn get(&self, value: &Option<JsonValue>) -> Option<JsonValue> {
                        let start = if let Some(JsonValue::Number(n)) = self.0.apply(value, 1) {
                            if let Ok(start) = TryInto::<usize>::try_into(n) {
                                start
                            } else {
                                return None;
                            }
                        } else {
                            return None;
                        };
                        let length = if let Some(JsonValue::Number(n)) = self.0.apply(value, 2) {
                            if let Ok(start) = TryInto::<usize>::try_into(n) {
                                start
                            } else {
                                return None;
                            }
                        } else {
                            return None;
                        };
                        match self.0.apply(value, 0) {
                            Some(JsonValue::Object(map)) => {
                                let mut new_map = IndexMap::with_capacity(length);
                                for (index, (k, v)) in map.into_iter().enumerate() {
                                    if new_map.len() == length {
                                        break;
                                    }
                                    if index >= start {
                                        new_map.insert(k, v);
                                    }
                                }
                                Some(new_map.into())
                            },
                            Some(JsonValue::Array(vec)) => {
                                let mut new_vec = Vec::with_capacity(length);
                                for (index, i) in vec.into_iter().enumerate() {
                                    if new_vec.len() == length {
                                        break;
                                    }
                                    if index >= start {
                                        new_vec.push(i);
                                    }
                                }
                                Some(new_vec.into())
                            },
                            Some(JsonValue::String(str)) => {
                                if (start >= str.len()) || (length == 0) {
                                    Some("".to_string().into())
                                } else {
                                    let last_index = start + length;
                                    let last_index = if last_index >= str.len() {
                                        str.len()
                                    } else {
                                        last_index
                                    };
                                    let str = str[start..last_index].to_string();
                                    Some(str.into())
                                }
                            },
                             _ => None
                        }
                    }
                }
                Box::new(Impl(Arguments::new(args)))
            },
            description: vec![
                "If the first argument is a list, creates a new list that start from the second arguments and has the size of the third argument.",
                "If the first argument is an object, creates a new object that start from the second arguments and has the size of the third argument.",
                "If the first argument is a string, creates a substring that start from the second arguments and has the size of the third argument.",
            ],
            examples: vec![
                Example {
                    input: None,
                    arguments: vec!["[1, 2, 3, 4, 5, 6]", "2", "3"],
                    output: Some("[3, 4, 5]"),
                },
                Example {
                    input: None,
                    arguments: vec!["[1, 2, 3, 4]", "6", "10"],
                    output: Some("[]"),
                },
                Example {
                    input: None,
                    arguments: vec!["[1, 2, 3, 4]", "0", "10"],
                    output: Some("[1, 2, 3, 4]"),
                },
                Example {
                    input: None,
                    arguments: vec!["[1, 2, 3, 4]", "1", "10"],
                    output: Some("[2, 3, 4]"),
                },
                Example {
                    input: None,
                    arguments: vec!["{\"key-1\": 1, \"key-2\": 2, \"key-3\": 3, \"key-4\": 4, \"key-5\": 5, \"key-6\": 6}", "1", "2"],
                    output: Some("{\"key-2\": 2, \"key-3\": 3}"),
                },
                Example {
                    input: None,
                    arguments: vec!["{\"key-1\": 1, \"key-2\": 2, \"key-3\": 3, \"key-4\": 4, \"key-5\": 5, \"key-6\": 6}", "0", "2"],
                    output: Some("{\"key-1\": 1, \"key-2\": 2}"),
                },
                Example {
                    input: None,
                    arguments: vec!["{\"key-1\": 1, \"key-2\": 2, \"key-3\": 3, \"key-4\": 4, \"key-5\": 5, \"key-6\": 6}", "4", "10"],
                    output: Some("{\"key-5\": 5, \"key-6\": 6}"),
                },
                Example {
                    input: None,
                    arguments: vec!["{\"key-1\": 1, \"key-2\": 2, \"key-3\": 3, \"key-4\": 4, \"key-5\": 5, \"key-6\": 6}", "20", "10"],
                    output: Some("{}"),
                },
                Example {
                    input: None,
                    arguments: vec!["\"123456\"", "1", "3"],
                    output: Some("\"234\""),
                },
                Example {
                    input: None,
                    arguments: vec!["\"123456\"", "1", "30"],
                    output: Some("\"23456\""),
                },
                Example {
                    input: None,
                    arguments: vec!["\"123456\"", "0", "30"],
                    output: Some("\"123456\""),
                },
                Example {
                    input: None,
                    arguments: vec!["\"123456\"", "20", "30"],
                    output: Some("\"\""),
                },
                Example {
                    input: None,
                    arguments: vec!["\"123456\"", "2", "0"],
                    output: Some("\"\""),
                },
                Example {
                    input: None,
                    arguments: vec!["50", "0", "10"],
                    output: None,
                },
                Example {
                    input: None,
                    arguments: vec!["\"123\"", "false", "10"],
                    output: None,
                },
                Example {
                    input: None,
                    arguments: vec!["\"123\"", "10", "{}"],
                    output: None,
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
                    arguments: vec!["(get . 1)", "(get . \"key-1\")", "22"],
                    output: Some("1")
                },
                Example {
                    input: Some("{\"key-1\": 1, \"key-2\": false}"),
                    arguments: vec!["(get . 1)", "(get . \"key-3\")", "22"],
                    output: Some("22")
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
                    arguments: vec!["true", "12", "22"],
                    output: Some("12")
                },
                Example {
                    input: None,
                    arguments: vec!["false", "12", "22"],
                    output: Some("22")
                },
                Example {
                    input: Some("[1, 2, 3]"),
                    arguments: vec!["(array? .)", "#1", "#2"],
                    output: Some("2")
                },
                Example {
                    input: Some("[1, 2, 3]"),
                    arguments: vec!["(null? .)", "#1", "#2"],
                    output: Some("3")
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
                    arguments: vec!["true"],
                    output: Some("\"true\"")
                },
                Example {
                    input: None,
                    arguments: vec!["1e2"],
                    output: Some("\"100\"")
                },
                Example {
                    input: None,
                    arguments: vec!["{\"key\": [1, 2, \"3\"]}"],
                    output: Some("\"{\\\"key\\\": [1, 2, \\\"3\\\"]}\"")
                },
            ]
        },
    ]}
}
