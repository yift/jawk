use std::rc::Rc;

use indexmap::IndexMap;

use crate::{
    functions_definitions::{Arguments, Example, FunctionDefinitions},
    json_value::JsonValue,
    processor::Context,
    selection::Get,
};

pub fn get() -> FunctionDefinitions {
    FunctionDefinitions::new("map_keys", 2, 2, |args| {
        struct Impl(Vec<Rc<dyn Get>>);
        impl Get for Impl {
            fn get(&self, value: &Context) -> Option<JsonValue> {
                if let Some(JsonValue::Object(map)) = self.0.apply(value, 0) {
                    Some(
                        map
                            .into_iter()
                            .filter_map(|(k, v)| {
                                let k = value.with_inupt(k.into());
                                match self.0.apply(&k, 1) {
                                    Some(JsonValue::String(str)) => Some((str, v)),
                                    _ => None,
                                }
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
        .add_description_line("Map an object keys.")
        .add_description_line(
            "The first argument should be the object and the second should be a function to map the keys to."
        )
        .add_example(
            Example::new()
                .add_argument("{\"a\": 1, \"aa\": 2, \"aaa\": 3, \"aaaa\": 4}")
                .add_argument("(concat \"_\" .)")
                .expected_output("{\"_a\": 1, \"_aa\": 2, \"_aaa\": 3, \"_aaaa\": 4}")
        )
        .add_example(
            Example::new()
                .input("\"prefix-\"")
                .add_argument("{\"a\": 1, \"aa\": 2, \"aaa\": 3, \"aaaa\": 4}")
                .add_argument("(concat ^. .)")
                .expected_output(
                    "{\"prefix-a\": 1, \"prefix-aa\": 2, \"prefix-aaa\": 3, \"prefix-aaaa\": 4}"
                )
        )
        .add_example(
            Example::new()
                .add_argument("{\"a\": 1, \"aa\": 2, \"aaa\": 3, \"aaaa\": 4}")
                .add_argument("(number? .)")
                .expected_output("{}")
        )
        .add_example(Example::new().add_argument("[1, 2, 4]").add_argument("false"))
}
