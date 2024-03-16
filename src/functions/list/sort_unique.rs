use std::rc::Rc;

use crate::{
    functions_definitions::{Arguments, Example, FunctionDefinitions},
    json_value::JsonValue,
    processor::Context,
    selection::Get,
};

pub fn get() -> FunctionDefinitions {
    FunctionDefinitions::new("sort_unique", 1, 1, |args| {
        struct Impl(Vec<Rc<dyn Get>>);
        impl Get for Impl {
            fn get(&self, value: &Context) -> Option<JsonValue> {
                match self.0.apply(value, 0) {
                    Some(JsonValue::Array(list)) => {
                        let mut list: Vec<JsonValue> = list.clone();
                        list.sort_unstable();
                        list.dedup();

                        Some(list.into())
                    }
                    _ => None,
                }
            }
        }
        Rc::new(Impl(args))
    })
    .add_alias("order_unique")
    .add_description_line("Sort a list and remove duplicates.")
    .add_description_line("If the first argument is a list, return list sorted without duplicates.")
    .add_example(
        Example::new()
            .add_argument("[1, -2, 3.01, 3.05, -544, 100]")
            .expected_output("[-544, -2, 1, 3.01, 3.05, 100]"),
    )
    .add_example(
        Example::new()
            .add_argument("[1, 2, 3, 2, 3, 3]")
            .expected_output("[1, 2, 3]"),
    )
    .add_example(
        Example::new()
            .add_argument("[null, true, false, {}, [1, 2, 3], \"abc\", \"cde\", {\"key\": 12}]")
            .expected_output("[null, false, true, \"abc\", \"cde\", {}, {\"key\": 12}, [1, 2, 3]]"),
    )
    .add_example(Example::new().add_argument("344"))
}
