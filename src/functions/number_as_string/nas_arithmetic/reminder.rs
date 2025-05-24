use std::rc::Rc;

use bigdecimal::{BigDecimal, Zero};

use crate::{
    functions::number_as_string::to_big_decimal::BigDecimalConvert,
    functions_definitions::{Arguments, Example, FunctionDefinitions},
    json_value::JsonValue,
    processor::Context,
    selection::Get,
};

pub fn get() -> FunctionDefinitions {
    FunctionDefinitions::new("\"%\"", 2, 2, |args| {
        struct Impl(Vec<Rc<dyn Get>>);
        impl Get for Impl {
            fn get(&self, value: &Context) -> Option<JsonValue> {
                if
                let (Some(num1), Some(num2)) = (
                    self.0.apply(value, 0).to_big_decimal(),
                    self.0.apply(value, 1).to_big_decimal(),
                )
                {
                    if num2 == BigDecimal::zero() {
                        None
                    } else {
                        Some((num1 % num2).into())
                    }
                } else {
                    None
                }
            }
        }
        Rc::new(Impl(args))
    })
        .add_alias("nas_mod")
        .add_alias("nas_modulo")
        .add_alias("nas_remider")
        .add_description_line(
            "Find the reminder of the division of the firs argument by the second argument if both are strings as number. If the second argument is \"0\" will return nothing"
        )
        .add_example(
            Example::new()
            .add_argument("\"5\"")
            .add_argument("\"3\"")
            .expected_output("\"2\"")
        )
        .add_example(
            Example::new()
            .add_argument("\"7\"")
            .add_argument("\"2\"")
            .expected_output("\"1\"")
        )
        .add_example(
            Example::new()
            .add_argument("\"7\"")
            .add_argument("\"0\"")
            .explain("Can not divide by zero.")
        )
        .add_example(
            Example::new()
            .add_argument("\"10\"")
            .add_argument("\"7.5\"")
            .expected_output("\"2.5\"")
        )
        .add_example(
            Example::new()
            .add_argument("\"10\"")
            .add_argument("\"-7\"")
            .expected_output("\"3\"")
        )
        .add_example(
            Example::new()
            .add_argument("\"-10\"")
            .add_argument("\"7\"")
            .expected_output("\"-3\"")
        )
        .add_example(
            Example::new()
            .add_argument("\"-10\"")
            .add_argument("\"-7\"")
            .expected_output("\"-3\"")
        )
        .add_example(
            Example::new()
            .add_argument("\"7\"")
            .add_argument("false")
        )
        .add_example(
            Example::new()
            .add_argument("[1]")
            .add_argument("\"4\"")
        )
}
