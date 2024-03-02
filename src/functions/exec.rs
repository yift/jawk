use base64::prelude::*;
use std::{
    process::{Command, Stdio},
    rc::Rc,
    str::FromStr,
};

use indexmap::IndexMap;

use crate::{
    functions_definitions::{Arguments, Example, FunctionDefinitions, FunctionsGroup},
    json_value::JsonValue,
    processor::Context,
    selection::Get,
};

pub fn get_exec_functions() -> FunctionsGroup {
    FunctionsGroup::new("proccess")
        .add_function(
            FunctionDefinitions::new("exec", 1, usize::MAX, |vec| {
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
                        command.stdout(Stdio::piped());
                        command.stderr(Stdio::piped());
                        let Ok(child) = command.spawn() else {
                            return None;
                        };
                        let Ok(result) = child.wait_with_output() else {
                            return None;
                        };
                        let mut value = IndexMap::with_capacity(6);
                        value.insert("success".into(), result.status.success().into());
                        if let Some(code) = result.status.code() {
                            let code = f64::from(code);
                            value.insert("exit_code".into(), code.into());
                        }
                        value.insert(
                            "raw_stdout".into(),
                            BASE64_STANDARD.encode(&result.stdout).into()
                        );
                        if let Ok(stdout) = String::from_utf8(result.stdout) {
                            value.insert("stdout".into(), stdout.into());
                        }
                        value.insert(
                            "raw_stderr".into(),
                            BASE64_STANDARD.encode(&result.stderr).into()
                        );
                        if let Ok(stderr) = String::from_utf8(result.stderr) {
                            value.insert("stderr".into(), stderr.into());
                        }
                        Some(value.into())
                    }
                }
                Rc::new(Impl(vec))
            })
                .add_alias("execute")
                .add_description_line("Execute an external proccess and wait for it's completion.")
                .add_description_line(
                    "If all the arguments are strings run a process with that list."
                )
                .add_description_line("The result is an object with:")
                .add_description_line(
                    "* `success` Boolean to indicate if the process was successfull."
                )
                .add_description_line("* `exit_code` The process exit code.")
                .add_description_line(
                    "* `raw_stdout` The standart output of the process encode as BASE64."
                )
                .add_description_line("* `stdout` The standart output as text.")
                .add_description_line(
                    "* `raw_stderr` The standart error of the process encode as BASE64."
                )
                .add_description_line("* `stderr` The standart error as text.")
                .add_example(
                    Example::new()
                        .add_argument("\"echo\"")
                        .add_argument("\"hello\"")
                        .add_argument("\"world\"")
                        .validate_output(|o| {
                            if cfg!(windows) {
                                o.is_none()
                            } else {
                                let expected =
                                  JsonValue::from_str(r#"{"success": true, "exit_code": 0, "raw_stdout": "aGVsbG8gd29ybGQK", "stdout": "hello world\n", "raw_stderr": "", "stderr": ""}"#).ok();
                                expected == *o
                            }
                        })
                )
                .add_example(
                    Example::new()
                        .add_argument("\"cat\"")
                        .add_argument("\"no such file\"")
                        .validate_output(|o| {
                            if cfg!(windows) {
                                o.is_none()
                            } else {
                                let expected = JsonValue::from_str(r#"{"success": false, "exit_code": 1, "raw_stdout": "", "stdout": "", "raw_stderr": "Y2F0OiAnbm8gc3VjaCBmaWxlJzogTm8gc3VjaCBmaWxlIG9yIGRpcmVjdG9yeQo=", "stderr": "cat: 'no such file': No such file or directory\n"}"#).ok();
                                expected == *o
                            }
                        })
                )
                .add_example(Example::new().add_argument("\"no such exec\""))
                .add_example(Example::new().add_argument("23").explain("23 is not a string"))
                .add_example(
                    Example::new()
                        .add_argument("\"echo\"")
                        .add_argument("23")
                        .explain("23 is not a string")
                )
        )
        .add_function(
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
                .add_description_line("Trigger an external proccess and return it's process ID.")
                .add_description_line(
                    "If all the arguments are strings run a process with that list."
                )
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
                            }
                        )
                )
                .add_example(Example::new().add_argument("\"no such exec\""))
                .add_example(Example::new().add_argument("23").explain("23 is not a string"))
                .add_example(
                    Example::new()
                        .add_argument("\"echo\"")
                        .add_argument("23")
                        .explain("23 is not a string")
                )
        )
}
