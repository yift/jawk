use std::rc::Rc;

use indexmap::IndexMap;

use crate::{
    functions_definitions::{Arguments, Example, FunctionDefinitions},
    json_value::JsonValue,
    processor::Context,
    selection::Get,
};

pub fn get() -> FunctionDefinitions {
    FunctionDefinitions::new("map_values", 2, 2, |args| {
        struct Impl(Vec<Rc<dyn Get>>);
        impl Get for Impl {
            fn get(&self, value: &Context) -> Option<JsonValue> {
                if let Some(JsonValue::Object(map)) = self.0.apply(value, 0) {
                    Some(
                        map
                            .into_iter()
                            .filter_map(|(k, v)| {
                                let v = value.with_inupt(v);
                                self.0.apply(&v, 1).map(|v| (k, v))
                            })
                            .collect::<IndexMap<_, _>>()
                            .into()
                    )
                } else {
                    None
                }
            }
        }
        Rc::new(Impl(args))
    })
        .add_description_line("Map an object values.")
        .add_description_line(
            "The first argument should be the object and the second should be a function to map the values to."
        )
        .add_example(
            Example::new()
                .add_argument("{\"a\": 1, \"aa\": 2, \"aaa\": 3, \"aaaa\": 4}")
                .add_argument("(% . 2)")
                .expected_output("{\"a\": 1, \"aa\": 0, \"aaa\": 1, \"aaaa\": 0}")
        )
        .add_example(
            Example::new()
                .input("3")
                .add_argument("{\"a\": 1, \"aa\": 2, \"aaa\": 3, \"aaaa\": 4}")
                .add_argument("(+ . ^.)")
                .expected_output("{\"a\": 4, \"aa\": 5, \"aaa\": 6, \"aaaa\": 7}")
                .explain("it adds the input (3) to all the values.")
        )
        .add_example(Example::new().add_argument("[1, 2, 4]").add_argument("false"))
}
