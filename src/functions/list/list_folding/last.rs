use std::rc::Rc;

use crate::{
    functions_definitions::{Arguments, Example, FunctionDefinitions},
    json_value::JsonValue,
    processor::Context,
    selection::Get,
};

pub fn get() -> FunctionDefinitions {
    FunctionDefinitions::new("last", 1, 1, |args| {
        struct Impl(Vec<Rc<dyn Get>>);
        impl Get for Impl {
            fn get(&self, value: &Context) -> Option<JsonValue> {
                match self.0.apply(value, 0) {
                    Some(JsonValue::Array(list)) => list.last().cloned(),
                    _ => None,
                }
            }
        }
        Rc::new(Impl(args))
    })
    .add_description_line("The last item in a list.")
    .add_example(
        Example::new()
            .add_argument("[1, 5, 1.1]")
            .expected_output("1.1"),
    )
    .add_example(Example::new().add_argument("[]"))
    .add_example(
        Example::new()
            .add_argument("[\"text\"]")
            .expected_output("\"text\""),
    )
    .add_example(Example::new().add_argument("\"text\""))
}
