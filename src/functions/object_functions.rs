use std::rc::Rc;

use indexmap::IndexMap;

use crate::{
    functions_definitions::{Arguments, Example, FunctionDefinitions, FunctionsGroup},
    json_value::JsonValue,
    processor::Context,
    selection::Get,
};

pub fn get_object_functions() -> FunctionsGroup {
    FunctionsGroup::new("object")

        .add_function(
            FunctionDefinitions::new("keys", 1, 1, |args| {
                struct Impl(Vec<Rc<dyn Get>>);
                impl Get for Impl {
                    fn get(&self, value: &Context) -> Option<JsonValue> {
                        if let Some(JsonValue::Object(map)) = self.0.apply(value, 0) {
                            Some(
                                JsonValue::Array(
                                    map.keys().cloned().map(JsonValue::String).collect()
                                )
                            )
                        } else {
                            None
                        }
                    }
                }
                Rc::new(Impl(args))
            })
                .add_description_line("Get the list of keys from an object.")
                .add_example(
                    Example::new()
                        .add_argument("{\"key-1\": 1, \"key-2\": false}")
                        .expected_output("[\"key-1\",\"key-2\"]")
                )
                .add_example(Example::new().add_argument("[1, 2, 4]"))
        )

        .add_function(
            FunctionDefinitions::new("values", 1, 1, |args| {
                struct Impl(Vec<Rc<dyn Get>>);
                impl Get for Impl {
                    fn get(&self, value: &Context) -> Option<JsonValue> {
                        if let Some(JsonValue::Object(map)) = self.0.apply(value, 0) {
                            Some(JsonValue::Array(map.values().cloned().collect()))
                        } else {
                            None
                        }
                    }
                }
                Rc::new(Impl(args))
            })
                .add_alias("vals")
                .add_description_line("Get the list of values from an object.")
                .add_example(
                    Example::new()
                        .add_argument("{\"key-1\": 1, \"key-2\": false}")
                        .expected_output("[1, false]")
                )
                .add_example(Example::new().add_argument("[1, 2, 4]"))
        )

        .add_function(
            FunctionDefinitions::new("entries", 1, 1, |args| {
                struct Impl(Vec<Rc<dyn Get>>);
                impl Get for Impl {
                    fn get(&self, value: &Context) -> Option<JsonValue> {
                        if let Some(JsonValue::Object(map)) = self.0.apply(value, 0) {
                            let mut list = Vec::with_capacity(map.len());
                            for (k, v) in map {
                                let mut data = IndexMap::with_capacity(2);
                                data.insert("value".to_string(), v.clone());
                                data.insert("key".to_string(), k.into());
                                list.push(data.into());
                            }
                            Some(JsonValue::Array(list))
                        } else {
                            None
                        }
                    }
                }
                Rc::new(Impl(args))
            })
                .add_alias("to_list")
                .add_description_line(
                    "Get the list of all the entries of an obejct. Each item of the list will be an object with `key` and `value` entries"
                )
                .add_example(
                    Example::new()
                        .add_argument(r#"{"key-1": 1, "key-2": false}"#)
                        .expected_output(
                            r#"[{"key": "key-1", "value": 1}, {"key": "key-2", "value": false}]"#
                        )
                )
                .add_example(Example::new().add_argument("[1, 2, 4]"))
        )

        .add_function(
            FunctionDefinitions::new("sort_by_keys", 1, 1, |args| {
                struct Impl(Vec<Rc<dyn Get>>);
                impl Get for Impl {
                    fn get(&self, value: &Context) -> Option<JsonValue> {
                        match self.0.apply(value, 0) {
                            Some(JsonValue::Object(map)) => {
                                let mut map = map.clone();
                                map.sort_keys();

                                Some(map.into())
                            }
                            _ => None,
                        }
                    }
                }
                Rc::new(Impl(args))
            })
                .add_alias("order_by_keys")
                .add_description_line("Sort an object by it's keys.")
                .add_description_line(
                    "If the first argument is an object, return object sorted by it's keys."
                )
                .add_example(
                    Example::new()
                        .add_argument("{\"z\": 1, \"x\": 2, \"w\": null}")
                        .expected_output("{\"w\":null,\"x\":2,\"z\":1}")
                )
                .add_example(Example::new().add_argument("false"))
        )

        .add_function(
            FunctionDefinitions::new("sort_by_values", 1, 1, |args| {
                struct Impl(Vec<Rc<dyn Get>>);
                impl Get for Impl {
                    fn get(&self, value: &Context) -> Option<JsonValue> {
                        match self.0.apply(value, 0) {
                            Some(JsonValue::Object(map)) => {
                                let mut map = map.clone();
                                map.sort_by(|_, v1, _, v2| v1.cmp(v2));

                                Some(map.into())
                            }
                            _ => None,
                        }
                    }
                }
                Rc::new(Impl(args))
            })
                .add_alias("order_by_values")
                .add_description_line("Sort an object by it's values.")
                .add_description_line(
                    "If the first argument is an object, return object sorted by it's values."
                )
                .add_example(
                    Example::new()
                        .add_argument("{\"z\": 5, \"x\": 2, \"w\": null}")
                        .expected_output("{\"w\":null,\"x\":2,\"z\":5}")
                )
                .add_example(Example::new().add_argument("false"))
        )

        .add_function(
            FunctionDefinitions::new("sort_by_values_by", 2, 2, |args| {
                struct Impl(Vec<Rc<dyn Get>>);
                impl Get for Impl {
                    fn get(&self, value: &Context) -> Option<JsonValue> {
                        match self.0.apply(value, 0) {
                            Some(JsonValue::Object(map)) => {
                                let mut map = map.clone();
                                map.sort_by(|_, v1, _, v2| {
                                    let v1 = value.with_inupt(v1.clone());
                                    let v1 = self.0.apply(&v1, 1);
                                    let v2 = value.with_inupt(v2.clone());
                                    let v2 = self.0.apply(&v2, 1);
                                    v1.cmp(&v2)
                                });

                                Some(map.into())
                            }
                            _ => None,
                        }
                    }
                }
                Rc::new(Impl(args))
            })
                .add_alias("order_by_values_by")
                .add_description_line("Sort an object by a function to it's values.")
                .add_description_line(
                    "If the first argument is an object, return object sorted by applying the second argumetn to it's values."
                )
                .add_example(
                    Example::new()
                        .add_argument(
                            "{\"a\": [1, 2, 3], \"b\": [1], \"c\": [2], \"d\": [3], \"e\": [0, null, 0]}"
                        )
                        .add_argument("(.len)")
                        .expected_output(
                            "{\"b\":[1],\"c\":[2],\"d\":[3],\"a\":[1,2,3],\"e\":[0,null,0]}"
                        )
                )
                .add_example(Example::new().add_argument("false").add_argument("."))
        )

        .add_function(
            FunctionDefinitions::new("filter_keys", 2, 2, |args| {
                struct Impl(Vec<Rc<dyn Get>>);
                impl Get for Impl {
                    fn get(&self, value: &Context) -> Option<JsonValue> {
                        if let Some(JsonValue::Object(map)) = self.0.apply(value, 0) {
                            Some(
                                map
                                    .into_iter()
                                    .filter(|(k, _)| {
                                        let k = value.with_inupt(k.into());
                                        self.0.apply(&k, 1) == Some(true.into())
                                    })
                                    .collect::<IndexMap<_, _>>()
                                    .into()
                            )
                        } else {
                            None
                        }
                    }
                }
                Rc::new(Impl(args))
            })
                .add_description_line("Filter an object by keys.")
                .add_description_line(
                    "The first argument should be the object and the second should be a function to filter the keys by."
                )
                .add_example(
                    Example::new()
                        .add_argument("{\"a\": 1, \"aa\": 2, \"aaa\": 3, \"aaaa\": 4}")
                        .add_argument("(>= (len .) 3)")
                        .expected_output("{\"aaa\": 3, \"aaaa\": 4}")
                        .explain("it filters all the keys that are shorter than 3 characters.")
                )
                .add_example(Example::new().add_argument("[1, 2, 4]").add_argument("false"))
        )

        .add_function(
            FunctionDefinitions::new("filter_values", 2, 2, |args| {
                struct Impl(Vec<Rc<dyn Get>>);
                impl Get for Impl {
                    fn get(&self, value: &Context) -> Option<JsonValue> {
                        if let Some(JsonValue::Object(map)) = self.0.apply(value, 0) {
                            Some(
                                map
                                    .into_iter()
                                    .filter(|(_, v)| {
                                        let v = value.with_inupt(v.clone());
                                        self.0.apply(&v, 1) == Some(true.into())
                                    })
                                    .collect::<IndexMap<_, _>>()
                                    .into()
                            )
                        } else {
                            None
                        }
                    }
                }
                Rc::new(Impl(args))
            })
                .add_description_line("Filter an object by values.")
                .add_description_line(
                    "The first argument should be the object and the second should be a function to filter the values by."
                )
                .add_example(
                    Example::new()
                        .add_argument("{\"a\": 1, \"aa\": 2, \"aaa\": 3, \"aaaa\": 4}")
                        .add_argument("(= 0 (% . 2))")
                        .expected_output("{\"aa\": 2, \"aaaa\": 4}")
                        .explain("it filters all the odds values")
                )
                .add_example(Example::new().add_argument("[1, 2, 4]").add_argument("false"))
        )

        .add_function(
            FunctionDefinitions::new("map_values", 2, 2, |args| {
                struct Impl(Vec<Rc<dyn Get>>);
                impl Get for Impl {
                    fn get(&self, value: &Context) -> Option<JsonValue> {
                        if let Some(JsonValue::Object(map)) = self.0.apply(value, 0) {
                            Some(
                                map
                                    .into_iter()
                                    .filter_map(|(k, v)| {
                                        let v = value.with_inupt(v);
                                        self.0.apply(&v, 1).map(|v| (k, v))
                                    })
                                    .collect::<IndexMap<_, _>>()
                                    .into()
                            )
                        } else {
                            None
                        }
                    }
                }
                Rc::new(Impl(args))
            })
                .add_description_line("Map an object values.")
                .add_description_line(
                    "The first argument should be the object and the second should be a function to map the values to."
                )
                .add_example(
                    Example::new()
                        .add_argument("{\"a\": 1, \"aa\": 2, \"aaa\": 3, \"aaaa\": 4}")
                        .add_argument("(% . 2)")
                        .expected_output("{\"a\": 1, \"aa\": 0, \"aaa\": 1, \"aaaa\": 0}")
                )
                .add_example(
                    Example::new()
                        .input("3")
                        .add_argument("{\"a\": 1, \"aa\": 2, \"aaa\": 3, \"aaaa\": 4}")
                        .add_argument("(+ . ^.)")
                        .expected_output("{\"a\": 4, \"aa\": 5, \"aaa\": 6, \"aaaa\": 7}")
                        .explain("it adds the input (3) to all the values.")
                )
                .add_example(Example::new().add_argument("[1, 2, 4]").add_argument("false"))
        )

        .add_function(
            FunctionDefinitions::new("map_keys", 2, 2, |args| {
                struct Impl(Vec<Rc<dyn Get>>);
                impl Get for Impl {
                    fn get(&self, value: &Context) -> Option<JsonValue> {
                        if let Some(JsonValue::Object(map)) = self.0.apply(value, 0) {
                            Some(
                                map
                                    .into_iter()
                                    .filter_map(|(k, v)| {
                                        let k = value.with_inupt(k.into());
                                        match self.0.apply(&k, 1) {
                                            Some(JsonValue::String(str)) => Some((str, v)),
                                            _ => None,
                                        }
                                    })
                                    .collect::<IndexMap<_, _>>()
                                    .into()
                            )
                        } else {
                            None
                        }
                    }
                }
                Rc::new(Impl(args))
            })
                .add_description_line("Map an object keys.")
                .add_description_line(
                    "The first argument should be the object and the second should be a function to map the keys to."
                )
                .add_example(
                    Example::new()
                        .add_argument("{\"a\": 1, \"aa\": 2, \"aaa\": 3, \"aaaa\": 4}")
                        .add_argument("(concat \"_\" .)")
                        .expected_output("{\"_a\": 1, \"_aa\": 2, \"_aaa\": 3, \"_aaaa\": 4}")
                )
                .add_example(
                    Example::new()
                        .input("\"prefix-\"")
                        .add_argument("{\"a\": 1, \"aa\": 2, \"aaa\": 3, \"aaaa\": 4}")
                        .add_argument("(concat ^. .)")
                        .expected_output(
                            "{\"prefix-a\": 1, \"prefix-aa\": 2, \"prefix-aaa\": 3, \"prefix-aaaa\": 4}"
                        )
                )
                .add_example(
                    Example::new()
                        .add_argument("{\"a\": 1, \"aa\": 2, \"aaa\": 3, \"aaaa\": 4}")
                        .add_argument("(number? .)")
                        .expected_output("{}")
                )
                .add_example(Example::new().add_argument("[1, 2, 4]").add_argument("false"))
        )

        .add_function(
            FunctionDefinitions::new("put", 3, 3, |args| {
                struct Impl(Vec<Rc<dyn Get>>);
                impl Get for Impl {
                    fn get(&self, value: &Context) -> Option<JsonValue> {
                        if
                            let (
                                Some(JsonValue::Object(map)),
                                Some(JsonValue::String(key)),
                                Some(val),
                            ) = (
                                self.0.apply(value, 0),
                                self.0.apply(value, 1),
                                self.0.apply(value, 2),
                            )
                        {
                            let mut new_map = map.clone();
                            new_map.insert(key, val);
                            Some(new_map.into())
                        } else {
                            None
                        }
                    }
                }
                Rc::new(Impl(args))
            })
                .add_alias("insert")
                .add_alias("replace")
                .add_alias("{}")
                .add_description_line("Add a new entry to a map.")
                .add_description_line("The first argument should be an object.")
                .add_description_line("The second argument should be a key.")
                .add_description_line("The third argument should be a value.")
                .add_description_line("If the object has that key, it will be replaced.")
                .add_example(
                    Example::new()
                        .add_argument("{}")
                        .add_argument("\"a\"")
                        .add_argument("1")
                        .expected_output("{\"a\": 1}")
                )
                .add_example(
                    Example::new()
                        .add_argument("{\"a\": 10, \"b\": 22}")
                        .add_argument("\"a\"")
                        .add_argument("-1")
                        .expected_output("{\"a\": -1, \"b\": 22}")
                )
                .add_example(
                    Example::new().add_argument("[]").add_argument("\"a\"").add_argument("1")
                )
                .add_example(Example::new().add_argument("{}").add_argument("1").add_argument("1"))
                .add_example(
                    Example::new()
                        .add_argument("{}")
                        .add_argument("\"1\"")
                        .add_argument("({} {} 1 1)")
                )
        )

        .add_function(
            FunctionDefinitions::new("insert_if_absent", 3, 3, |args| {
                struct Impl(Vec<Rc<dyn Get>>);
                impl Get for Impl {
                    fn get(&self, value: &Context) -> Option<JsonValue> {
                        if
                            let (
                                Some(JsonValue::Object(map)),
                                Some(JsonValue::String(key)),
                                Some(val),
                            ) = (
                                self.0.apply(value, 0),
                                self.0.apply(value, 1),
                                self.0.apply(value, 2),
                            )
                        {
                            if map.contains_key(&key) {
                                Some(map.into())
                            } else {
                                let mut new_map = map.clone();
                                new_map.insert(key, val);
                                Some(new_map.into())
                            }
                        } else {
                            None
                        }
                    }
                }
                Rc::new(Impl(args))
            })
                .add_alias("put_if_absent")
                .add_alias("replace_if_absent")
                .add_alias("{-}")
                .add_description_line("Add a new entry to a map if it has no such key.")
                .add_description_line("The first argument should be an object.")
                .add_description_line("The second argument should be a key.")
                .add_description_line("The third argument should be a value.")
                .add_description_line("If the object has that key, it will not be replaced.")
                .add_example(
                    Example::new()
                        .add_argument("{}")
                        .add_argument("\"a\"")
                        .add_argument("1")
                        .expected_output("{\"a\": 1}")
                )
                .add_example(
                    Example::new()
                        .add_argument("{\"a\": 10, \"b\": 22}")
                        .add_argument("\"a\"")
                        .add_argument("-1")
                        .expected_output("{\"a\": 10, \"b\": 22}")
                        .explain("the object already has a value with the `a` key.")
                )
                .add_example(
                    Example::new().add_argument("[]").add_argument("\"a\"").add_argument("1")
                )
                .add_example(Example::new().add_argument("{}").add_argument("1").add_argument("1"))
                .add_example(
                    Example::new()
                        .add_argument("{}")
                        .add_argument("\"1\"")
                        .add_argument("({} {} 1 1)")
                )
        )

        .add_function(
            FunctionDefinitions::new("replace_if_exists", 3, 3, |args| {
                struct Impl(Vec<Rc<dyn Get>>);
                impl Get for Impl {
                    fn get(&self, value: &Context) -> Option<JsonValue> {
                        if
                            let (
                                Some(JsonValue::Object(map)),
                                Some(JsonValue::String(key)),
                                Some(val),
                            ) = (
                                self.0.apply(value, 0),
                                self.0.apply(value, 1),
                                self.0.apply(value, 2),
                            )
                        {
                            if map.contains_key(&key) {
                                let mut new_map = map.clone();
                                new_map.insert(key, val);
                                Some(new_map.into())
                            } else {
                                Some(map.into())
                            }
                        } else {
                            None
                        }
                    }
                }
                Rc::new(Impl(args))
            })
                .add_alias("insert_if_exists")
                .add_alias("put_if_exists")
                .add_alias("{+}")
                .add_description_line("Add a new entry to a map if it has such key.")
                .add_description_line("The first argument should be an object.")
                .add_description_line("The second argument should be a key.")
                .add_description_line("The third argument should be a value.")
                .add_description_line(
                    "If the object dosen't has that key, it will not be replaced."
                )
                .add_example(
                    Example::new()
                        .add_argument("{}")
                        .add_argument("\"a\"")
                        .add_argument("1")
                        .expected_output("{}")
                        .explain("The object has no value for key `a`.")
                )
                .add_example(
                    Example::new()
                        .add_argument("{\"a\": 10, \"b\": 22}")
                        .add_argument("\"a\"")
                        .add_argument("-1")
                        .expected_output("{\"a\": -1, \"b\": 22}")
                )
                .add_example(
                    Example::new().add_argument("[]").add_argument("\"a\"").add_argument("1")
                )
                .add_example(Example::new().add_argument("{}").add_argument("1").add_argument("1"))
                .add_example(
                    Example::new()
                        .add_argument("{}")
                        .add_argument("\"1\"")
                        .add_argument("({} {} 1 1)")
                )
        )
}
