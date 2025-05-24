use std::{process::Command, rc::Rc};

use crate::{
    functions_definitions::{Arguments, Example, FunctionDefinitions},
    json_value::JsonValue,
    processor::Context,
    selection::Get,
};

pub fn get() -> FunctionDefinitions {
    FunctionDefinitions::new("trigger", 1, usize::MAX, |vec| {
        struct Impl(Vec<Rc<dyn Get>>);
        impl Get for Impl {
            fn get(&self, context: &Context) -> Option<JsonValue> {
                let Some(JsonValue::String(command)) = self.0.apply(context, 0) else {
                    return None;
                };
                let mut command = Command::new(command);
                for arg in self.0.iter().skip(1) {
                    let Some(JsonValue::String(arg)) = arg.get(context) else {
                        return None;
                    };
                    command.arg(arg);
                }
                let Ok(child) = command.spawn() else {
                    return None;
                };
                Some(f64::from(child.id()).into())
            }
        }
        Rc::new(Impl(vec))
    })
    .add_description_line("Trigger an external process and return it's process ID.")
    .add_description_line("If all the arguments are strings run a process with that list.")
    .add_description_line("The result is the process ID:")
    .add_example(
        Example::new()
            .add_argument("\"echo\"")
            .add_argument("\"hello\"")
            .add_argument("\"world\"")
            .validate_output(|output| {
                if cfg!(windows) {
                    output.is_none()
                } else {
                    matches!(output, Some(JsonValue::Number(_)))
                }
            }),
    )
    .add_example(Example::new().add_argument("\"no such exec\""))
    .add_example(
        Example::new()
            .add_argument("23")
            .explain("23 is not a string"),
    )
    .add_example(
        Example::new()
            .add_argument("\"echo\"")
            .add_argument("23")
            .explain("23 is not a string"),
    )
}
