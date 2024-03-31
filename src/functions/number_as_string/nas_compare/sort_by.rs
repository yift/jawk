use std::rc::Rc;

use crate::{
    functions::number_as_string::to_big_decimal::BigDecimalConvert,
    functions_definitions::{Arguments, Example, FunctionDefinitions},
    json_value::JsonValue,
    processor::Context,
    selection::Get,
};

pub fn get() -> FunctionDefinitions {
    FunctionDefinitions::new("\"sort_by\"", 2, 2, |args| {
        struct Impl(Vec<Rc<dyn Get>>);
        impl Get for Impl {
            fn get(&self, value: &Context) -> Option<JsonValue> {
                match self.0.apply(value, 0) {
                    Some(JsonValue::Array(list)) => {
                        let mut list: Vec<JsonValue> = list.clone();
                        list.sort_by(|v1, v2| {
                            let v1 = value.with_inupt(v1.clone());
                            let v1 = self.0.apply(&v1, 1).to_big_decimal();
                            let v2 = value.with_inupt(v2.clone());
                            let v2 = self.0.apply(&v2, 1).to_big_decimal();
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
    .add_alias("\"order_by\"")
    .add_alias("sort_by_nas")
    .add_alias("order_by_nas")
    .add_description_line("Sort a list using number as strings.")
        .add_description_line(
            "If the first argument is a list, return list sorted by the second argument, asuming it's a number as string."
        )
        .add_example(
            Example::new()
                .add_argument(r#"[
                    {"key": 0},
                    {"key": 1, "value": "1"},
                    {"key": 2, "value": "1E-100"},
                    {"key": 3, "value": "-1"},
                    {"key": 4, "value": "1E+100"},
                    {"key": 5, "value": "9999"},
                    {"key": 6, "value": "1000"},
                    {"key": 7, "value": "1.334E2"},
                    {"key": 8, "value": "1"}
                 ]"#)
                .add_argument(".value")
                .expected_output(r#"[
                    {"key": 0},
                    {"key": 3, "value": "-1"},
                    {"key": 2, "value": "1E-100"},
                    {"key": 1, "value": "1"},
                    {"key": 8, "value": "1"},
                    {"key": 7, "value": "1.334E2"},
                    {"key": 6, "value": "1000"},
                    {"key": 5, "value": "9999"},
                    {"key": 4, "value": "1E+100"}
                 ]"#)
                .explain(
                    "it sort the elements by their value as numbers."
                )
        )
        .add_example(Example::new().add_argument("true").add_argument("(len .)"))
}
