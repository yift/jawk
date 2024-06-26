use std::rc::Rc;

use crate::{
    functions_definitions::{Arguments, Example, FunctionDefinitions},
    json_value::JsonValue,
    processor::Context,
    selection::Get,
};

pub fn get() -> FunctionDefinitions {
    FunctionDefinitions::new("as_array", 1, 1, |args| {
        struct Impl(Vec<Rc<dyn Get>>);
        impl Get for Impl {
            fn get(&self, value: &Context) -> Option<JsonValue> {
                match self.0.apply(value, 0) {
                    Some(JsonValue::Array(lst)) => Some(lst.into()),
                    _ => None,
                }
            }
        }
        Rc::new(Impl(args))
    })
    .add_alias("as_list")
    .add_description_line("return the array if the argument is an array, nothing if it's not.")
    .add_example(
        Example::new()
            .add_argument("[1, 2, 3, 4]")
            .expected_output("[1, 2, 3, 4]"),
    )
    .add_example(Example::new().add_argument("312"))
}
