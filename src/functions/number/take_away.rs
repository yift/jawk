use std::rc::Rc;

use crate::{
    functions_definitions::{Arguments, Example, FunctionDefinitions},
    json_value::JsonValue,
    processor::Context,
    selection::Get,
};

pub fn get() -> FunctionDefinitions {
    FunctionDefinitions::new("-", 1, 2, |args| {
        struct Impl(Vec<Rc<dyn Get>>);
        impl Get for Impl {
            fn get(&self, value: &Context) -> Option<JsonValue> {
                if let (Some(JsonValue::Number(num1)), Some(JsonValue::Number(num2))) = if self.0.len() == 1 {
                    (Some(0.into()), self.0.apply(value, 0))
                } else {
                    (self.0.apply(value, 0), self.0.apply(value, 1))
                } {
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
    .add_alias("minus")
    .add_alias("take_away")
    .add_alias("substruct")
    .add_description_line("If there are two numeric arguments, substract the second argument from the first one if both are number.")
    .add_description_line("If there is one numeric arguments, return the negative of that number.")
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
    .add_example(
        Example::new()
            .add_argument("10")
            .expected_output("-10"),
    )
    .add_example(
        Example::new()
            .add_argument("-11.3")
            .expected_output("11.3"),
    )
    .add_example(Example::new().add_argument("10").add_argument("\"text\""))
    .add_example(Example::new().add_argument("null").add_argument("6"))
    .add_example(Example::new().add_argument("{}"))
}
