use indexmap::IndexMap;

use crate::{
    functions_definitions::{Arguments, Example, FunctionDefinitions, FunctionsGroup},
    json_value::JsonValue,
    selection::Get,
};

pub fn get_list_functions() -> FunctionsGroup {
    FunctionsGroup { name: "List functions", functions:  vec![
        FunctionDefinitions {
            name: "filter",
            aliases: vec![],
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
                    arguments: vec!["[1, 2, 3, 4]", "true"],
                    output: Some("[1, 2, 3, 4]")
                },
                Example {
                    input: None,
                    arguments: vec!["[1, 2, 3, 4]", "null"],
                    output: Some("[]")
                },
                Example {
                    input: None,
                    arguments: vec!["[1, 2, 3, 4, \"one\", null]", "(string? .)"],
                    output: Some("[\"one\"]")
                },
                Example {
                    input: Some("[1, 2, null, \"a\", 4]"),
                    arguments: vec![".", "(number? .)"],
                    output: Some("[1, 2, 4]")
                },
                Example {
                    input: None,
                    arguments: vec!["{}", "true"],
                    output: None,
                },
            ]
        },
        FunctionDefinitions {
            name: "sort",
            aliases: vec!["order"],
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
                    arguments: vec!["[1, -2, 3.01, 3.05, -544, 100]"],
                    output: Some("[-544, -2, 1, 3.01, 3.05, 100]")
                },
                Example {
                    input: None,
                    arguments: vec!["[null, true, false, {}, [1, 2, 3], \"abc\", \"cde\", {\"key\": 12}]"],
                    output: Some("[null, false, true, \"abc\", \"cde\", {}, {\"key\": 12}, [1, 2, 3]]")
                },
                Example {
                    input: None,
                    arguments: vec!["344"],
                    output: None,
                },
            ]
        },
        FunctionDefinitions {
            name: "group_by",
            aliases: vec![],
            min_args_count: 2,
            max_args_count: 2,
            build_extractor: |args| {
                struct Impl(Arguments);
                impl Get for Impl {
                    fn get(&self, value: &Option<JsonValue>) -> Option<JsonValue> {
                        match self.0.apply(value, 0) {
                            Some(JsonValue::Array(list)) => {
                                let mut groups = IndexMap::new();
                                for item in list {
                                    let value = Some(item.clone());
                                    let key = match self.0.apply(&value,1 ) {
                                        Some(JsonValue::String(str)) => str,
                                        _ => return None,
                                    };
                                    let values = groups.entry(key).or_insert_with(Vec::new);
                                    values.push(item);
                                }

                                Some(groups.iter().map(|(k, v)| {
                                    (k.clone(), Into::<JsonValue>::into(v.clone()))
                                }).collect::<IndexMap<_,_>>().into())
                            },
                            _ => None,
                        }
                    }
                }
                Box::new(Impl(Arguments::new(args)))
            },
            description: vec![
                "Group items by function.",
                "If the first argument is a list, return list grouped by the second argument."
            ],
            examples: vec![
                Example {
                    input: None,
                    arguments: vec!["[\"11\", \"5\", \"23\", \"ab\", \"1\", \"\", \"100\", {}]", "(stringify (len .))"],
                    output: Some(r#"{"2":["11","23","ab"],"1":["5","1"],"0":["",{}],"3":["100"]}"#),
                },
                Example {
                    input: None,
                    arguments: vec!["[{\"g\": \"one\", \"v\": 1}, {\"g\": \"two\", \"v\": 2}, {\"g\": \"one\", \"v\": 33}, {\"g\": \"two\", \"v\": false}]", ".g"],
                    output: Some(r#"{"one":[{"g":"one","v":1},{"g":"one","v":33}],"two":[{"g":"two","v":2},{"g":"two","v":false}]}"#),
                },
                Example {
                    input: None,
                    arguments: vec!["344", "(stringify (len .))"],
                    output: None,
                },
                Example {
                    input: None,
                    arguments: vec!["[\"11\", \"5\", \"23\", \"ab\", \"1\", \"\", \"100\", {}]", "(len .)"],
                    output: None,
                },
            ]
        },
        FunctionDefinitions {
            name: "sort_by",
            aliases: vec!["order_by"],
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
                    arguments: vec!["[\"12345\", \"5\", \"23\", \"abc\", \"-1-2\", \"\"]", "(len .)"],
                    output: Some(r#"["","5","23","abc","-1-2","12345"]"#),
                },
                Example {
                    input: None,
                    arguments: vec!["true", "(len .)"],
                    output: None,
                },
            ]
        },
        FunctionDefinitions {
            name: "sum",
            aliases: vec![],
            min_args_count: 1,
            max_args_count: 1,
            build_extractor: |args| {
                struct Impl(Arguments);
                impl Get for Impl {
                    fn get(&self, value: &Option<JsonValue>) -> Option<JsonValue> {
                        match self.0.apply(value, 0) {
                            Some(JsonValue::Array(list)) => {
                                let mut sum = 0.0;
                                for t in list {
                                    let t: Result<f64, _> = t.try_into();
                                    match t {
                                        Ok(num) => sum += num,
                                        Err(_) => return None,
                                    }
                                }
                                Some(sum.into())
                            },
                            _ => None,
                        }
                    }
                }
                Box::new(Impl(Arguments::new(args)))
            },
            description: vec![
                "Sum all the items in the list.",
                "If list have non numeric items, it will return nuthing."
            ],
            examples: vec![
                Example {
                    input: None,
                    arguments: vec!["[1, 5, 1.1]"],
                    output: Some("7.1"),
                },
                Example {
                    input: None,
                    arguments: vec!["[]"],
                    output: Some("0"),
                },
                Example {
                    input: None,
                    arguments: vec!["[1, 5, 1.1, \"text\"]"],
                    output: None,
                },
            ]
        },
    ]}
}
