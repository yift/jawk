use std::rc::Rc;

use crate::{
    functions_definitions::{Arguments, Example, FunctionDefinitions},
    json_value::JsonValue,
    processor::Context,
    selection::Get,
};

pub fn get() -> FunctionDefinitions {
    FunctionDefinitions::new("ceil", 1, 1, |args| {
        struct Impl(Vec<Rc<dyn Get>>);
        impl Get for Impl {
            fn get(&self, value: &Context) -> Option<JsonValue> {
                if let Some(JsonValue::Number(num)) = self.0.apply(value, 0) {
                    let num: f64 = num.into();
                    Some(num.ceil().into())
                } else {
                    None
                }
            }
        }
        Rc::new(Impl(args))
    })
    .add_alias("ceiling")
    .add_description_line("If the argument is numeric, return it's ceiling.")
    .add_example(Example::new().add_argument("10.3").expected_output("11"))
    .add_example(Example::new().add_argument("-10.3").expected_output("-10"))
    .add_example(Example::new().add_argument("-10").expected_output("-10"))
    .add_example(Example::new().add_argument("-10.5").expected_output("-10"))
    .add_example(Example::new().add_argument("10.99").expected_output("11"))
    .add_example(Example::new().add_argument("[0]"))
}
