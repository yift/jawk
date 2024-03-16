use std::rc::Rc;

use indexmap::IndexMap;

use crate::{
    functions_definitions::{Arguments, Example, FunctionDefinitions},
    json_value::JsonValue,
    processor::Context,
    selection::Get,
};

pub fn get() -> FunctionDefinitions {
    FunctionDefinitions::new("filter_keys", 2, 2, |args| {
                struct Impl(Vec<Rc<dyn Get>>);
                impl Get for Impl {
                    fn get(&self, value: &Context) -> Option<JsonValue> {
                        if let Some(JsonValue::Object(map)) = self.0.apply(value, 0) {
                            Some(
                                map
                                    .into_iter()
                                    .filter(|(k, _)| {
                                        let k = value.with_inupt(k.into());
                                        self.0.apply(&k, 1) == Some(true.into())
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
                .add_description_line("Filter an object by keys.")
                .add_description_line(
                    "The first argument should be the object and the second should be a function to filter the keys by."
                )
                .add_example(
                    Example::new()
                        .add_argument("{\"a\": 1, \"aa\": 2, \"aaa\": 3, \"aaaa\": 4}")
                        .add_argument("(>= (len .) 3)")
                        .expected_output("{\"aaa\": 3, \"aaaa\": 4}")
                        .explain("it filters all the keys that are shorter than 3 characters.")
                )
                .add_example(Example::new().add_argument("[1, 2, 4]").add_argument("false"))
}
