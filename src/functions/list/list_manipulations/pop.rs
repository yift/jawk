use std::rc::Rc;

use crate::{
    functions_definitions::{Arguments, Example, FunctionDefinitions},
    json_value::JsonValue,
    processor::Context,
    selection::Get,
};

pub fn get() -> FunctionDefinitions {
    FunctionDefinitions::new("pop", 1, 1, |args| {
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
                            for val in &lst {
                                if new_list.len() < new_len {
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
    .add_alias("pop_last")
    .add_description_line("Pop the last item from a list.")
    .add_description_line(
        "If the argument is a list, will return the list without it's last argument.",
    )
    .add_example(
        Example::new()
            .add_argument("[1, 2, 3, 4]")
            .expected_output("[1, 2, 3]"),
    )
    .add_example(Example::new().add_argument("[]").expected_output("[]"))
    .add_example(Example::new().add_argument("false"))
}
