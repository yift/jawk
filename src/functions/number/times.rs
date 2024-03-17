use std::rc::Rc;

use crate::{
    functions_definitions::{Example, FunctionDefinitions},
    json_value::JsonValue,
    processor::Context,
    selection::Get,
};

pub fn get() -> FunctionDefinitions {
    FunctionDefinitions::new("*", 2, usize::MAX, |args| {
        struct Impl(Vec<Rc<dyn Get>>);
        impl Get for Impl {
            fn get(&self, value: &Context) -> Option<JsonValue> {
                let mut sum = 1.0;
                for s in &self.0 {
                    if let Some(JsonValue::Number(num)) = s.get(value) {
                        let num: f64 = num.into();
                        sum *= num;
                    } else {
                        return None;
                    }
                }
                Some(sum.into())
            }
        }
        Rc::new(Impl(args))
    })
    .add_alias("times")
    .add_alias("multiple")
    .add_description_line("If all the arguments are number, multiply them.")
    .add_example(
        Example::new()
            .add_argument("2")
            .add_argument("3")
            .expected_output("6"),
    )
    .add_example(
        Example::new()
            .add_argument("2")
            .add_argument("15")
            .add_argument("0.1")
            .expected_output("3"),
    )
    .add_example(Example::new().add_argument("2").add_argument("true"))
}
