use std::rc::Rc;

use bigdecimal::{BigDecimal, One};

use crate::{
    functions::number_as_string::to_big_decimal::BigDecimalConvert,
    functions_definitions::{Example, FunctionDefinitions},
    json_value::JsonValue,
    processor::Context,
    selection::Get,
};

pub fn get() -> FunctionDefinitions {
    FunctionDefinitions::new("\"*\"", 2, usize::MAX, |args| {
        struct Impl(Vec<Rc<dyn Get>>);
        impl Get for Impl {
            fn get(&self, value: &Context) -> Option<JsonValue> {
                let mut sum = BigDecimal::one();
                for s in &self.0 {
                    if let Some(number) = s.get(value).to_big_decimal() {
                        sum *= number;
                    } else {
                        return None;
                    }
                }
                Some(sum.into())
            }
        }
        Rc::new(Impl(args))
    })
    .add_alias("nas_times")
    .add_alias("nas_multiple")
    .add_description_line("If all the arguments are numbers as string, multiply them.")
    .add_example(
        Example::new()
            .add_argument("\"2\"")
            .add_argument("\"3\"")
            .expected_output("\"6\""),
    )
    .add_example(
        Example::new()
            .add_argument("\"2\"")
            .add_argument("\"15\"")
            .add_argument("\"0.1\"")
            .expected_output("\"3\""),
    )
    .add_example(
        Example::new()
            .add_argument("\"1e100\"")
            .add_argument("\"1e80\"")
            .add_argument("\"1e300\"")
            .expected_output("\"1E+480\""),
    )
    .add_example(Example::new().add_argument("\"2\"").add_argument("true"))
}
