use std::ops::Deref;

use crate::{
    functions_definitions::{Arguments, Example, FunctionDefinitions, FunctionsGroup},
    json_value::JsonValue,
    processor::Context,
    selection::Get,
};

use indexmap::IndexMap;
pub fn get_basic_functions() -> FunctionsGroup {
    FunctionsGroup::new("Basic functions")

        .add_function(
            FunctionDefinitions::new("get", 2, 2, |args| {
                struct Impl(Vec<Box<dyn Get>>);
                impl Get for Impl {
                    fn get(&self, value: &Context) -> Option<JsonValue> {
                        match self.0.apply(value, 0) {
                            Some(JsonValue::Object(map)) => {
                                if let Some(JsonValue::String(key)) = self.0.apply(value, 1) {
                                    map.get(&key).cloned()
                                } else {
                                    None
                                }
                            }
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
                            }
                            _ => None,
                        }
                    }
                }
                Box::new(Impl(args))
            })
                .add_alias("[]")
                .add_description_line("Get an item from an array by index or from a map by key.")
                .add_example(
                    Example::new()
                        .add_argument("[\"a\", \"b\", \"c\"]")
                        .add_argument("1")
                        .expected_output("\"b\"")
                )
                .add_example(
                    Example::new()
                        .add_argument("{\"key-1\": 12, \"key-2\": 32}")
                        .add_argument("\"key-1\"")
                        .expected_output("12")
                )
                .add_example(
                    Example::new().add_argument("[\"a\", \"b\", \"c\"]").add_argument("100")
                )
        )

        .add_function(
            FunctionDefinitions::new("|", 2, usize::MAX, |args| {
                struct Impl(Vec<Box<dyn Get>>);
                impl Get for Impl {
                    fn get(&self, value: &Context) -> Option<JsonValue> {
                        let mut value = value.input().deref().clone();
                        for e in &self.0 {
                            let context = Context::new(value);
                            if let Some(val) = e.get(&context) {
                                value = val;
                            } else {
                                return None;
                            }
                        }
                        Some(value)
                    }
                }
                Box::new(Impl(args))
            })
                .add_description_line("Pipe the output of one function to the next function.")
                .add_example(
                    Example::new()
                        .add_argument("(get . \"key\")")
                        .add_argument("(get . 3)")
                        .add_argument("(get . \"key-2\")")
                        .expected_output("100")
                        .input("{\"key\": [20, 40, 60, {\"key-2\": 100}]}")
                )
        )

        .add_function(
            FunctionDefinitions::new("size", 1, 1, |args| {
                struct Impl(Vec<Box<dyn Get>>);
                impl Get for Impl {
                    fn get(&self, value: &Context) -> Option<JsonValue> {
                        match self.0.apply(value, 0) {
                            Some(JsonValue::Object(map)) => { Some(map.len().into()) }
                            Some(JsonValue::Array(list)) => { Some(list.len().into()) }
                            Some(JsonValue::String(str)) => { Some(str.len().into()) }
                            _ => None,
                        }
                    }
                }
                Box::new(Impl(args))
            })
                .add_alias("count")
                .add_alias("length")
                .add_alias("len")
                .add_description_line("Get the number of element in an array,")
                .add_description_line("the number of keys in an object or the number of characters")
                .add_description_line("in a string.")
                .add_example(Example::new().add_argument("[1, 2, 3, 4]").expected_output("4"))
                .add_example(
                    Example::new()
                        .add_argument("{\"key-1\": 1, \"key-2\": false}")
                        .expected_output("2")
                )
                .add_example(Example::new().add_argument("\"123\"").expected_output("3"))
                .add_example(Example::new().add_argument("50"))
        )

        .add_function(
            FunctionDefinitions::new("take", 2, 2, |args| {
                struct Impl(Vec<Box<dyn Get>>);
                impl Get for Impl {
                    fn get(&self, value: &Context) -> Option<JsonValue> {
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
                                                    break;
                                                }
                                            }
                                            new_map
                                        };
                                        Some(map.into())
                                    }
                                    Some(JsonValue::Array(vec)) => {
                                        let vec = if size > vec.len() {
                                            vec
                                        } else {
                                            let mut new_vec = Vec::with_capacity(size);
                                            for i in vec {
                                                new_vec.push(i);
                                                if new_vec.len() == size {
                                                    break;
                                                }
                                            }
                                            new_vec
                                        };
                                        Some(vec.into())
                                    }
                                    Some(JsonValue::String(str)) => {
                                        let str = if size > str.len() {
                                            str
                                        } else {
                                            str[..size].into()
                                        };
                                        Some(str.into())
                                    }
                                    _ => None,
                                }
                            } else {
                                None
                            }
                        } else {
                            None
                        }
                    }
                }
                Box::new(Impl(args))
            })
                .add_alias("take_first")
                .add_description_line("Take the first N of element in an array, object of string")
                .add_example(
                    Example::new()
                        .add_argument("[1, 2, 3, 4]")
                        .add_argument("2")
                        .expected_output("[1, 2]")
                )
                .add_example(
                    Example::new()
                        .add_argument("[1, 2, 3, 4]")
                        .add_argument("6")
                        .expected_output("[1, 2, 3, 4]")
                )
                .add_example(
                    Example::new()
                        .add_argument("{\"key-1\": 1, \"key-2\": false}")
                        .add_argument("1")
                        .expected_output("{\"key-1\": 1}")
                )
                .add_example(
                    Example::new()
                        .add_argument("{\"key-1\": 1, \"key-2\": false}")
                        .add_argument("3")
                        .expected_output("{\"key-1\": 1, \"key-2\": false}")
                )
                .add_example(
                    Example::new()
                        .add_argument("\"123\"")
                        .add_argument("2")
                        .expected_output("\"12\"")
                )
                .add_example(
                    Example::new()
                        .add_argument("\"123\"")
                        .add_argument("20")
                        .expected_output("\"123\"")
                )
                .add_example(Example::new().add_argument("50").add_argument("10"))
                .add_example(Example::new().add_argument("\"123\"").add_argument("false"))
        )

        .add_function(
            FunctionDefinitions::new("take_last", 2, 2, |args| {
                struct Impl(Vec<Box<dyn Get>>);
                impl Get for Impl {
                    fn get(&self, value: &Context) -> Option<JsonValue> {
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
                                                index -= 1;
                                                if index < size {
                                                    new_map.insert(k, v);
                                                }
                                            }
                                            new_map
                                        };
                                        Some(map.into())
                                    }
                                    Some(JsonValue::Array(vec)) => {
                                        let vec = if size > vec.len() {
                                            vec
                                        } else {
                                            let mut new_vec = Vec::with_capacity(size);
                                            let mut index = vec.len();
                                            for i in vec {
                                                index -= 1;
                                                if index < size {
                                                    new_vec.push(i);
                                                }
                                            }
                                            new_vec
                                        };
                                        Some(vec.into())
                                    }
                                    Some(JsonValue::String(str)) => {
                                        let str = if size > str.len() {
                                            str
                                        } else {
                                            str[size - 1..].into()
                                        };
                                        Some(str.into())
                                    }
                                    _ => None,
                                }
                            } else {
                                None
                            }
                        } else {
                            None
                        }
                    }
                }
                Box::new(Impl(args))
            })
                .add_description_line("Take the last N of element in an array, object of string")
                .add_example(
                    Example::new()
                        .add_argument("[1, 2, 3, 4]")
                        .add_argument("2")
                        .expected_output("[3, 4]")
                )
                .add_example(
                    Example::new()
                        .add_argument("[1, 2, 3, 4]")
                        .add_argument("6")
                        .expected_output("[1, 2, 3, 4]")
                )
                .add_example(
                    Example::new()
                        .add_argument("{\"key-1\": 1, \"key-2\": false}")
                        .add_argument("1")
                        .expected_output("{\"key-2\": false}")
                )
                .add_example(
                    Example::new()
                        .add_argument("{\"key-1\": 1, \"key-2\": false}")
                        .add_argument("3")
                        .expected_output("{\"key-1\": 1, \"key-2\": false}")
                )
                .add_example(
                    Example::new()
                        .add_argument("\"123\"")
                        .add_argument("2")
                        .expected_output("\"23\"")
                )
                .add_example(
                    Example::new()
                        .add_argument("\"123\"")
                        .add_argument("20")
                        .expected_output("\"123\"")
                )
                .add_example(Example::new().add_argument("50").add_argument("10"))
                .add_example(Example::new().add_argument("\"123\"").add_argument("false"))
        )

        .add_function(
            FunctionDefinitions::new("sub", 3, 3, |args| {
                struct Impl(Vec<Box<dyn Get>>);
                impl Get for Impl {
                    fn get(&self, value: &Context) -> Option<JsonValue> {
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
                            }
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
                            }
                            Some(JsonValue::String(str)) => {
                                if start >= str.len() || length == 0 {
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
                            }
                            _ => None,
                        }
                    }
                }
                Box::new(Impl(args))
            })
                .add_description_line(
                    "If the first argument is a list, creates a new list that start from the second arguments and has the size of the third argument."
                )
                .add_description_line(
                    "If the first argument is an object, creates a new object that start from the second arguments and has the size of the third argument."
                )
                .add_description_line(
                    "If the first argument is a string, creates a substring that start from the second arguments and has the size of the third argument."
                )
                .add_example(
                    Example::new()
                        .add_argument("[1, 2, 3, 4, 5, 6]")
                        .add_argument("2")
                        .add_argument("3")
                        .expected_output("[3, 4, 5]")
                )
                .add_example(
                    Example::new()
                        .add_argument("[1, 2, 3, 4]")
                        .add_argument("6")
                        .add_argument("10")
                        .expected_output("[]")
                )
                .add_example(
                    Example::new()
                        .add_argument("[1, 2, 3, 4]")
                        .add_argument("0")
                        .add_argument("10")
                        .expected_output("[1, 2, 3, 4]")
                )
                .add_example(
                    Example::new()
                        .add_argument("[1, 2, 3, 4]")
                        .add_argument("1")
                        .add_argument("10")
                        .expected_output("[2, 3, 4]")
                )
                .add_example(
                    Example::new()
                        .add_argument(
                            "{\"key-1\": 1, \"key-2\": 2, \"key-3\": 3, \"key-4\": 4, \"key-5\": 5, \"key-6\": 6}"
                        )
                        .add_argument("1")
                        .add_argument("2")
                        .expected_output("{\"key-2\": 2, \"key-3\": 3}")
                )
                .add_example(
                    Example::new()
                        .add_argument(
                            "{\"key-1\": 1, \"key-2\": 2, \"key-3\": 3, \"key-4\": 4, \"key-5\": 5, \"key-6\": 6}"
                        )
                        .add_argument("0")
                        .add_argument("2")
                        .expected_output("{\"key-1\": 1, \"key-2\": 2}")
                )
                .add_example(
                    Example::new()
                        .add_argument(
                            "{\"key-1\": 1, \"key-2\": 2, \"key-3\": 3, \"key-4\": 4, \"key-5\": 5, \"key-6\": 6}"
                        )
                        .add_argument("4")
                        .add_argument("10")
                        .expected_output("{\"key-5\": 5, \"key-6\": 6}")
                )
                .add_example(
                    Example::new()
                        .add_argument(
                            "{\"key-1\": 1, \"key-2\": 2, \"key-3\": 3, \"key-4\": 4, \"key-5\": 5, \"key-6\": 6}"
                        )
                        .add_argument("20")
                        .add_argument("10")
                        .expected_output("{}")
                )
                .add_example(
                    Example::new()
                        .add_argument("\"123456\"")
                        .add_argument("1")
                        .add_argument("3")
                        .expected_output("\"234\"")
                )
                .add_example(
                    Example::new()
                        .add_argument("\"123456\"")
                        .add_argument("1")
                        .add_argument("30")
                        .expected_output("\"23456\"")
                )
                .add_example(
                    Example::new()
                        .add_argument("\"123456\"")
                        .add_argument("0")
                        .add_argument("30")
                        .expected_output("\"123456\"")
                )
                .add_example(
                    Example::new()
                        .add_argument("\"123456\"")
                        .add_argument("20")
                        .add_argument("30")
                        .expected_output("\"\"")
                )
                .add_example(
                    Example::new()
                        .add_argument("\"123456\"")
                        .add_argument("2")
                        .add_argument("0")
                        .expected_output("\"\"")
                )
                .add_example(Example::new().add_argument("50").add_argument("0").add_argument("10"))
                .add_example(
                    Example::new().add_argument("\"123\"").add_argument("false").add_argument("10")
                )
                .add_example(
                    Example::new().add_argument("\"123\"").add_argument("10").add_argument("{}")
                )
        )

        .add_function(
            FunctionDefinitions::new("default", 1, usize::MAX, |args| {
                struct Impl(Vec<Box<dyn Get>>);
                impl Get for Impl {
                    fn get(&self, value: &Context) -> Option<JsonValue> {
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
            })
                .add_alias("defaults")
                .add_alias("or_else")
                .add_description_line("Get the first non empty value.")
                .add_example(
                    Example::new()
                        .add_argument("(get . 1)")
                        .add_argument("(get . \"key-1\")")
                        .add_argument("22")
                        .expected_output("1")
                        .input("{\"key-1\": 1, \"key-2\": false}")
                )
                .add_example(
                    Example::new()
                        .add_argument("(get . 1)")
                        .add_argument("(get . \"key-3\")")
                        .add_argument("22")
                        .expected_output("22")
                        .input("{\"key-1\": 1, \"key-2\": false}")
                )
        )

        .add_function(
            FunctionDefinitions::new("?", 3, 3, |args| {
                struct Impl(Vec<Box<dyn Get>>);
                impl Get for Impl {
                    fn get(&self, value: &Context) -> Option<JsonValue> {
                        match self.0.apply(value, 0) {
                            Some(JsonValue::Boolean(true)) => self.0.apply(value, 1),
                            Some(JsonValue::Boolean(false)) => self.0.apply(value, 2),
                            _ => None,
                        }
                    }
                }
                Box::new(Impl(args))
            })
                .add_alias("if")
                .add_description_line(
                    "Return the second argument if the first argument is true. Return the third argument"
                )
                .add_description_line(
                    "if the first is false. Return nothing if the first argument is not Boolean"
                )
                .add_example(
                    Example::new()
                        .add_argument("true")
                        .add_argument("12")
                        .add_argument("22")
                        .expected_output("12")
                )
                .add_example(
                    Example::new()
                        .add_argument("false")
                        .add_argument("12")
                        .add_argument("22")
                        .expected_output("22")
                )
                .add_example(
                    Example::new()
                        .add_argument("(array? .)")
                        .add_argument("#1")
                        .add_argument("#2")
                        .expected_output("2")
                        .input("[1, 2, 3]")
                )
                .add_example(
                    Example::new()
                        .add_argument("(null? .)")
                        .add_argument("#1")
                        .add_argument("#2")
                        .expected_output("3")
                        .input("[1, 2, 3]")
                )
        )

        .add_function(
            FunctionDefinitions::new("stringify", 1, 1, |args| {
                struct Impl(Vec<Box<dyn Get>>);
                impl Get for Impl {
                    fn get(&self, value: &Context) -> Option<JsonValue> {
                        self.0.apply(value, 0).map(|val| format!("{}", val).into())
                    }
                }
                Box::new(Impl(args))
            })
                .add_description_line("Return the JSON represantation of the object.")
                .add_example(Example::new().add_argument("true").expected_output("\"true\""))
                .add_example(Example::new().add_argument("1e2").expected_output("\"100\""))
                .add_example(
                    Example::new()
                        .add_argument("{\"key\": [1, 2, \"3\"]}")
                        .expected_output("\"{\\\"key\\\": [1, 2, \\\"3\\\"]}\"")
                )
        )
}
