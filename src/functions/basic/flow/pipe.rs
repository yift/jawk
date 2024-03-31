use std::{ops::Deref, rc::Rc};

use crate::{
    functions_definitions::{Example, FunctionDefinitions},
    json_value::JsonValue,
    processor::Context,
    selection::Get,
};

pub fn get() -> FunctionDefinitions {
    FunctionDefinitions::new("|", 2, usize::MAX, |args| {
        struct Impl(Vec<Rc<dyn Get>>);
        impl Get for Impl {
            fn get(&self, context: &Context) -> Option<JsonValue> {
                let mut context = context.with_inupt(context.input().deref().clone());
                for e in &self.0 {
                    if let Some(val) = e.get(&context) {
                        context = context.with_inupt(val);
                    } else {
                        return None;
                    }
                }
                Some(context.input().deref().clone())
            }
        }
        Rc::new(Impl(args))
    })
        .add_description_line("Pipe the output of one function to the next function.")
        .add_example(
            Example::new()
                .add_argument("(get . \"key\")")
                .add_argument("(get . 3)")
                .add_argument("(get . \"key-2\")")
                .expected_output("100")
                .input("{\"key\": [20, 40, 60, {\"key-2\": 100}]}")
                .explain(
                    "the first `get` will return the list in the input, the second one will return the fourth item in the list, and the last one will get the `key-2` element in that item."
                )
        )
        .add_example(
            Example::new()
                .input("[1, 2, 3, 4]")
                .add_argument("(get . 1)")
                .add_argument("(+ . 4)")
                .add_argument("(+ . 10)")
                .add_argument("(+ . (len ^^^))")
                .expected_output("20")
                .explain(
                    "The first get will return 2, the second one will add 4, then third one will add 10, and the last one will add the size of the original list, so `2 + 4 + 10 + 4 = 20`."
                )
        )
}
