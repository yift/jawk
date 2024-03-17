use std::rc::Rc;

use crate::processor::Context;

use crate::{
    functions_definitions::{Arguments, Example, FunctionDefinitions},
    json_value::JsonValue,
    selection::Get,
};

pub fn get() -> FunctionDefinitions {
    FunctionDefinitions::new("split", 2, 2, |args| {
        struct Impl(Vec<Rc<dyn Get>>);
        impl Get for Impl {
            fn get(&self, value: &Context) -> Option<JsonValue> {
                if let (Some(JsonValue::String(str)), Some(JsonValue::String(splitter))) =
                    (self.0.apply(value, 0), self.0.apply(value, 1))
                {
                    Some(
                        str.split(splitter.as_str())
                            .map(|f| JsonValue::String(f.to_string()))
                            .collect::<Vec<_>>()
                            .into(),
                    )
                } else {
                    None
                }
            }
        }
        Rc::new(Impl(args))
    })
    .add_description_line("Split the string into array of strings.")
    .add_example(
        Example::new()
            .add_argument("\"one, two, three\"")
            .add_argument("\", \"")
            .expected_output("[\"one\", \"two\", \"three\"]"),
    )
    .add_example(
        Example::new()
            .add_argument("\"a|b|c\"")
            .add_argument("\"|\"")
            .expected_output("[\"a\", \"b\", \"c\"]"),
    )
}
