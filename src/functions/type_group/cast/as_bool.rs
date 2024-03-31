use std::rc::Rc;

use crate::{
    functions_definitions::{Arguments, Example, FunctionDefinitions},
    json_value::JsonValue,
    processor::Context,
    selection::Get,
};

pub fn get() -> FunctionDefinitions {
    FunctionDefinitions::new("as_boolean", 1, 1, |args| {
        struct Impl(Vec<Rc<dyn Get>>);
        impl Get for Impl {
            fn get(&self, value: &Context) -> Option<JsonValue> {
                match self.0.apply(value, 0) {
                    Some(JsonValue::Boolean(b)) => Some(b.into()),
                    _ => None,
                }
            }
        }
        Rc::new(Impl(args))
    })
    .add_alias("as_bool")
    .add_description_line("return the Boolean if the argument is a Boolean, nothing if it's not.")
    .add_example(
        Example::new()
            .add_argument("false")
            .expected_output("false"),
    )
    .add_example(Example::new().add_argument("true").expected_output("true"))
    .add_example(Example::new().add_argument("312"))
}
