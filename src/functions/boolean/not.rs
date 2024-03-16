use std::rc::Rc;

use crate::{
    functions_definitions::{Arguments, Example, FunctionDefinitions},
    json_value::JsonValue,
    processor::Context,
    selection::Get,
};

pub fn get() -> FunctionDefinitions {
    FunctionDefinitions::new("not", 1, 1, |args| {
        struct Impl(Vec<Rc<dyn Get>>);
        impl Get for Impl {
            fn get(&self, value: &Context) -> Option<JsonValue> {
                if let Some(JsonValue::Boolean(val1)) = self.0.apply(value, 0) {
                    let eq = !val1;
                    Some(eq.into())
                } else {
                    None
                }
            }
        }
        Rc::new(Impl(args))
    })
    .add_alias("!")
    .add_description_line("Return false if the argument is true and true if the argument is false.")
    .add_example(Example::new().add_argument("true").expected_output("false"))
    .add_example(Example::new().add_argument("false").expected_output("true"))
    .add_example(
        Example::new()
            .add_argument("(string? 12)")
            .expected_output("true"),
    )
    .add_example(Example::new().add_argument("12"))
}
