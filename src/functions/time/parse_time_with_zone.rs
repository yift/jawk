use std::rc::Rc;

use chrono::DateTime;

use crate::{
    functions_definitions::{Arguments, Example, FunctionDefinitions},
    json_value::JsonValue,
    processor::Context,
    selection::Get,
};

pub fn get() -> FunctionDefinitions {
    FunctionDefinitions::new("parse_time_with_zone", 2, 2, |args| {
        struct Impl(Vec<Rc<dyn Get>>);
        impl Get for Impl {
            fn get(&self, value: &Context) -> Option<JsonValue> {
                if
                let (Some(JsonValue::String(str)), Some(JsonValue::String(format))) = (
                    self.0.apply(value, 0),
                    self.0.apply(value, 1),
                )
                {
                    if let Ok(time) = DateTime::parse_from_str(&str, &format) {
                        let diff = time.signed_duration_since(DateTime::UNIX_EPOCH);
                        diff.num_microseconds().map(|ms| ((ms as f64) / 1_000_000.0).into())
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
        .add_description_line(
            "Parse a date/time from a string into seconds since epoc. This version expect to get the time zone as well"
        )
        .add_description_line("The first argemnt should be the date")
        .add_description_line("The second argemnt should be the format as string")
        .add_description_line(
            "See details in [https://docs.rs/chrono/latest/chrono/format/strftime/index.html]."
        )
        .add_example(
            Example::new()
                .add_argument("\"2023 Dec 3 13:51:55.360 +0500\"")
                .add_argument("\"%Y %b %d %H:%M:%S%.3f %z\"")
                .expected_output("1701593515.360")
        )
        .add_example(
            Example::new()
                .add_argument("\" 3-Dec-2023 - 13:51:55.360\"")
                .add_argument("122")
        )
        .add_example(Example::new().add_argument("{}").add_argument("\"%v - %T%.3f\""))
}
