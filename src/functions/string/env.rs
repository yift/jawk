use std::env::var;

use std::rc::Rc;

use crate::processor::Context;

use crate::{
    functions_definitions::{Arguments, Example, FunctionDefinitions},
    json_value::JsonValue,
    selection::Get,
};

pub fn get() -> FunctionDefinitions {
    FunctionDefinitions::new("env", 1, 1, |args| {
        struct Impl(Vec<Rc<dyn Get>>);
        impl Get for Impl {
            fn get(&self, value: &Context) -> Option<JsonValue> {
                if let Some(JsonValue::String(str)) = self.0.apply(value, 0) {
                    if let Ok(value) = var(str) {
                        Some(value.into())
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
    .add_alias("$")
    .add_description_line("Get enviornment variable.")
    .add_example(
        Example::new()
            .add_argument("\"PATH\"")
            .expected_json(var("PATH").map(Into::into).ok())
            .more_or_less(),
    )
}
