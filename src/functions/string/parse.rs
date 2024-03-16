use std::rc::Rc;

use crate::json_parser::JsonParser;
use crate::processor::Context;

use crate::{
    functions_definitions::{Arguments, Example, FunctionDefinitions},
    json_value::JsonValue,
    reader::from_string,
    selection::Get,
};

pub fn get() -> FunctionDefinitions {
    FunctionDefinitions::new("parse", 1, 1, |args| {
        struct Impl(Vec<Rc<dyn Get>>);
        impl Get for Impl {
            fn get(&self, value: &Context) -> Option<JsonValue> {
                match self.0.apply(value, 0) {
                    Some(JsonValue::String(str)) => {
                        let mut reader = from_string(&str);
                        match reader.next_json_value() {
                            Ok(Some(first_value)) => match reader.next_json_value() {
                                Ok(None) => Some(first_value),
                                _ => None,
                            },
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
        Example::new()
            .add_argument("\"[1, 2, 3, 4]\"")
            .expected_output("[1, 2, 3, 4]"),
    )
    .add_example(
        Example::new()
            .add_argument("\"312\"")
            .expected_output("312"),
    )
    .add_example(Example::new().add_argument("\"{}\"").expected_output("{}"))
    .add_example(Example::new().add_argument("400"))
}
