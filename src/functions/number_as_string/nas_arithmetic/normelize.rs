use std::rc::Rc;

use crate::{
    functions::number_as_string::to_big_decimal::BigDecimalConvert,
    functions_definitions::{Arguments, Example, FunctionDefinitions},
    json_value::JsonValue,
    processor::Context,
    selection::Get,
};

pub fn get() -> FunctionDefinitions {
    FunctionDefinitions::new("\"||\"", 1, 1, |args| {
        struct Impl(Vec<Rc<dyn Get>>);
        impl Get for Impl {
            fn get(&self, value: &Context) -> Option<JsonValue> {
                self.0.apply(value, 0)
                .to_big_decimal()
                .map(|number| number.into())
            }
        }
        Rc::new(Impl(args))
    })
    .add_alias("nas_normelize")
    .add_description_line("If the argument is string as number, return it in a normelize form (to allow a constent uniquness check).")
    .add_example(Example::new().add_argument("\"1000000\"").expected_output("\"1000000\""))
    .add_example(Example::new().add_argument("\"0.00000000005\"").expected_output("\"5E-11\""))
    .add_example(Example::new().add_argument("\"000000000005\"").expected_output("\"5\""))
    .add_example(Example::new().add_argument("\"00000000000.5\"").expected_output("\"0.5\""))
    .add_example(Example::new().add_argument("\"100e100\"").expected_output("\"1e+102\""))
    .add_example(Example::new().add_argument("10"))
}
