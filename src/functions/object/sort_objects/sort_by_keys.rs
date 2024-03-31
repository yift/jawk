use std::rc::Rc;

use crate::{
    functions_definitions::{Arguments, Example, FunctionDefinitions},
    json_value::JsonValue,
    processor::Context,
    selection::Get,
};

pub fn get() -> FunctionDefinitions {
    FunctionDefinitions::new("sort_by_keys", 1, 1, |args| {
        struct Impl(Vec<Rc<dyn Get>>);
        impl Get for Impl {
            fn get(&self, value: &Context) -> Option<JsonValue> {
                match self.0.apply(value, 0) {
                    Some(JsonValue::Object(map)) => {
                        let mut map = map.clone();
                        map.sort_keys();

                        Some(map.into())
                    }
                    _ => None,
                }
            }
        }
        Rc::new(Impl(args))
    })
    .add_alias("order_by_keys")
    .add_description_line("Sort an object by it's keys.")
    .add_description_line("If the first argument is an object, return object sorted by it's keys.")
    .add_example(
        Example::new()
            .add_argument("{\"z\": 1, \"x\": 2, \"w\": null}")
            .expected_output("{\"w\":null,\"x\":2,\"z\":1}"),
    )
    .add_example(Example::new().add_argument("false"))
}
