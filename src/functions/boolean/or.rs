use std::rc::Rc;

use crate::{
    functions_definitions::{Example, FunctionDefinitions},
    json_value::JsonValue,
    processor::Context,
    selection::Get,
};

pub fn get() -> FunctionDefinitions {
    FunctionDefinitions::new("or", 2, usize::MAX, |args| {
                struct Impl(Vec<Rc<dyn Get>>);
                impl Get for Impl {
                    fn get(&self, value: &Context) -> Option<JsonValue> {
                        for s in &self.0 {
                            match s.get(value) {
                                Some(JsonValue::Boolean(false)) => {}
                                Some(JsonValue::Boolean(true)) => {
                                    return Some(true.into());
                                }
                                _ => {
                                    return None;
                                }
                            }
                        }
                        Some(false.into())
                    }
                }
                Rc::new(Impl(args))
            })
                .add_alias("||")
                .add_description_line(
                    "Return true if any of the arguments are true, nothing if there is a non boolean argument and false if all the arguments are false."
                )
                .add_example(
                    Example::new()
                        .add_argument("true")
                        .add_argument("true")
                        .add_argument("true")
                        .add_argument("true")
                        .expected_output("true")
                )
                .add_example(
                    Example::new()
                        .add_argument("false")
                        .add_argument("false")
                        .add_argument("false")
                        .add_argument("true")
                        .add_argument("true")
                        .add_argument("false")
                        .add_argument("true")
                        .expected_output("true")
                )
                .add_example(
                    Example::new()
                        .add_argument("false")
                        .add_argument("false")
                        .expected_output("false")
                )
                .add_example(
                    Example::new().add_argument("121").add_argument("true").add_argument("true")
                )
}
