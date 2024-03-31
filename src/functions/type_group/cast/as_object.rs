use std::rc::Rc;

use crate::{
    functions_definitions::{Arguments, Example, FunctionDefinitions},
    json_value::JsonValue,
    processor::Context,
    selection::Get,
};

pub fn get() -> FunctionDefinitions {
    FunctionDefinitions::new("as_object", 1, 1, |args| {
        struct Impl(Vec<Rc<dyn Get>>);
        impl Get for Impl {
            fn get(&self, value: &Context) -> Option<JsonValue> {
                match self.0.apply(value, 0) {
                    Some(JsonValue::Object(b)) => Some(b.into()),
                    _ => None,
                }
            }
        }
        Rc::new(Impl(args))
    })
    .add_alias("as_map")
    .add_alias("as_hash")
    .add_description_line("return the object if the argument is an object, nothing if it's not.")
    .add_example(
        Example::new()
            .add_argument("{\"key\": 12}")
            .expected_output("{\"key\": 12}"),
    )
    .add_example(Example::new().add_argument("312"))
}
