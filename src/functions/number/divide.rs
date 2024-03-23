use std::rc::Rc;

use crate::{
    functions_definitions::{Arguments, Example, FunctionDefinitions},
    json_value::JsonValue,
    processor::Context,
    selection::Get,
};

pub fn get() -> FunctionDefinitions {
    FunctionDefinitions::new("/", 2, 2, |args| {
        struct Impl(Vec<Rc<dyn Get>>);
        impl Get for Impl {
            fn get(&self, value: &Context) -> Option<JsonValue> {
                if
                let (Some(JsonValue::Number(num1)), Some(JsonValue::Number(num2))) = (
                    self.0.apply(value, 0),
                    self.0.apply(value, 1),
                )
                {
                    let num1: f64 = num1.into();
                    let num2: f64 = num2.into();
                    if num2 == 0.0 {
                        None
                    } else {
                        Some((num1 / num2).into())
                    }
                } else {
                    None
                }
            }
        }
        Rc::new(Impl(args))
    })
        .add_alias("divide")
        .add_description_line(
            "Divide the firs argument by the second argument. If the second argument is 0 will return nothing"
        )
        .add_example(
            Example::new().add_argument("100").add_argument("25").expected_output("4")
        )
        .add_example(
            Example::new().add_argument("7").add_argument("2").expected_output("3.5")
        )
        .add_example(
            Example::new().add_argument("7").add_argument("0")
        )
        .add_example(Example::new().add_argument("7").add_argument("[]"))
        .add_example(Example::new().add_argument("{}").add_argument("5"))
}
