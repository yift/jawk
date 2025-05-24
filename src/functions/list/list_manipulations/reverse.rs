use std::rc::Rc;

use crate::{
    functions_definitions::{Arguments, Example, FunctionDefinitions},
    json_value::JsonValue,
    processor::Context,
    selection::Get,
};

pub fn get() -> FunctionDefinitions {
    FunctionDefinitions::new("reverse", 1, 1, |args| {
        struct Impl(Vec<Rc<dyn Get>>);
        impl Get for Impl {
            fn get(&self, value: &Context) -> Option<JsonValue> {
                match self.0.apply(value, 0) {
                    Some(JsonValue::Array(lst)) => {
                        let mut new_list = Vec::with_capacity(lst.len());
                        for val in lst.iter().rev() {
                            new_list.push(val.clone());
                        }
                        Some(new_list.into())
                    }
                    _ => None,
                }
            }
        }
        Rc::new(Impl(args))
    })
        .add_description_line("Reveres the order of a list.")
        .add_description_line(
            "If the first argument is a list, will iterate over all the other arguments and add them to the list if they exists."
        )
        .add_example(
            Example::new().add_argument("[1, 2, 3, 4]").expected_output("[4, 3, 2, 1]")
        )
        .add_example(Example::new().add_argument("1"))
}
