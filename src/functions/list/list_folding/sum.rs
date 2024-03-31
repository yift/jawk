use std::rc::Rc;

use crate::{
    functions_definitions::{Arguments, Example, FunctionDefinitions},
    json_value::JsonValue,
    processor::Context,
    selection::Get,
};

pub fn get() -> FunctionDefinitions {
    FunctionDefinitions::new("sum", 1, 1, |args| {
        struct Impl(Vec<Rc<dyn Get>>);
        impl Get for Impl {
            fn get(&self, value: &Context) -> Option<JsonValue> {
                match self.0.apply(value, 0) {
                    Some(JsonValue::Array(list)) => {
                        let mut sum = 0.0;
                        for t in list {
                            let t: Result<f64, _> = t.try_into();
                            match t {
                                Ok(num) => {
                                    sum += num;
                                }
                                Err(_) => {
                                    return None;
                                }
                            }
                        }
                        Some(sum.into())
                    }
                    _ => None,
                }
            }
        }
        Rc::new(Impl(args))
    })
    .add_description_line("Sum all the items in the list.")
    .add_description_line("If list have non numeric items, it will return nuthing.")
    .add_description_line("One can `filter` with `number?` to ensure there is a result.")
    .add_example(
        Example::new()
            .add_argument("[1, 5, 1.1]")
            .expected_output("7.1"),
    )
    .add_example(Example::new().add_argument("[]").expected_output("0"))
    .add_example(Example::new().add_argument("[1, 5, 1.1, \"text\"]"))
}
