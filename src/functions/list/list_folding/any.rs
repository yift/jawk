use std::rc::Rc;

use crate::{
    functions_definitions::{Arguments, Example, FunctionDefinitions},
    json_value::JsonValue,
    processor::Context,
    selection::Get,
};

pub fn get() -> FunctionDefinitions {
    FunctionDefinitions::new("any", 1, 1, |args| {
        struct Impl(Vec<Rc<dyn Get>>);
        impl Get for Impl {
            fn get(&self, value: &Context) -> Option<JsonValue> {
                match self.0.apply(value, 0) {
                    Some(JsonValue::Array(list)) => {
                        for t in list {
                            if t == JsonValue::Boolean(true) {
                                return Some(true.into());
                            }
                        }
                        Some(false.into())
                    }
                    _ => None,
                }
            }
        }
        Rc::new(Impl(args))
    })
    .add_description_line("Check if any of item in a list is ture.")
    .add_example(
        Example::new()
            .add_argument("[1, 5, false, 1.1]")
            .expected_output("false"),
    )
    .add_example(
        Example::new()
            .add_argument("(map (range 4) (.= 2))")
            .expected_output("true")
            .explain("there is 2 in the list of numbers from 0 to 4."),
    )
    .add_example(
        Example::new()
            .add_argument("(map (range 4) (.= 12))")
            .expected_output("false")
            .explain("there is no 12 in the list of numbers from 0 to 4."),
    )
    .add_example(Example::new().add_argument("[]").expected_output("false"))
    .add_example(
        Example::new()
            .add_argument("[1, 2, true, false, 4]")
            .expected_output("true"),
    )
    .add_example(Example::new().add_argument("{}"))
}
