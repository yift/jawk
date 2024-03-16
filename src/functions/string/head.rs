use std::rc::Rc;

use crate::processor::Context;

use crate::{
    functions_definitions::{Arguments, Example, FunctionDefinitions},
    json_value::JsonValue,
    selection::Get,
};

pub fn get() -> FunctionDefinitions {
    FunctionDefinitions::new("head", 2, 2, |args| {
                struct Impl(Vec<Rc<dyn Get>>);
                impl Get for Impl {
                    fn get(&self, value: &Context) -> Option<JsonValue> {
                        if
                            let (Some(JsonValue::String(str)), Some(JsonValue::Number(index))) = (
                                self.0.apply(value, 0),
                                self.0.apply(value, 1),
                            )
                        {
                            if let Ok(index) = TryInto::<usize>::try_into(index) {
                                if str.len() < index {
                                    Some(str.into())
                                } else {
                                    let head = str[..index].to_string();
                                    Some(head.into())
                                }
                            } else {
                                None
                            }
                        } else {
                            None
                        }
                    }
                }
                Rc::new(Impl(args))
            })
                .add_description_line("Extract a string header.")
                .add_description_line(
                    "If the first argument is a string and the second argument is a positive integer, the returned value will be a string with the beggining of the first argument."
                )
                .add_description_line("See also `take`.")
                .add_example(
                    Example::new()
                        .add_argument("\"test-123\"")
                        .add_argument("4")
                        .expected_output("\"test\"")
                )
                .add_example(
                    Example::new()
                        .add_argument("\"test-123\"")
                        .add_argument("20")
                        .expected_output("\"test-123\"")
                )
                .add_example(Example::new().add_argument("20").add_argument("20"))
                .add_example(Example::new().add_argument("\"20\"").add_argument("-5"))
}
