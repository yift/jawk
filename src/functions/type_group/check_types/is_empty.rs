use std::rc::Rc;

use crate::{
    functions_definitions::{Arguments, Example, FunctionDefinitions},
    json_value::JsonValue,
    processor::Context,
    selection::Get,
};

pub fn get() -> FunctionDefinitions {
    FunctionDefinitions::new("empty?", 1, 1, |args| {
        struct Impl(Vec<Rc<dyn Get>>);
        impl Get for Impl {
            fn get(&self, value: &Context) -> Option<JsonValue> {
                match self.0.apply(value, 0) {
                    Some(_) => Some(false.into()),
                    _ => Some(true.into()),
                }
            }
        }
        Rc::new(Impl(args))
    })
    .add_alias("nothing?")
    .add_description_line("return true if the argument is nothing.")
    .add_example(
        Example::new()
            .add_argument("\"one\"")
            .expected_output("false"),
    )
    .add_example(
        Example::new()
            .add_argument(".key")
            .expected_output("true")
            .input("{}"),
    )
}
