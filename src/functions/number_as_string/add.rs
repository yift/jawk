use std::rc::Rc;

use bigdecimal::{BigDecimal, Zero};

use crate::{
    functions::number_as_string::to_big_decimal::BigDecimalConvert,
    functions_definitions::{Example, FunctionDefinitions},
    json_value::JsonValue,
    processor::Context,
    selection::Get,
};

pub fn get() -> FunctionDefinitions {
    FunctionDefinitions::new("\"+\"", 2, usize::MAX, |args| {
        struct Impl(Vec<Rc<dyn Get>>);
        impl Get for Impl {
            fn get(&self, value: &Context) -> Option<JsonValue> {
                let mut sum = BigDecimal::zero();
                for s in &self.0 {
                    if let Some(number) = s.get(value).to_big_decimal() {
                        sum += number;
                    } else {
                        return None;
                    }
                }
                Some(sum.into())
            }
        }
        Rc::new(Impl(args))
    })
    .add_alias("nas_add")
    .add_alias("nas_plus")
    .add_description_line("If all the arguments are numbers as string number, add them.")
    .add_example(
        Example::new()
            .add_argument("\"1\"")
            .add_argument("\"3\"")
            .expected_output("\"4\""),
    )
    .add_example(
        Example::new()
            .add_argument("\"1\"")
            .add_argument("\"10\"")
            .add_argument("\"-4.1\"")
            .add_argument("\"0.1\"")
            .expected_output("\"7\""),
    )
    .add_example(
        Example::new()
            .add_argument("\"1e999\"")
            .add_argument("\"1e999\"")
            .add_argument("\"4e999\"")
            .expected_output("\"6E+999\""),
    )
    .add_example(
        Example::new()
            .add_argument("\"1\"")
            .add_argument("\"3\"")
            .add_argument("\"text\""),
    )
    .add_example(
        Example::new()
            .add_argument("\"1\"")
            .add_argument("\"3\"")
            .add_argument("[]"),
    )
}
