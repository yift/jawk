use std::rc::Rc;

use crate::{
    functions_definitions::{Example, FunctionDefinitions},
    json_value::JsonValue,
    processor::Context,
    selection::Get,
};

pub fn get() -> FunctionDefinitions {
    FunctionDefinitions::new("default", 1, usize::MAX, |args| {
        struct Impl(Vec<Rc<dyn Get>>);
        impl Get for Impl {
            fn get(&self, value: &Context) -> Option<JsonValue> {
                for e in &self.0 {
                    let val = e.get(value);
                    if val.is_some() {
                        return val;
                    }
                }
                None
            }
        }
        Rc::new(Impl(args))
    })
        .add_alias("defaults")
        .add_alias("or_else")
        .add_description_line("Get the first non empty value.")
        .add_example(
            Example::new()
                .add_argument("(get . 1)")
                .add_argument("(get . \"key-1\")")
                .add_argument("22")
                .expected_output("1")
                .input("{\"key-1\": 1, \"key-2\": false}")
                .explain(
                    "the first get will return nothing because the input is an object and get should have a string argument, the second get will return 1, which is not a nothing, so it will be the returned value."
                )
        )
        .add_example(
            Example::new()
                .add_argument("(get . 1)")
                .add_argument("(get . \"key-3\")")
                .add_argument("22")
                .expected_output("22")
                .input("{\"key-1\": 1, \"key-2\": false}")
                .explain(
                    "all the `get` will return nothing, so 22 is the first non nothing argument."
                )
        )
        .add_example(Example::new().add_argument("(.get 1)").add_argument("(.get 2)").input("100"))
}
