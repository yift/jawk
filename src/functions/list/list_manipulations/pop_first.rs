use std::rc::Rc;

use crate::{
    functions_definitions::{Arguments, Example, FunctionDefinitions},
    json_value::JsonValue,
    processor::Context,
    selection::Get,
};

pub fn get() -> FunctionDefinitions {
    FunctionDefinitions::new("pop_first", 1, 1, |args| {
        struct Impl(Vec<Rc<dyn Get>>);
        impl Get for Impl {
            fn get(&self, value: &Context) -> Option<JsonValue> {
                match self.0.apply(value, 0) {
                    Some(JsonValue::Array(lst)) => {
                        if lst.is_empty() {
                            Some(lst.into())
                        } else {
                            let new_len = lst.len() - 1;
                            let mut new_list = Vec::with_capacity(new_len);
                            for (i, val) in lst.iter().enumerate() {
                                if i != 0 {
                                    new_list.push(val.clone());
                                }
                            }
                            Some(new_list.into())
                        }
                    }
                    _ => None,
                }
            }
        }
        Rc::new(Impl(args))
    })
    .add_description_line("Pop the first item from a list.")
    .add_description_line(
        "If the argument is a list, will return the list without it's first argument.",
    )
    .add_example(
        Example::new()
            .add_argument("[1, 2, 3, 4]")
            .expected_output("[2, 3, 4]"),
    )
    .add_example(Example::new().add_argument("[]").expected_output("[]"))
    .add_example(Example::new().add_argument("false"))
}
