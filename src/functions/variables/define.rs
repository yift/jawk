use std::rc::Rc;

use crate::{
    functions_definitions::{Arguments, Example, FunctionDefinitions},
    json_value::JsonValue,
    processor::Context,
    selection::Get,
};

pub fn get() -> FunctionDefinitions {
    FunctionDefinitions::new("define", 3, 3, |args| {
                struct Impl(Vec<Rc<dyn Get>>);
                impl Get for Impl {
                    fn get(&self, context: &Context) -> Option<JsonValue> {
                        if
                            let (Some(JsonValue::String(name)), Some(definition)) = (
                                self.0.apply(context, 0),
                                self.0.get(1),
                            )
                        {
                            let new_context = context.with_definition(name, definition);
                            self.0.apply(&new_context, 2)
                        } else {
                            None
                        }
                    }
                }
                Rc::new(Impl(args))
            })
                .add_alias("macro")
                .add_alias("def")
                .add_alias("#")
                .add_description_line(
                    "Define a new macro definition. The first argument should be the macro definition name, the second one should be macro and the third"
                )
                .add_description_line(" one should be the function to run with the macro.")
                .add_example(
                    Example::new()
                        .add_argument("\"add-1\"")
                        .add_argument("(.+ 1)")
                        .add_argument("(map [1, 2, 3] (@ \"add-1\"))")
                        .expected_output("[2, 3, 4]")
                )
                .add_example(
                    Example::new()
                        .add_argument("\"add-1\"")
                        .add_argument("(.+ 1)")
                        .add_argument("(map [1, 2, 3] @add-1)")
                        .expected_output("[2, 3, 4]")
                )
                .add_example(
                    Example::new()
                        .add_argument("\"three\"")
                        .add_argument("3")
                        .add_argument("(+ 1 @three)")
                        .expected_output("4")
                )
                .add_example(
                    Example::new()
                        .add_argument("12")
                        .add_argument("{\"key\": 100}")
                        .add_argument("(get (: \"foo\") \"key\" )")
                )
}
