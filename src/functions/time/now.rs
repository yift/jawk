use std::{rc::Rc, time::SystemTime};

use crate::{
    functions_definitions::{Example, FunctionDefinitions},
    json_value::{JsonValue, NumberValue},
    processor::Context,
    selection::Get,
};

pub fn get() -> FunctionDefinitions {
    FunctionDefinitions::new("now", 0, 0, |_| {
        struct Impl;
        impl Get for Impl {
            fn get(&self, _: &Context) -> Option<JsonValue> {
                match SystemTime::now().duration_since(SystemTime::UNIX_EPOCH) {
                    Ok(d) => Some(d.as_secs_f64().into()),
                    _ => None,
                }
            }
        }
        Rc::new(Impl)
    })
    .add_description_line("Return the current time as seconds since epoch.")
    .add_example(
        Example::new()
            .validate_output(|value| match value {
                Some(JsonValue::Number(NumberValue::Float(num))) => {
                    match SystemTime::now().duration_since(SystemTime::UNIX_EPOCH) {
                        Ok(d) => d.as_secs_f64() >= *num,
                        _ => false,
                    }
                }
                _ => false,
            })
            .more_or_less(),
    )
}
