use std::rc::Rc;

use crate::processor::Context;
use crate::regex_cache::RegexCompile;

use crate::{
    functions_definitions::{Arguments, Example, FunctionDefinitions},
    json_value::JsonValue,
    selection::Get,
};

pub fn get() -> FunctionDefinitions {
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
}
