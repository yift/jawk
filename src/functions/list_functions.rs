use indexmap::IndexMap;

use crate::{
    functions_definitions::{Arguments, Example, FunctionDefinitions, FunctionsGroup},
    json_value::JsonValue,
    selection::Get,
};

pub fn get_list_functions() -> FunctionsGroup {
    FunctionsGroup::new("List functions")

        .add_function(
            FunctionDefinitions::new("filter", 2, 2, |args| {
                struct Impl(Vec<Box<dyn Get>>);
                impl Get for Impl {
                    fn get(&self, value: &Option<JsonValue>) -> Option<JsonValue> {
                        match self.0.apply(value, 0) {
                            Some(JsonValue::Array(list)) => {
                                let list: Vec<_> = list
                                    .into_iter()

                                    .filter(|v| {
                                        let v = Some(v.clone());
                                        matches!(
                                            self.0.apply(&v, 1),
                                            Some(JsonValue::Boolean(true))
                                        )
                                    })
                                    .collect();
                                Some(list.into())
                            }
                            _ => None,
                        }
                    }
                }
                Box::new(Impl(args))
            })
                .add_description_line("Filter a list.")
                .add_description_line(
                    "If the first argument is a list, return all the values for which the second argument is a list."
                )
                .add_example(
                    Example::new()
                        .add_argument("[1, 2, 3, 4]")
                        .add_argument("true")
                        .expected_output("[1, 2, 3, 4]")
                )
                .add_example(
                    Example::new()
                        .add_argument("[1, 2, 3, 4]")
                        .add_argument("null")
                        .expected_output("[]")
                )
                .add_example(
                    Example::new()
                        .add_argument("[1, 2, 3, 4, \"one\", null]")
                        .add_argument("(string? .)")
                        .expected_output("[\"one\"]")
                )
                .add_example(
                    Example::new()
                        .add_argument(".")
                        .add_argument("(number? .)")
                        .expected_output("[1, 2, 4]")
                        .input("[1, 2, null, \"a\", 4]")
                )
                .add_example(Example::new().add_argument("{}").add_argument("true"))
        )

        .add_function(
            FunctionDefinitions::new("sort", 1, 1, |args| {
                struct Impl(Vec<Box<dyn Get>>);
                impl Get for Impl {
                    fn get(&self, value: &Option<JsonValue>) -> Option<JsonValue> {
                        match self.0.apply(value, 0) {
                            Some(JsonValue::Array(list)) => {
                                let mut list: Vec<JsonValue> = list.clone();
                                list.sort();

                                Some(list.into())
                            }
                            _ => None,
                        }
                    }
                }
                Box::new(Impl(args))
            })
                .add_alias("order")
                .add_description_line("Sort a list.")
                .add_description_line("If the first argument is a list, return list sorted.")
                .add_example(
                    Example::new()
                        .add_argument("[1, -2, 3.01, 3.05, -544, 100]")
                        .expected_output("[-544, -2, 1, 3.01, 3.05, 100]")
                )
                .add_example(
                    Example::new()
                        .add_argument(
                            "[null, true, false, {}, [1, 2, 3], \"abc\", \"cde\", {\"key\": 12}]"
                        )
                        .expected_output(
                            "[null, false, true, \"abc\", \"cde\", {}, {\"key\": 12}, [1, 2, 3]]"
                        )
                )
                .add_example(Example::new().add_argument("344"))
        )

        .add_function(
            FunctionDefinitions::new("group_by", 2, 2, |args| {
                struct Impl(Vec<Box<dyn Get>>);
                impl Get for Impl {
                    fn get(&self, value: &Option<JsonValue>) -> Option<JsonValue> {
                        match self.0.apply(value, 0) {
                            Some(JsonValue::Array(list)) => {
                                let mut groups = IndexMap::new();
                                for item in list {
                                    let value = Some(item.clone());
                                    let key = match self.0.apply(&value, 1) {
                                        Some(JsonValue::String(str)) => str,
                                        _ => {
                                            return None;
                                        }
                                    };
                                    let values = groups.entry(key).or_insert_with(Vec::new);
                                    values.push(item);
                                }

                                Some(
                                    groups
                                        .iter()
                                        .map(|(k, v)| {
                                            (k.clone(), Into::<JsonValue>::into(v.clone()))
                                        })
                                        .collect::<IndexMap<_, _>>()
                                        .into()
                                )
                            }
                            _ => None,
                        }
                    }
                }
                Box::new(Impl(args))
            })
                .add_description_line("Group items by function.")
                .add_description_line(
                    "If the first argument is a list, return list grouped by the second argument."
                )
                .add_example(
                    Example::new()
                        .add_argument("[\"11\", \"5\", \"23\", \"ab\", \"1\", \"\", \"100\", {}]")
                        .add_argument("(stringify (len .))")
                        .expected_output(
                            "{\"2\":[\"11\",\"23\",\"ab\"],\"1\":[\"5\",\"1\"],\"0\":[\"\",{}],\"3\":[\"100\"]}"
                        )
                )
                .add_example(
                    Example::new()
                        .add_argument(
                            "[{\"g\": \"one\", \"v\": 1}, {\"g\": \"two\", \"v\": 2}, {\"g\": \"one\", \"v\": 33}, {\"g\": \"two\", \"v\": false}]"
                        )
                        .add_argument(".g")
                        .expected_output(
                            "{\"one\":[{\"g\":\"one\",\"v\":1},{\"g\":\"one\",\"v\":33}],\"two\":[{\"g\":\"two\",\"v\":2},{\"g\":\"two\",\"v\":false}]}"
                        )
                )
                .add_example(Example::new().add_argument("344").add_argument("(stringify (len .))"))
                .add_example(
                    Example::new()
                        .add_argument("[\"11\", \"5\", \"23\", \"ab\", \"1\", \"\", \"100\", {}]")
                        .add_argument("(len .)")
                )
        )

        .add_function(
            FunctionDefinitions::new("sort_by", 2, 2, |args| {
                struct Impl(Vec<Box<dyn Get>>);
                impl Get for Impl {
                    fn get(&self, value: &Option<JsonValue>) -> Option<JsonValue> {
                        match self.0.apply(value, 0) {
                            Some(JsonValue::Array(list)) => {
                                let mut list: Vec<JsonValue> = list.clone();
                                list.sort_by(|v1, v2| {
                                    let v1 = Some(v1.clone());
                                    let v1 = self.0.apply(&v1, 1);
                                    let v2 = Some(v2.clone());
                                    let v2 = self.0.apply(&v2, 1);
                                    v1.cmp(&v2)
                                });

                                Some(list.into())
                            }
                            _ => None,
                        }
                    }
                }
                Box::new(Impl(args))
            })
                .add_alias("order_by")
                .add_description_line("Filter a list.")
                .add_description_line(
                    "If the first argument is a list, return list sorted by the second argument."
                )
                .add_example(
                    Example::new()
                        .add_argument("[\"12345\", \"5\", \"23\", \"abc\", \"-1-2\", \"\"]")
                        .add_argument("(len .)")
                        .expected_output("[\"\",\"5\",\"23\",\"abc\",\"-1-2\",\"12345\"]")
                )
                .add_example(Example::new().add_argument("true").add_argument("(len .)"))
        )

        .add_function(
            FunctionDefinitions::new("sum", 1, 1, |args| {
                struct Impl(Vec<Box<dyn Get>>);
                impl Get for Impl {
                    fn get(&self, value: &Option<JsonValue>) -> Option<JsonValue> {
                        match self.0.apply(value, 0) {
                            Some(JsonValue::Array(list)) => {
                                let mut sum = 0.0;
                                for t in list {
                                    let t: Result<f64, _> = t.try_into();
                                    match t {
                                        Ok(num) => {
                                            sum += num;
                                        }
                                        Err(_) => {
                                            return None;
                                        }
                                    }
                                }
                                Some(sum.into())
                            }
                            _ => None,
                        }
                    }
                }
                Box::new(Impl(args))
            })
                .add_description_line("Sum all the items in the list.")
                .add_description_line("If list have non numeric items, it will return nuthing.")
                .add_example(Example::new().add_argument("[1, 5, 1.1]").expected_output("7.1"))
                .add_example(Example::new().add_argument("[]").expected_output("0"))
                .add_example(Example::new().add_argument("[1, 5, 1.1, \"text\"]"))
        )

        .add_function(
            FunctionDefinitions::new("join", 1, 2, |args| {
                struct Impl(Vec<Box<dyn Get>>);
                impl Get for Impl {
                    fn get(&self, value: &Option<JsonValue>) -> Option<JsonValue> {
                        let sepetator = self.0
                            .apply(value, 1)
                            .and_then(|f| TryInto::<String>::try_into(f).ok())
                            .unwrap_or(", ".into());
                        match self.0.apply(value, 0) {
                            Some(JsonValue::Array(list)) => {
                                let mut str = String::new();
                                for t in list {
                                    let t: Result<String, _> = t.try_into();
                                    match t {
                                        Ok(to_add) => {
                                            if !str.is_empty() {
                                                str.push_str(sepetator.as_str());
                                            }
                                            str.push_str(to_add.as_str());
                                        }
                                        Err(_) => {
                                            return None;
                                        }
                                    }
                                }
                                Some(str.into())
                            }
                            _ => None,
                        }
                    }
                }
                Box::new(Impl(args))
            })
                .add_description_line("Join all the items in the list into a String.")
                .add_description_line("If list have non string items, it will return nuthing.")
                .add_description_line(
                    "If the second argument is ommited, the items will be seperated by comma."
                )
                .add_example(
                    Example::new()
                        .add_argument("[\"one\", \"two\", \"three\"]")
                        .expected_output("\"one, two, three\"")
                )
                .add_example(
                    Example::new()
                        .add_argument("[\"one\", \"two\", \"three\"]")
                        .add_argument("\" ; \"")
                        .expected_output("\"one ; two ; three\"")
                )
                .add_example(Example::new().add_argument("[\"one\", \"two\", 3]"))
        )

        .add_function(
            FunctionDefinitions::new("first", 1, 1, |args| {
                struct Impl(Vec<Box<dyn Get>>);
                impl Get for Impl {
                    fn get(&self, value: &Option<JsonValue>) -> Option<JsonValue> {
                        match self.0.apply(value, 0) {
                            Some(JsonValue::Array(list)) => { list.first().cloned() }
                            _ => None,
                        }
                    }
                }
                Box::new(Impl(args))
            })
                .add_description_line("The first item in a list.")
                .add_example(Example::new().add_argument("[1, 5, 1.1]").expected_output("1"))
                .add_example(Example::new().add_argument("[]"))
                .add_example(Example::new().add_argument("[\"text\"]").expected_output("\"text\""))
                .add_example(Example::new().add_argument("\"text\""))
        )

        .add_function(
            FunctionDefinitions::new("last", 1, 1, |args| {
                struct Impl(Vec<Box<dyn Get>>);
                impl Get for Impl {
                    fn get(&self, value: &Option<JsonValue>) -> Option<JsonValue> {
                        match self.0.apply(value, 0) {
                            Some(JsonValue::Array(list)) => { list.last().cloned() }
                            _ => None,
                        }
                    }
                }
                Box::new(Impl(args))
            })
                .add_description_line("The last item in a list.")
                .add_example(Example::new().add_argument("[1, 5, 1.1]").expected_output("1.1"))
                .add_example(Example::new().add_argument("[]"))
                .add_example(Example::new().add_argument("[\"text\"]").expected_output("\"text\""))
                .add_example(Example::new().add_argument("\"text\""))
        )
        .add_function(
            FunctionDefinitions::new("map", 2, 2, |args| {
                struct Impl(Vec<Box<dyn Get>>);
                impl Get for Impl {
                    fn get(&self, value: &Option<JsonValue>) -> Option<JsonValue> {
                        match self.0.apply(value, 0) {
                            Some(JsonValue::Array(list)) => {
                                let list: Vec<_> = list
                                    .into_iter()
                                    .filter_map(|v| {
                                        let v = Some(v.clone());
                                        self.0.apply(&v, 1)
                                    })
                                    .collect();

                                Some(list.into())
                            }
                            _ => None,
                        }
                    }
                }
                Box::new(Impl(args))
            })
                .add_description_line("Map a list into a new list using a function.")
                .add_description_line(
                    "If the first argument is a list, activate the second argument on each item and collect into a new list."
                )
                .add_example(
                    Example::new()
                        .add_argument("[1, 2, 3, 4]")
                        .add_argument("(+ . 4)")
                        .expected_output("[5, 6, 7, 8]")
                )
                .add_example(
                    Example::new()
                        .add_argument("[1, 2, 3, 4]")
                        .add_argument("(.len)")
                        .expected_output("[]")
                )
                .add_example(
                    Example::new()
                        .add_argument("[1, 2, 3, \"4\"]")
                        .add_argument("(* . 2)")
                        .expected_output("[2, 4, 6]")
                )
                .add_example(
                    Example::new()
                        .add_argument(".")
                        .add_argument("(+ 2 .)")
                        .expected_output("[3, 4, 6]")
                        .input("[1, 2, null, \"a\", 4]")
                )
                .add_example(Example::new().add_argument("{}").add_argument("true"))
        )
        .add_function(
            FunctionDefinitions::new("flat_map", 2, 2, |args| {
                struct Impl(Vec<Box<dyn Get>>);
                impl Get for Impl {
                    fn get(&self, value: &Option<JsonValue>) -> Option<JsonValue> {
                        match self.0.apply(value, 0) {
                            Some(JsonValue::Array(list)) => {
                                let list: Vec<_> = list
                                    .into_iter()
                                    .filter_map(|v| {
                                        let v = Some(v.clone());
                                        if let Some(JsonValue::Array(list)) = self.0.apply(&v, 1) {
                                            Some(list)
                                        } else {
                                            None
                                        }
                                    })
                                    .flatten()
                                    .collect();

                                Some(list.into())
                            }
                            _ => None,
                        }
                    }
                }
                Box::new(Impl(args))
            })
                .add_description_line("Flat map a list into a new list using a function.")
                .add_description_line(
                    "If the first argument is a list, activate the second argument on each item, and if that returns a list, add all the items to a new list."
                )
                .add_example(
                    Example::new()
                        .add_argument("[\"a,b,c\", \"d,e\", 4, \"g\"]")
                        .add_argument("(split . \",\")")
                        .expected_output("[\"a\", \"b\", \"c\", \"d\", \"e\", \"g\"]")
                )
                .add_example(
                    Example::new()
                        .add_argument("[1, 2, 3, 4]")
                        .add_argument("(.len)")
                        .expected_output("[]")
                )
                .add_example(Example::new().add_argument("{}").add_argument("true"))
        )
        .add_function(
            FunctionDefinitions::new("range", 1, 1, |args| {
                struct Impl(Vec<Box<dyn Get>>);
                impl Get for Impl {
                    fn get(&self, value: &Option<JsonValue>) -> Option<JsonValue> {
                        match self.0.apply(value, 0) {
                            Some(JsonValue::Number(n)) => {
                                if let Ok(size) = TryInto::<usize>::try_into(n) {
                                    let mut vec = vec![];
                                    for i in 0..size {
                                        vec.push(i.into());
                                    }
                                    Some(vec.into())
                                } else {
                                    None
                                }
                            }
                            _ => None,
                        }
                    }
                }
                Box::new(Impl(args))
            })
                .add_description_line("Create a new list with items from 0 to the second argument.")
                .add_description_line(
                    "If the second argument is not a positive integer, return nothing."
                )
                .add_example(Example::new().add_argument("4").expected_output("[0, 1, 2, 3]"))
                .add_example(Example::new().add_argument("-4"))
                .add_example(Example::new().add_argument("[1, 2, 3, 4]"))
        )
}
