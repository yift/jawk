use std::rc::Rc;

use crate::{
    functions_definitions::{Arguments, Example, FunctionDefinitions},
    json_value::JsonValue,
    processor::Context,
    selection::Get,
};

pub fn get() -> FunctionDefinitions {
    FunctionDefinitions::new("@", 1, 1, |args| {
        struct Impl(Vec<Rc<dyn Get>>);
        impl Get for Impl {
            fn get(&self, context: &Context) -> Option<JsonValue> {
                if let Some(JsonValue::String(name)) = self.0.apply(context, 0) {
                    context.get_definition(&name).and_then(|g| g.get(context))
                } else {
                    None
                }
            }
        }
        Rc::new(Impl(args))
    })
    .add_description_line("Return the value of a named variable. See define for examples.")
    .add_example(Example::new().add_argument("\"foo\""))
}
