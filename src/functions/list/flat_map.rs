use std::rc::Rc;

use crate::{
    functions_definitions::{Arguments, Example, FunctionDefinitions},
    json_value::JsonValue,
    processor::Context,
    selection::Get,
};

pub fn get() -> FunctionDefinitions {
    FunctionDefinitions::new("flat_map", 2, 2, |args| {
                struct Impl(Vec<Rc<dyn Get>>);
                impl Get for Impl {
                    fn get(&self, value: &Context) -> Option<JsonValue> {
                        match self.0.apply(value, 0) {
                            Some(JsonValue::Array(list)) => {
                                let list: Vec<_> = list
                                    .into_iter()
                                    .filter_map(|v| {
                                        let v = value.with_inupt(v);
                                        if let Some(JsonValue::Array(list)) = self.0.apply(&v, 1) {
                                            Some(list)
                                        } else {
                                            None
                                        }
                                    })
                                    .flatten()
                                    .collect();

                                Some(list.into())
                            }
                            _ => None,
                        }
                    }
                }
                Rc::new(Impl(args))
            })
                .add_description_line("Flat map a list into a new list using a function.")
                .add_description_line(
                    "If the first argument is a list, activate the second argument on each item, and if that returns a list, add all the items to a new list."
                )
                .add_example(
                    Example::new()
                        .add_argument("[\"a,b,c\", \"d,e\", 4, \"g\"]")
                        .add_argument("(split . \",\")")
                        .expected_output("[\"a\", \"b\", \"c\", \"d\", \"e\", \"g\"]")
                        .explain(
                            "it will split each element by the comma, and return a list of all those lists."
                        )
                )
                .add_example(
                    Example::new()
                        .add_argument("[1, 2, 3, 4]")
                        .add_argument("(.len)")
                        .expected_output("[]")
                )
                .add_example(Example::new().add_argument("{}").add_argument("true"))
}
