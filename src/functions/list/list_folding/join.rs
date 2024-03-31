use std::rc::Rc;

use crate::{
    functions_definitions::{Arguments, Example, FunctionDefinitions},
    json_value::JsonValue,
    processor::Context,
    selection::Get,
};

pub fn get() -> FunctionDefinitions {
    FunctionDefinitions::new("join", 1, 2, |args| {
        struct Impl(Vec<Rc<dyn Get>>);
        impl Get for Impl {
            fn get(&self, value: &Context) -> Option<JsonValue> {
                let sepetator = self
                    .0
                    .apply(value, 1)
                    .and_then(|f| TryInto::<String>::try_into(f).ok())
                    .unwrap_or(", ".into());
                match self.0.apply(value, 0) {
                    Some(JsonValue::Array(list)) => {
                        let mut str = String::new();
                        for t in list {
                            let t: Result<String, _> = t.try_into();
                            match t {
                                Ok(to_add) => {
                                    if !str.is_empty() {
                                        str.push_str(sepetator.as_str());
                                    }
                                    str.push_str(to_add.as_str());
                                }
                                Err(_) => {
                                    return None;
                                }
                            }
                        }
                        Some(str.into())
                    }
                    _ => None,
                }
            }
        }
        Rc::new(Impl(args))
    })
    .add_description_line("Join all the items in the list into a String.")
    .add_description_line("If list have non string items, it will return nuthing.")
    .add_description_line(
        "If the second argument is ommited, the items will be seperated by comma.",
    )
    .add_example(
        Example::new()
            .add_argument("[\"one\", \"two\", \"three\"]")
            .expected_output("\"one, two, three\""),
    )
    .add_example(
        Example::new()
            .add_argument("[\"one\", \"two\", \"three\"]")
            .add_argument("\" ; \"")
            .expected_output("\"one ; two ; three\""),
    )
    .add_example(Example::new().add_argument("[\"one\", \"two\", 3]"))
}
