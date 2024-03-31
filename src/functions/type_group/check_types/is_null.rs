use std::rc::Rc;

use crate::{
    functions_definitions::{Arguments, Example, FunctionDefinitions},
    json_value::JsonValue,
    processor::Context,
    selection::Get,
};

pub fn get() -> FunctionDefinitions {
    FunctionDefinitions::new("null?", 1, 1, |args| {
        struct Impl(Vec<Rc<dyn Get>>);
        impl Get for Impl {
            fn get(&self, value: &Context) -> Option<JsonValue> {
                match self.0.apply(value, 0) {
                    Some(JsonValue::Null) => Some(true.into()),
                    _ => Some(false.into()),
                }
            }
        }
        Rc::new(Impl(args))
    })
    .add_alias("nil?")
    .add_description_line("return true if the argument is a null.")
    .add_example(Example::new().add_argument("null").expected_output("true"))
    .add_example(Example::new().add_argument("1").expected_output("false"))
}
