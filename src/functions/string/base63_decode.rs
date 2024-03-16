use base64::prelude::*;
use std::rc::Rc;

use crate::processor::Context;

use crate::{
    functions_definitions::{Arguments, Example, FunctionDefinitions},
    json_value::JsonValue,
    selection::Get,
};

pub fn get() -> FunctionDefinitions {
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
}
