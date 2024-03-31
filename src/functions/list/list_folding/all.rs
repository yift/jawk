use std::rc::Rc;

use crate::{
    functions_definitions::{Arguments, Example, FunctionDefinitions},
    json_value::JsonValue,
    processor::Context,
    selection::Get,
};

pub fn get() -> FunctionDefinitions {
    FunctionDefinitions::new("all", 1, 1, |args| {
        struct Impl(Vec<Rc<dyn Get>>);
        impl Get for Impl {
            fn get(&self, value: &Context) -> Option<JsonValue> {
                match self.0.apply(value, 0) {
                    Some(JsonValue::Array(list)) => {
                        if list.is_empty() {
                            return Some(false.into());
                        }
                        for t in list {
                            if t != JsonValue::Boolean(true) {
                                return Some(false.into());
                            }
                        }
                        Some(true.into())
                    }
                    _ => None,
                }
            }
        }
        Rc::new(Impl(args))
    })
    .add_description_line("Check if all the items in a list are true.")
    .add_description_line("Will return false if the list is empty .")
    .add_example(
        Example::new()
            .add_argument("[1, 5, false, 1.1]")
            .expected_output("false"),
    )
    .add_example(
        Example::new()
            .add_argument("[true, true, 1, true, true]")
            .expected_output("false"),
    )
    .add_example(
        Example::new()
            .add_argument("(map (range 4) (.= 2))")
            .expected_output("false")
            .explain("not all the numbers between 0 and 4 are 2."),
    )
    .add_example(
        Example::new()
            .add_argument("(map (range 4) (.< 10))")
            .expected_output("true")
            .explain("all the numbers between 0 and 4 less than 10."),
    )
    .add_example(
        Example::new()
            .add_argument("[true, true, false, true, true]")
            .expected_output("false"),
    )
    .add_example(
        Example::new()
            .add_argument("[true, true, true, true]")
            .expected_output("true"),
    )
    .add_example(Example::new().add_argument("[]").expected_output("false"))
    .add_example(Example::new().add_argument("{}"))
}
