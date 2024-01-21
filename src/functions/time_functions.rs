use std::{rc::Rc, time::SystemTime};

use chrono::{DateTime, NaiveDateTime, TimeZone, Utc};

use crate::{
    functions_definitions::{Arguments, Example, FunctionDefinitions, FunctionsGroup},
    json_value::JsonValue,
    processor::Context,
    selection::Get,
};

pub fn get_time_functions() -> FunctionsGroup {
    FunctionsGroup::new("time")
        .add_function(
            FunctionDefinitions::new("now", 0, 0, |_| {
                struct Impl;
                impl Get for Impl {
                    fn get(&self, _: &Context) -> Option<JsonValue> {
                        match SystemTime::now().duration_since(SystemTime::UNIX_EPOCH) {
                            Ok(d) => Some(d.as_secs_f64().into()),
                            _ => None,
                        }
                    }
                }
                Rc::new(Impl)
            })
            .add_description_line("Return the current time as seconds since epoch."),
        )
        .add_function(
            FunctionDefinitions::new("format_time", 2, 2, |args| {
                struct Impl(Vec<Rc<dyn Get>>);
                impl Get for Impl {
                    fn get(&self, value: &Context) -> Option<JsonValue> {
                        if let Some(JsonValue::Number(time)) = self.0.apply(value, 0) {
                            let since_epoch: f64 = time.into();
                            let seconds = since_epoch as i64;
                            let nsecs = ((since_epoch - seconds as f64) * 1e9) as u32;
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
            .add_description_line("The first argemnt should be the number of seconds since epoch")
            .add_description_line("The second argemnt should be the format as string")
            .add_description_line(
                "See details in https://docs.rs/chrono/0.4.31/chrono/format/strftime/index.html.",
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
            ),
        )

        .add_function(
            FunctionDefinitions::new("parse_time_with_zone", 2, 2, |args| {
                struct Impl(Vec<Rc<dyn Get>>);
                impl Get for Impl {
                    fn get(&self, value: &Context) -> Option<JsonValue> {
                        if let (Some(JsonValue::String(str)), Some(JsonValue::String(format))) =
                            (self.0.apply(value, 0), self.0.apply(value, 1))
                        {
                            if let Ok(time) = DateTime::parse_from_str(&str, &format) {
                                let diff = time.signed_duration_since(DateTime::UNIX_EPOCH);
                                diff.num_microseconds()
                                    .map(|ms| (ms as f64 / 1_000_000.0).into())
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
            .add_description_line("Parse a date/time from a string into seconds since epoc. This version expect to get the time zone as well")
            .add_description_line("The first argemnt should be the date")
            .add_description_line("The second argemnt should be the format as string")
            .add_description_line(
                "See details in https://docs.rs/chrono/0.4.31/chrono/format/strftime/index.html.",
            )
            .add_example(
                Example::new()
                    .add_argument("\"2023 Dec 3 13:51:55.360 +0500\"")
                    .add_argument("\"%Y %b %d %H:%M:%S%.3f %z\"")
                    .expected_output("1701593515.360"),
            )
            .add_example(
                Example::new()
                    .add_argument("\" 3-Dec-2023 - 13:51:55.360\"")
                    .add_argument("122"),
            )
            .add_example(
                Example::new()
                    .add_argument("{}")
                    .add_argument("\"%v - %T%.3f\""),
            ),
        )

        .add_function(
            FunctionDefinitions::new("parse_time", 2, 2, |args| {
                struct Impl(Vec<Rc<dyn Get>>);
                impl Get for Impl {
                    fn get(&self, value: &Context) -> Option<JsonValue> {
                        if let (Some(JsonValue::String(str)), Some(JsonValue::String(format))) =
                            (self.0.apply(value, 0), self.0.apply(value, 1))
                        {
                            if let Ok(time) = NaiveDateTime::parse_from_str(&str, &format) {
                                let diff = time.signed_duration_since(NaiveDateTime::UNIX_EPOCH);
                                diff.num_microseconds()
                                    .map(|ms| (ms as f64 / 1_000_000.0).into())
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
            .add_description_line("Parse a date/time from a string into seconds since epoc")
            .add_description_line("The first argemnt should be the date")
            .add_description_line("The second argemnt should be the format as string")
            .add_description_line(
                "See details in https://docs.rs/chrono/0.4.31/chrono/format/strftime/index.html.",
            )
            .add_example(
                Example::new()
                    .add_argument("\" 3-Dec-2023 - 13:51:55.360\"")
                    .add_argument("\"%v - %T%.3f\"")
                    .expected_output("1701611515.360"),
            )
            .add_example(
                Example::new()
                    .add_argument("\"2023 Dec 3 13:51:55.360 +0500\"")
                    .add_argument("\"%Y %b %d %H:%M:%S%.3f %z\"")
                    .expected_output("1701611515.360"),
            )
            .add_example(
                Example::new()
                    .add_argument("\" 3-Dec-2023 - 13:51:55.360\"")
                    .add_argument("122"),
            )
            .add_example(
                Example::new()
                    .add_argument("{}")
                    .add_argument("\"%v - %T%.3f\""),
            ),
        )
}
