use std::rc::Rc;

use crate::{
    functions_definitions::{Arguments, Example, FunctionDefinitions},
    json_value::JsonValue,
    processor::Context,
    selection::Get,
};

pub fn get() -> FunctionDefinitions {
    FunctionDefinitions::new("set", 3, 3, |args| {
                struct Impl(Vec<Rc<dyn Get>>);
                impl Get for Impl {
                    fn get(&self, context: &Context) -> Option<JsonValue> {
                        if
                            let (Some(JsonValue::String(name)), Some(value)) = (
                                self.0.apply(context, 0),
                                self.0.apply(context, 1),
                            )
                        {
                            let new_context = context.with_variable(name, value);
                            self.0.apply(&new_context, 2)
                        } else {
                            None
                        }
                    }
                }
                Rc::new(Impl(args))
            })
                .add_description_line(
                    "Set a variable. The first argument should be the variable name, the second one should be the value and the third"
                )
                .add_description_line(" one should be the function to run with the variable.")
                .add_example(
                    Example::new()
                        .add_argument("\"foo\"")
                        .add_argument("{\"key\": 100}")
                        .add_argument("(get (: \"foo\") \"key\" )")
                        .expected_output("100")
                )
                .add_example(
                    Example::new()
                        .add_argument("\"foo\"")
                        .add_argument("{\"key\": 100}")
                        .add_argument("(get :foo \"key\" )")
                        .expected_output("100")
                )
                .add_example(
                    Example::new()
                        .add_argument("12")
                        .add_argument("{\"key\": 100}")
                        .add_argument("(get (: \"foo\") \"key\" )")
                )
                .add_example(
                    Example::new()
                        .add_argument("\"foo\"")
                        .add_argument("{\"key\": 100}")
                        .add_argument("(get (: 12) \"key\" )")
                )
}
