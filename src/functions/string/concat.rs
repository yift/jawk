use std::rc::Rc;

use crate::processor::Context;

use crate::{
    functions_definitions::{Example, FunctionDefinitions},
    json_value::JsonValue,
    selection::Get,
};

pub fn get() -> FunctionDefinitions {
    FunctionDefinitions::new("concat", 2, usize::MAX, |args| {
        struct Impl(Vec<Rc<dyn Get>>);
        impl Get for Impl {
            fn get(&self, value: &Context) -> Option<JsonValue> {
                let mut all = String::new();
                for s in &self.0 {
                    if let Some(JsonValue::String(str)) = s.get(value) {
                        all.push_str(str.as_str());
                    } else {
                        return None;
                    }
                }
                Some(all.into())
            }
        }
        Rc::new(Impl(args))
    })
    .add_description_line("Concat all string arguments.")
    .add_example(
        Example::new()
            .add_argument("\"one\"")
            .add_argument("\" \"")
            .add_argument("\"two\"")
            .expected_output("\"one two\""),
    )
    .add_example(
        Example::new()
            .add_argument("\"one\"")
            .add_argument("\" \"")
            .add_argument("2"),
    )
}
