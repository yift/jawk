use std::rc::Rc;

use crate::{
    functions_definitions::{Arguments, Example, FunctionDefinitions},
    json_value::JsonValue,
    processor::Context,
    selection::Get,
};

pub fn get() -> FunctionDefinitions {
    FunctionDefinitions::new("sort_by_values", 1, 1, |args| {
        struct Impl(Vec<Rc<dyn Get>>);
        impl Get for Impl {
            fn get(&self, value: &Context) -> Option<JsonValue> {
                match self.0.apply(value, 0) {
                    Some(JsonValue::Object(map)) => {
                        let mut map = map.clone();
                        map.sort_by(|_, v1, _, v2| v1.cmp(v2));

                        Some(map.into())
                    }
                    _ => None,
                }
            }
        }
        Rc::new(Impl(args))
    })
    .add_alias("order_by_values")
    .add_description_line("Sort an object by it's values.")
    .add_description_line(
        "If the first argument is an object, return object sorted by it's values.",
    )
    .add_example(
        Example::new()
            .add_argument("{\"z\": 5, \"x\": 2, \"w\": null}")
            .expected_output("{\"w\":null,\"x\":2,\"z\":5}"),
    )
    .add_example(Example::new().add_argument("false"))
}
