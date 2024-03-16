use std::rc::Rc;

use crate::{
    functions_definitions::{Arguments, Example, FunctionDefinitions},
    json_value::JsonValue,
    processor::Context,
    selection::Get,
};

pub fn get() -> FunctionDefinitions {
    FunctionDefinitions::new("-", 2, 2, |args| {
        struct Impl(Vec<Rc<dyn Get>>);
        impl Get for Impl {
            fn get(&self, value: &Context) -> Option<JsonValue> {
                if let (Some(JsonValue::Number(num1)), Some(JsonValue::Number(num2))) =
                    (self.0.apply(value, 0), self.0.apply(value, 1))
                {
                    let num1: f64 = num1.into();
                    let num2: f64 = num2.into();
                    Some((num1 - num2).into())
                } else {
                    None
                }
            }
        }
        Rc::new(Impl(args))
    })
    .add_alias("take_away")
    .add_alias("minus")
    .add_alias("substruct")
    .add_description_line("Substract the second argument from the first one if both are number.")
    .add_example(
        Example::new()
            .add_argument("100")
            .add_argument("3")
            .expected_output("97"),
    )
    .add_example(
        Example::new()
            .add_argument("10")
            .add_argument("3.2")
            .expected_output("6.8"),
    )
    .add_example(Example::new().add_argument("10").add_argument("\"text\""))
    .add_example(Example::new().add_argument("null").add_argument("6"))
}
