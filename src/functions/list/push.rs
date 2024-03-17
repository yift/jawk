use std::rc::Rc;

use crate::{
    functions_definitions::{Arguments, Example, FunctionDefinitions},
    json_value::JsonValue,
    processor::Context,
    selection::Get,
};

pub fn get() -> FunctionDefinitions {
    FunctionDefinitions::new("push", 2, usize::MAX, |args| {
        struct Impl(Vec<Rc<dyn Get>>);
        impl Get for Impl {
            fn get(&self, value: &Context) -> Option<JsonValue> {
                match self.0.apply(value, 0) {
                    Some(JsonValue::Array(lst)) => {
                        let mut new_list = lst.clone();
                        for index in 1..self.0.len() {
                            if let Some(val) = self.0.apply(value, index) {
                                new_list.push(val);
                            }
                        }
                        Some(new_list.into())
                    }
                    _ => None,
                }
            }
        }
        Rc::new(Impl(args))
    })
        .add_alias("push_back")
        .add_description_line("Add items to a list.")
        .add_description_line(
            "If the first argument is a list, will iterate over all the other arguments and add them to the list if they exists."
        )
        .add_example(
            Example::new()
                .add_argument("[]")
                .add_argument("1")
                .add_argument("2")
                .add_argument("3")
                .add_argument("4")
                .expected_output("[1, 2, 3, 4]")
        )
        .add_example(
            Example::new()
                .add_argument("[\"a\"]")
                .add_argument("\"b\"")
                .expected_output("[\"a\", \"b\"]")
        )
        .add_example(
            Example::new()
                .add_argument("[\"a\"]")
                .add_argument("(push 1 1)")
                .expected_output("[\"a\"]")
        )
        .add_example(Example::new().add_argument("-4").add_argument("-4"))
}
