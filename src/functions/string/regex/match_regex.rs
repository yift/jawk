use std::rc::Rc;

use crate::processor::Context;
use crate::regex_cache::RegexCompile;

use crate::{
    functions_definitions::{Arguments, Example, FunctionDefinitions},
    json_value::JsonValue,
    selection::Get,
};

pub fn get() -> FunctionDefinitions {
    FunctionDefinitions::new("match", 2, 2, |args| {
        struct Impl(Vec<Rc<dyn Get>>);
        impl Get for Impl {
            fn get(&self, value: &Context) -> Option<JsonValue> {
                if let (Some(JsonValue::String(str)), Some(JsonValue::String(regex))) =
                    (self.0.apply(value, 0), self.0.apply(value, 1))
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
        "Return true if the first string argument match the second regular expression argument.",
    )
    .add_description_line(
        "For regular expression syntax, see [https://docs.rs/regex/latest/regex/#syntax].",
    )
    .add_description_line(
        "Use `--regular_expression_cache_size` so set a cache for compiled regular expressions.",
    )
    .add_example(
        Example::new()
            .add_argument("\"test\"")
            .add_argument("\"[a-z]+\"")
            .expected_output("true"),
    )
    .add_example(
        Example::new()
            .add_argument("\"test\"")
            .add_argument("\"[0-9]+\"")
            .expected_output("false"),
    )
    .add_example(
        Example::new()
            .add_argument("\"test\"")
            .add_argument("\"[0-9\""),
    )
}
