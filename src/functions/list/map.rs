use std::rc::Rc;

use crate::{
    functions_definitions::{Arguments, Example, FunctionDefinitions},
    json_value::JsonValue,
    processor::Context,
    selection::Get,
};

pub fn get() -> FunctionDefinitions {
    FunctionDefinitions::new("map", 2, 2, |args| {
        struct Impl(Vec<Rc<dyn Get>>);
        impl Get for Impl {
            fn get(&self, value: &Context) -> Option<JsonValue> {
                match self.0.apply(value, 0) {
                    Some(JsonValue::Array(list)) => {
                        let list: Vec<_> = list
                            .into_iter()
                            .filter_map(|v| {
                                let v = value.with_inupt(v);
                                self.0.apply(&v, 1)
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
        .add_description_line("Map a list into a new list using a function.")
        .add_description_line(
            "If the first argument is a list, activate the second argument on each item and collect into a new list."
        )
        .add_example(
            Example::new()
                .add_argument("[1, 2, 3, 4]")
                .add_argument("(+ . 4)")
                .expected_output("[5, 6, 7, 8]")
        )
        .add_example(
            Example::new()
                .add_argument("[1, 2, 3, 4]")
                .add_argument("(.len)")
                .expected_output("[]")
        )
        .add_example(
            Example::new()
                .add_argument("[1, 2, 3, \"4\"]")
                .add_argument("(* . 2)")
                .expected_output("[2, 4, 6]")
                .explain("`\"4\"` is a string and not a number, so it can't be multiple.")
        )
        .add_example(
            Example::new()
                .add_argument(".")
                .add_argument("(+ 2 .)")
                .expected_output("[3, 4, 6]")
                .input("[1, 2, null, \"a\", 4]")
        )
        .add_example(
            Example::new()
                .input("{\"list\": [1, 2, 3, 4], \"add\": 12}")
                .add_argument(".list")
                .add_argument("(+ ^.add .)")
                .expected_output("[13, 14, 15, 16]")
        )
        .add_example(
            Example::new()
                .input("[[1, 2, 3, 4], [1, 2, 3], [6, 7]]")
                .add_argument(".")
                .add_argument("(map . (+ (len ^^.) .))")
                .expected_output("[[4, 5, 6, 7], [4, 5, 6], [9, 10]]")
                .explain(
                    "it will add the length of the input list (i.e. 3) to each item in each list in that list."
                )
        )
        .add_example(Example::new().add_argument("{}").add_argument("true"))
}
