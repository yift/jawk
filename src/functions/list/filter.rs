use std::rc::Rc;

use crate::{
    functions_definitions::{Arguments, Example, FunctionDefinitions},
    json_value::JsonValue,
    processor::Context,
    selection::Get,
};

pub fn get() -> FunctionDefinitions {
    FunctionDefinitions::new("filter", 2, 2, |args| {
                struct Impl(Vec<Rc<dyn Get>>);
                impl Get for Impl {
                    fn get(&self, value: &Context) -> Option<JsonValue> {
                        match self.0.apply(value, 0) {
                            Some(JsonValue::Array(list)) => {
                                let list: Vec<_> = list
                                    .into_iter()

                                    .filter(|v| {
                                        let v = value.with_inupt(v.clone());
                                        matches!(
                                            self.0.apply(&v, 1),
                                            Some(JsonValue::Boolean(true))
                                        )
                                    })
                                    .collect();
                                Some(list.into())
                            }
                            _ => None,
                        }
                    }
                }
                Rc::new(Impl(args))
            })
                .add_description_line("Filter a list.")
                .add_description_line(
                    "If the first argument is a list, return all the values for which the second argument is a list."
                )
                .add_example(
                    Example::new()
                        .add_argument("[1, 2, 3, 4]")
                        .add_argument("true")
                        .expected_output("[1, 2, 3, 4]")
                )
                .add_example(
                    Example::new()
                        .add_argument("[1, 2, 3, 4]")
                        .add_argument("null")
                        .expected_output("[]")
                )
                .add_example(
                    Example::new()
                        .add_argument("[1, 2, 3, 4, \"one\", null]")
                        .add_argument("(string? .)")
                        .expected_output("[\"one\"]")
                        .explain("only the value `\"one\"` is a string.")
                )
                .add_example(
                    Example::new()
                        .add_argument(".")
                        .add_argument("(.number?)")
                        .expected_output("[1, 2, 4]")
                        .input("[1, 2, null, \"a\", 4]")
                )
                .add_example(Example::new().add_argument("{}").add_argument("true"))
}
