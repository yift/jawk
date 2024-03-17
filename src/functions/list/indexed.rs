use std::rc::Rc;

use indexmap::IndexMap;

use crate::{
    functions_definitions::{Arguments, Example, FunctionDefinitions},
    json_value::JsonValue,
    processor::Context,
    selection::Get,
};

pub fn get() -> FunctionDefinitions {
    FunctionDefinitions::new("indexed", 1, 1, |args| {
        struct Impl(Vec<Rc<dyn Get>>);
        impl Get for Impl {
            fn get(&self, value: &Context) -> Option<JsonValue> {
                if let Some(JsonValue::Array(list)) = self.0.apply(value, 0) {
                    let list: Vec<_> = list
                        .into_iter()
                        .enumerate()
                        .map(|(i, v)| {
                            let mut mp = IndexMap::with_capacity(2);
                            mp.insert("value".into(), v.clone());
                            mp.insert("index".into(), i.into());
                            mp.into()
                        })
                        .collect();
                    Some(list.into())
                } else {
                    None
                }
            }
        }
        Rc::new(Impl(args))
    })
        .add_description_line(
            "Map a list into a new list where each element in the new list is an object with two elements:"
        )
        .add_description_line("* `index` with the index of the element in the list")
        .add_description_line("* `value` with the element in the original list")
        .add_description_line("Can be used later for filters or map based on index.")
        .add_description_line("If the first argument is not a list will return nothing.")
        .add_example(
            Example::new()
                .add_argument("[false, null, 10, {}]")
                .expected_output(
                    r#"[{"value": false, "index": 0}, {"value": null, "index": 1}, {"value": 10, "index": 2}, {"value": {}, "index": 3}]"#
                )
        )
        .add_example(Example::new().add_argument("{}"))
}
