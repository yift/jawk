use std::rc::Rc;

use chrono::{TimeZone, Utc};

use crate::{
    functions_definitions::{Arguments, Example, FunctionDefinitions},
    json_value::JsonValue,
    processor::Context,
    selection::Get,
};

pub fn get() -> FunctionDefinitions {
    FunctionDefinitions::new("format_time", 2, 2, |args| {
        struct Impl(Vec<Rc<dyn Get>>);
        impl Get for Impl {
            fn get(&self, value: &Context) -> Option<JsonValue> {
                if let Some(JsonValue::Number(time)) = self.0.apply(value, 0) {
                    let since_epoch: f64 = time.into();
                    let seconds = since_epoch as i64;
                    let nsecs = ((since_epoch - (seconds as f64)) * 1e9) as u32;
                    if let Some(datetime) = Utc.timestamp_opt(seconds, nsecs).single() {
                        if let Some(JsonValue::String(format)) = self.0.apply(value, 1) {
                            Some(datetime.format(&format).to_string().into())
                        } else {
                            None
                        }
                    } else {
                        None
                    }
                } else {
                    None
                }
            }
        }
        Rc::new(Impl(args))
    })
    .add_description_line("Format a date/time into a string")
    .add_description_line("The first argument should be the number of seconds since epoch")
    .add_description_line("The second argument should be the format as string")
    .add_description_line(
        "See details in [https://docs.rs/chrono/latest/chrono/format/strftime/index.html].",
    )
    .add_example(
        Example::new()
            .add_argument("1701611515.3603675")
            .add_argument("122"),
    )
    .add_example(
        Example::new()
            .add_argument("{}")
            .add_argument("\"%a %b %e %T %Y - %H:%M:%S%.f\""),
    )
    .add_example(
        Example::new()
            .add_argument("1701611515.3603675")
            .expected_output("\"Sun Dec  3 13:51:55 2023 - 13:51:55.360367536\"")
            .add_argument("\"%a %b %e %T %Y - %H:%M:%S%.f\""),
    )
}
