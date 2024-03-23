use std::rc::Rc;

use crate::{
    functions_definitions::{Arguments, Example, FunctionDefinitions},
    json_value::JsonValue,
    processor::Context,
    selection::Get,
};

pub fn get() -> FunctionDefinitions {
    FunctionDefinitions::new("abs", 1, 1, |args| {
        struct Impl(Vec<Rc<dyn Get>>);
        impl Get for Impl {
            fn get(&self, value: &Context) -> Option<JsonValue> {
                if let Some(JsonValue::Number(num)) = self.0.apply(value, 0) {
                    let num: f64 = num.into();
                    Some(num.abs().into())
                } else {
                    None
                }
            }
        }
        Rc::new(Impl(args))
    })
    .add_description_line("If the argument is numeric, return it's absolute value.")
    .add_example(Example::new().add_argument("100").expected_output("100"))
    .add_example(Example::new().add_argument("-100").expected_output("100"))
    .add_example(Example::new().add_argument("0").expected_output("0"))
    .add_example(Example::new().add_argument("[0]"))
}
