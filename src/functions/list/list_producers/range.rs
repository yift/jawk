use std::rc::Rc;

use crate::{
    functions_definitions::{Arguments, Example, FunctionDefinitions},
    json_value::JsonValue,
    processor::Context,
    selection::Get,
};

pub fn get() -> FunctionDefinitions {
    FunctionDefinitions::new("range", 1, 1, |args| {
        struct Impl(Vec<Rc<dyn Get>>);
        impl Get for Impl {
            fn get(&self, value: &Context) -> Option<JsonValue> {
                match self.0.apply(value, 0) {
                    Some(JsonValue::Number(n)) => {
                        if let Ok(size) = TryInto::<usize>::try_into(n) {
                            let mut vec = vec![];
                            for i in 0..size {
                                vec.push(i.into());
                            }
                            Some(vec.into())
                        } else {
                            None
                        }
                    }
                    _ => None,
                }
            }
        }
        Rc::new(Impl(args))
    })
    .add_description_line("Create a new list with items from 0 to the second argument.")
    .add_description_line("If the second argument is not a positive integer, return nothing.")
    .add_description_line("Be careful not to use large numbers.")
    .add_example(
        Example::new()
            .add_argument("4")
            .expected_output("[0, 1, 2, 3]"),
    )
    .add_example(Example::new().add_argument("-4"))
    .add_example(Example::new().add_argument("[1, 2, 3, 4]"))
}
