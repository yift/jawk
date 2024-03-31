use std::rc::Rc;
use std::str::FromStr;

use crate::processor::Context;

use crate::selection::Selection;
use crate::{
    functions_definitions::{Arguments, Example, FunctionDefinitions},
    json_value::JsonValue,
    selection::Get,
};

pub fn get() -> FunctionDefinitions {
    FunctionDefinitions::new("parse_selection", 1, 1, |args| {
        struct Impl(Vec<Rc<dyn Get>>);
        impl Get for Impl {
            fn get(&self, value: &Context) -> Option<JsonValue> {
                match self.0.apply(value, 0) {
                    Some(JsonValue::String(str)) => match Selection::from_str(str.as_str()) {
                        Ok(selection) => selection.get(value),
                        _ => None,
                    },
                    _ => None,
                }
            }
        }
        Rc::new(Impl(args))
    })
    .add_description_line("Parse a string into a new selection.")
    .add_example(
        Example::new()
            .add_argument("\"(+ 10 11)\"")
            .expected_output("21"),
    )
    .add_example(Example::new().add_argument("false"))
}
