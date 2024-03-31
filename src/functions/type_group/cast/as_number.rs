use std::rc::Rc;

use crate::{
    functions_definitions::{Arguments, Example, FunctionDefinitions},
    json_value::JsonValue,
    processor::Context,
    selection::Get,
};

pub fn get() -> FunctionDefinitions {
    FunctionDefinitions::new("as_number", 1, 1, |args| {
        struct Impl(Vec<Rc<dyn Get>>);
        impl Get for Impl {
            fn get(&self, value: &Context) -> Option<JsonValue> {
                match self.0.apply(value, 0) {
                    Some(JsonValue::Number(num)) => Some(JsonValue::Number(num)),
                    _ => None,
                }
            }
        }
        Rc::new(Impl(args))
    })
    .add_description_line("return the number if the argument is a number, nothing if it's not.")
    .add_example(Example::new().add_argument("100").expected_output("100"))
    .add_example(Example::new().add_argument("-4.2").expected_output("-4.2"))
    .add_example(Example::new().add_argument("false"))
}
