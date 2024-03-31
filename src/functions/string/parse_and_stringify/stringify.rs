use std::rc::Rc;

use crate::{
    functions_definitions::{Arguments, Example, FunctionDefinitions},
    json_value::JsonValue,
    processor::Context,
    selection::Get,
};

pub fn get() -> FunctionDefinitions {
    FunctionDefinitions::new("stringify", 1, 1, |args| {
        struct Impl(Vec<Rc<dyn Get>>);
        impl Get for Impl {
            fn get(&self, value: &Context) -> Option<JsonValue> {
                self.0.apply(value, 0).map(|val| format!("{val}").into())
            }
        }
        Rc::new(Impl(args))
    })
    .add_description_line("Return the JSON represantation of the object.")
    .add_example(
        Example::new()
            .add_argument("true")
            .expected_output("\"true\""),
    )
    .add_example(
        Example::new()
            .add_argument("1e2")
            .expected_output("\"100\""),
    )
    .add_example(
        Example::new()
            .add_argument("{\"key\": [1, 2, \"3\"]}")
            .expected_output("\"{\\\"key\\\": [1, 2, \\\"3\\\"]}\""),
    )
    .add_example(
        Example::new()
            .add_argument("(+ 10 20)")
            .expected_output("\"30\""),
    )
}
