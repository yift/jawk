use std::env::var;

use base64::prelude::*;
use std::rc::Rc;
use std::str::FromStr;

use crate::json_parser::JsonParser;
use crate::processor::Context;
use crate::regex_cache::RegexCompile;
use crate::selection::Selection;
use crate::{
    functions_definitions::{Arguments, Example, FunctionDefinitions, FunctionsGroup},
    json_value::JsonValue,
    reader::from_string,
    selection::Get,
};

pub fn get_string_functions() -> FunctionsGroup {
    FunctionsGroup::new("string")

        .add_function(
            FunctionDefinitions::new("parse", 1, 1, |args| {
                struct Impl(Vec<Rc<dyn Get>>);
                impl Get for Impl {
                    fn get(&self, value: &Context) -> Option<JsonValue> {
                        match self.0.apply(value, 0) {
                            Some(JsonValue::String(str)) => {
                                let mut reader = from_string(&str);
                                match reader.next_json_value() {
                                    Ok(Some(first_value)) =>
                                        match reader.next_json_value() {
                                            Ok(None) => Some(first_value),
                                            _ => None,
                                        }
                                    _ => None,
                                }
                            }
                            _ => None,
                        }
                    }
                }
                Rc::new(Impl(args))
            })
                .add_alias("parse_json")
                .add_description_line("Parse a string into JSON value.")
                .add_example(
                    Example::new().add_argument("\"[1, 2, 3, 4]\"").expected_output("[1, 2, 3, 4]")
                )
                .add_example(Example::new().add_argument("\"312\"").expected_output("312"))
                .add_example(Example::new().add_argument("\"{}\"").expected_output("{}"))
                .add_example(Example::new().add_argument("400"))
        )

        .add_function(
            FunctionDefinitions::new("parse_selection", 1, 1, |args| {
                struct Impl(Vec<Rc<dyn Get>>);
                impl Get for Impl {
                    fn get(&self, value: &Context) -> Option<JsonValue> {
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
                Rc::new(Impl(args))
            })
                .add_description_line("Parse a string into a new selection.")
                .add_example(Example::new().add_argument("\"(+ 10 11)\"").expected_output("21"))
                .add_example(Example::new().add_argument("false"))
        )

        .add_function(
            FunctionDefinitions::new("env", 1, 1, |args| {
                struct Impl(Vec<Rc<dyn Get>>);
                impl Get for Impl {
                    fn get(&self, value: &Context) -> Option<JsonValue> {
                        if let Some(JsonValue::String(str)) = self.0.apply(value, 0) {
                            if let Ok(value) = var(str) { Some(value.into()) } else { None }
                        } else {
                            None
                        }
                    }
                }
                Rc::new(Impl(args))
            })
                .add_alias("$")
                .add_description_line("Get enviornment variable.")
                .add_example(
                    Example::new()
                        .add_argument("\"PATH\"")
                        .expected_json(var("PATH").map(Into::into).ok())
                        .more_or_less()
                )
        )

        .add_function(
            FunctionDefinitions::new("concat", 2, usize::MAX, |args| {
                struct Impl(Vec<Rc<dyn Get>>);
                impl Get for Impl {
                    fn get(&self, value: &Context) -> Option<JsonValue> {
                        let mut all = String::new();
                        for s in &self.0 {
                            if let Some(JsonValue::String(str)) = s.get(value) {
                                all.push_str(str.as_str());
                            } else {
                                return None;
                            }
                        }
                        Some(all.into())
                    }
                }
                Rc::new(Impl(args))
            })
                .add_description_line("Concat all string arguments.")
                .add_example(
                    Example::new()
                        .add_argument("\"one\"")
                        .add_argument("\" \"")
                        .add_argument("\"two\"")
                        .expected_output("\"one two\"")
                )
                .add_example(
                    Example::new().add_argument("\"one\"").add_argument("\" \"").add_argument("2")
                )
        )

        .add_function(
            FunctionDefinitions::new("head", 2, 2, |args| {
                struct Impl(Vec<Rc<dyn Get>>);
                impl Get for Impl {
                    fn get(&self, value: &Context) -> Option<JsonValue> {
                        if
                            let (Some(JsonValue::String(str)), Some(JsonValue::Number(index))) = (
                                self.0.apply(value, 0),
                                self.0.apply(value, 1),
                            )
                        {
                            if let Ok(index) = TryInto::<usize>::try_into(index) {
                                if str.len() < index {
                                    Some(str.into())
                                } else {
                                    let head = str[..index].to_string();
                                    Some(head.into())
                                }
                            } else {
                                None
                            }
                        } else {
                            None
                        }
                    }
                }
                Rc::new(Impl(args))
            })
                .add_description_line("Extract a string header.")
                .add_description_line(
                    "If the first argument is a string and the second argument is a positive integer, the returned value will be a string with the beggining of the first argument."
                )
                .add_description_line("See also `take`.")
                .add_example(
                    Example::new()
                        .add_argument("\"test-123\"")
                        .add_argument("4")
                        .expected_output("\"test\"")
                )
                .add_example(
                    Example::new()
                        .add_argument("\"test-123\"")
                        .add_argument("20")
                        .expected_output("\"test-123\"")
                )
                .add_example(Example::new().add_argument("20").add_argument("20"))
                .add_example(Example::new().add_argument("\"20\"").add_argument("-5"))
        )

        .add_function(
            FunctionDefinitions::new("tail", 2, 2, |args| {
                struct Impl(Vec<Rc<dyn Get>>);
                impl Get for Impl {
                    fn get(&self, value: &Context) -> Option<JsonValue> {
                        if
                            let (Some(JsonValue::String(str)), Some(JsonValue::Number(index))) = (
                                self.0.apply(value, 0),
                                self.0.apply(value, 1),
                            )
                        {
                            if let Ok(index) = TryInto::<usize>::try_into(index) {
                                if str.len() < index {
                                    Some(str.into())
                                } else {
                                    let head = str[index..].to_string();
                                    Some(head.into())
                                }
                            } else {
                                None
                            }
                        } else {
                            None
                        }
                    }
                }
                Rc::new(Impl(args))
            })
                .add_description_line("Extract a string tail.")
                .add_description_line(
                    "If the first argument is a string and the second argument is a positive integer, the returned value will be a string with the end of the first argument."
                )
                .add_description_line("See also `take_last`.")
                .add_example(
                    Example::new()
                        .add_argument("\"test-123\"")
                        .add_argument("4")
                        .expected_output("\"-123\"")
                )
                .add_example(
                    Example::new()
                        .add_argument("\"test-123\"")
                        .add_argument("20")
                        .expected_output("\"test-123\"")
                )
                .add_example(Example::new().add_argument("20").add_argument("20"))
                .add_example(Example::new().add_argument("\"20\"").add_argument("-5"))
        )

        .add_function(
            FunctionDefinitions::new("split", 2, 2, |args| {
                struct Impl(Vec<Rc<dyn Get>>);
                impl Get for Impl {
                    fn get(&self, value: &Context) -> Option<JsonValue> {
                        if
                            let (Some(JsonValue::String(str)), Some(JsonValue::String(splitter))) =
                                (self.0.apply(value, 0), self.0.apply(value, 1))
                        {
                            Some(
                                str
                                    .split(splitter.as_str())
                                    .map(|f| JsonValue::String(f.to_string()))
                                    .collect::<Vec<_>>()
                                    .into()
                            )
                        } else {
                            None
                        }
                    }
                }
                Rc::new(Impl(args))
            })
                .add_description_line("Split the string into array of strings.")
                .add_example(
                    Example::new()
                        .add_argument("\"one, two, three\"")
                        .add_argument("\", \"")
                        .expected_output("[\"one\", \"two\", \"three\"]")
                )
                .add_example(
                    Example::new()
                        .add_argument("\"a|b|c\"")
                        .add_argument("\"|\"")
                        .expected_output("[\"a\", \"b\", \"c\"]")
                )
        )

        .add_function(
            FunctionDefinitions::new("match", 2, 2, |args| {
                struct Impl(Vec<Rc<dyn Get>>);
                impl Get for Impl {
                    fn get(&self, value: &Context) -> Option<JsonValue> {
                        if
                            let (Some(JsonValue::String(str)), Some(JsonValue::String(regex))) = (
                                self.0.apply(value, 0),
                                self.0.apply(value, 1),
                            )
                        {
                            if let Ok(regex) = &*value.compile_regex(&regex) {
                                Some(regex.is_match(&str).into())
                            } else {
                                None
                            }
                        } else {
                            None
                        }
                    }
                }
                Rc::new(Impl(args))
            })
                .add_alias("match_regex")
                .add_description_line(
                    "Return true if the first string argument match the second regular expression argument."
                )
                .add_description_line(
                    "For regular expression syntax, see [https://docs.rs/regex/latest/regex/#syntax]."
                )
                .add_description_line(
                    "Use `--regular_expression_cache_size` so set a cache for compiled regular expressions."
                )
                .add_example(
                    Example::new()
                        .add_argument("\"test\"")
                        .add_argument("\"[a-z]+\"")
                        .expected_output("true")
                )
                .add_example(
                    Example::new()
                        .add_argument("\"test\"")
                        .add_argument("\"[0-9]+\"")
                        .expected_output("false")
                )
                .add_example(Example::new().add_argument("\"test\"").add_argument("\"[0-9\""))
        )

        .add_function(
            FunctionDefinitions::new("extract_regex_group", 3, 3, |args| {
                struct Impl(Vec<Rc<dyn Get>>);
                impl Get for Impl {
                    fn get(&self, value: &Context) -> Option<JsonValue> {
                        if
                            let (
                                Some(JsonValue::String(str)),
                                Some(JsonValue::String(regex)),
                                Some(JsonValue::Number(index)),
                            ) = (
                                self.0.apply(value, 0),
                                self.0.apply(value, 1),
                                self.0.apply(value, 2),
                            )
                        {
                            if
                                let (Ok(regex), Ok(index)) = (
                                    &*value.compile_regex(&regex),
                                    TryInto::<usize>::try_into(index),
                                )
                            {
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
                Rc::new(Impl(args))
            })
                .add_description_line("Return the capture group within the string.")
                .add_description_line(
                    "The first argument is expected to be the string to apply the expression on."
                )
                .add_description_line(
                    "The second argument is expected to be the string with the regular expression."
                )
                .add_description_line(
                    "The third argument is expected to be the group index with in the regular epression (the first group index is one)."
                )
                .add_description_line(
                    "For regular expression syntax, see [https://docs.rs/regex/latest/regex/#syntax]."
                )
                .add_description_line(
                    "Use `--regular_expression_cache_size` so set a cache for compiled regular expressions."
                )
                .add_example(
                    Example::new()
                        .add_argument("\"hello 200 world\"")
                        .add_argument("\"[a-z ]+([0-9]+)[a-z ]+\"")
                        .add_argument("1")
                        .expected_output("\"200\"")
                        .explain(
                            "the regular expression is letters and spaces, group with numbers, and more letter and spaces, so the group is the string `\"200\"`."
                        )
                )
                .add_example(
                    Example::new()
                        .add_argument("\"hello 200 world\"")
                        .add_argument("\"[a-z ]+([0-9]+)[a-z ]+\"")
                        .add_argument("20")
                )
                .add_example(
                    Example::new()
                        .add_argument("\"hello 200 world\"")
                        .add_argument("\"[a-z ]+([0-9]+)[a-z ]+\"")
                        .add_argument("0")
                        .expected_output("\"hello 200 world\"")
                )
                .add_example(
                    Example::new()
                        .add_argument("\"test\"")
                        .add_argument("\"[0-9\"")
                        .add_argument("10")
                )
        )

        .add_function(
            FunctionDefinitions::new("base63_decode", 1, 1, |args| {
                struct Impl(Vec<Rc<dyn Get>>);
                impl Get for Impl {
                    fn get(&self, value: &Context) -> Option<JsonValue> {
                        let Some(JsonValue::String(str)) = self.0.apply(value, 0) else {
                            return None;
                        };
                        let Ok(data) = BASE64_STANDARD.decode(str) else {
                            return None;
                        };
                        if let Ok(str) = String::from_utf8(data) {
                            Some(str.into())
                        } else {
                            None
                        }
                    }
                }
                Rc::new(Impl(args))
            })
            .add_alias("base64")
                .add_description_line("Decode a BASE64 string and try to convert to a string using UTF8.")
                .add_description_line(
                    "Retunr nothing if the first and only argument is not a valid UTF8 string encoded using BASE64."
                )
                .add_example(
                    Example::new()
                        .add_argument("\"dGVzdA==\"")
                        .expected_output("\"test\"")
                )
                .add_example(
                    Example::new()
                        .add_argument("\"test\"")
                )
                .add_example(
                    Example::new()
                        .add_argument("\"wyg=\"")
                )
                .add_example(
                    Example::new()
                        .add_argument("100")
                )
        )
}
