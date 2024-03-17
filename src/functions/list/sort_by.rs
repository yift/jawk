use std::rc::Rc;

use crate::{
    functions_definitions::{Arguments, Example, FunctionDefinitions},
    json_value::JsonValue,
    processor::Context,
    selection::Get,
};

pub fn get() -> FunctionDefinitions {
    FunctionDefinitions::new("sort_by", 2, 2, |args| {
        struct Impl(Vec<Rc<dyn Get>>);
        impl Get for Impl {
            fn get(&self, value: &Context) -> Option<JsonValue> {
                match self.0.apply(value, 0) {
                    Some(JsonValue::Array(list)) => {
                        let mut list: Vec<JsonValue> = list.clone();
                        list.sort_by(|v1, v2| {
                            let v1 = value.with_inupt(v1.clone());
                            let v1 = self.0.apply(&v1, 1);
                            let v2 = value.with_inupt(v2.clone());
                            let v2 = self.0.apply(&v2, 1);
                            v1.cmp(&v2)
                        });

                        Some(list.into())
                    }
                    _ => None,
                }
            }
        }
        Rc::new(Impl(args))
    })
        .add_alias("order_by")
        .add_description_line("Filter a list.")
        .add_description_line(
            "If the first argument is a list, return list sorted by the second argument."
        )
        .add_example(
            Example::new()
                .add_argument("[\"12345\", \"5\", \"23\", \"abc\", \"-1-2\", \"\"]")
                .add_argument("(len .)")
                .expected_output("[\"\",\"5\",\"23\",\"abc\",\"-1-2\",\"12345\"]")
                .explain(
                    "it sort the elements by their length, so the empty string (length zero) is the first."
                )
        )
        .add_example(Example::new().add_argument("true").add_argument("(len .)"))
        .add_example(
            Example::new()
                .add_argument("[\"12345\", \"\", 10]")
                .add_argument("(len .)")
                .expected_output("[10, \"\",\"12345\"]")
                .explain("the number `10` has no length, so it will be the first.")
        )
}
