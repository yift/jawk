use crate::{
    functions_definitions::{Arguments, Example, FunctionDefinitions, FunctionsGroup},
    json_value::JsonValue,
    processor::Context,
    selection::Get,
};

pub fn get_boolean_functions() -> FunctionsGroup {
    FunctionsGroup::new("Boolean functions")

        .add_function(
            FunctionDefinitions::new("=", 2, 2, |args| {
                struct Impl(Vec<Box<dyn Get>>);
                impl Get for Impl {
                    fn get(&self, value: &Context) -> Option<JsonValue> {
                        if
                            let (Some(val1), Some(val2)) = (
                                self.0.apply(value, 0),
                                self.0.apply(value, 1),
                            )
                        {
                            let eq = val1.eq(&val2);
                            Some(eq.into())
                        } else {
                            None
                        }
                    }
                }
                Box::new(Impl(args))
            })
                .add_description_line("Compare two value and return true if both are equals.")
                .add_example(
                    Example::new().add_argument("1").add_argument("3").expected_output("false")
                )
                .add_example(
                    Example::new().add_argument("1").add_argument("1").expected_output("true")
                )
                .add_example(
                    Example::new().add_argument("\"1\"").add_argument("1").expected_output("false")
                )
                .add_example(
                    Example::new()
                        .add_argument("\"abc\"")
                        .add_argument("\"abc\"")
                        .expected_output("true")
                )
        )

        .add_function(
            FunctionDefinitions::new("!=", 2, 2, |args| {
                struct Impl(Vec<Box<dyn Get>>);
                impl Get for Impl {
                    fn get(&self, value: &Context) -> Option<JsonValue> {
                        if
                            let (Some(val1), Some(val2)) = (
                                self.0.apply(value, 0),
                                self.0.apply(value, 1),
                            )
                        {
                            let eq = !val1.eq(&val2);
                            Some(eq.into())
                        } else {
                            None
                        }
                    }
                }
                Box::new(Impl(args))
            })
                .add_alias("<>")
                .add_description_line("Compare two value and return true if both are not equals.")
                .add_example(
                    Example::new().add_argument("1").add_argument("3").expected_output("true")
                )
                .add_example(
                    Example::new().add_argument("1").add_argument("1").expected_output("false")
                )
                .add_example(
                    Example::new().add_argument("\"1\"").add_argument("1").expected_output("true")
                )
                .add_example(
                    Example::new()
                        .add_argument("\"abc\"")
                        .add_argument("\"abc\"")
                        .expected_output("false")
                )
        )

        .add_function(
            FunctionDefinitions::new("<", 2, 2, |args| {
                struct Impl(Vec<Box<dyn Get>>);
                impl Get for Impl {
                    fn get(&self, value: &Context) -> Option<JsonValue> {
                        if
                            let (Some(val1), Some(val2)) = (
                                self.0.apply(value, 0),
                                self.0.apply(value, 1),
                            )
                        {
                            let eq = val1 < val2;
                            Some(eq.into())
                        } else {
                            None
                        }
                    }
                }
                Box::new(Impl(args))
            })
                .add_description_line(
                    "Compare two value and return true if the first is smaller than the second."
                )
                .add_example(
                    Example::new().add_argument("1").add_argument("3").expected_output("true")
                )
                .add_example(
                    Example::new().add_argument("1").add_argument("1").expected_output("false")
                )
                .add_example(
                    Example::new().add_argument("31").add_argument("1").expected_output("false")
                )
        )

        .add_function(
            FunctionDefinitions::new("<=", 2, 2, |args| {
                struct Impl(Vec<Box<dyn Get>>);
                impl Get for Impl {
                    fn get(&self, value: &Context) -> Option<JsonValue> {
                        if
                            let (Some(val1), Some(val2)) = (
                                self.0.apply(value, 0),
                                self.0.apply(value, 1),
                            )
                        {
                            let eq = val1 <= val2;
                            Some(eq.into())
                        } else {
                            None
                        }
                    }
                }
                Box::new(Impl(args))
            })
                .add_description_line(
                    "Compare two value and return true if the first is smaller or eqauls than the second."
                )
                .add_example(
                    Example::new().add_argument("1").add_argument("3").expected_output("true")
                )
                .add_example(
                    Example::new().add_argument("1").add_argument("1").expected_output("true")
                )
                .add_example(
                    Example::new().add_argument("31").add_argument("1").expected_output("false")
                )
        )

        .add_function(
            FunctionDefinitions::new(">=", 2, 2, |args| {
                struct Impl(Vec<Box<dyn Get>>);
                impl Get for Impl {
                    fn get(&self, value: &Context) -> Option<JsonValue> {
                        if
                            let (Some(val1), Some(val2)) = (
                                self.0.apply(value, 0),
                                self.0.apply(value, 1),
                            )
                        {
                            let eq = val1 >= val2;
                            Some(eq.into())
                        } else {
                            None
                        }
                    }
                }
                Box::new(Impl(args))
            })
                .add_description_line(
                    "Compare two value and return true if the first is greater or eqauls than the second."
                )
                .add_example(
                    Example::new().add_argument("1").add_argument("3").expected_output("false")
                )
                .add_example(
                    Example::new().add_argument("1").add_argument("1").expected_output("true")
                )
                .add_example(
                    Example::new().add_argument("31").add_argument("1").expected_output("true")
                )
        )

        .add_function(
            FunctionDefinitions::new(">", 2, 2, |args| {
                struct Impl(Vec<Box<dyn Get>>);
                impl Get for Impl {
                    fn get(&self, value: &Context) -> Option<JsonValue> {
                        if
                            let (Some(val1), Some(val2)) = (
                                self.0.apply(value, 0),
                                self.0.apply(value, 1),
                            )
                        {
                            let eq = val1 > val2;
                            Some(eq.into())
                        } else {
                            None
                        }
                    }
                }
                Box::new(Impl(args))
            })
                .add_description_line(
                    "Compare two value and return true if the first is greater than the second."
                )
                .add_example(
                    Example::new().add_argument("1").add_argument("3").expected_output("false")
                )
                .add_example(
                    Example::new().add_argument("1").add_argument("1").expected_output("false")
                )
                .add_example(
                    Example::new().add_argument("31").add_argument("1").expected_output("true")
                )
        )

        .add_function(
            FunctionDefinitions::new("and", 2, usize::MAX, |args| {
                struct Impl(Vec<Box<dyn Get>>);
                impl Get for Impl {
                    fn get(&self, value: &Context) -> Option<JsonValue> {
                        for s in &self.0 {
                            match s.get(value) {
                                Some(JsonValue::Boolean(true)) => {}
                                Some(JsonValue::Boolean(false)) => {
                                    return Some(false.into());
                                }
                                _ => {
                                    return None;
                                }
                            }
                        }
                        Some(true.into())
                    }
                }
                Box::new(Impl(args))
            })
                .add_alias("&&")
                .add_description_line(
                    "Return true if all the arguments are true, nothing if there is a non boolean argument and false if there is a false argument."
                )
                .add_example(
                    Example::new()
                        .add_argument("true")
                        .add_argument("true")
                        .add_argument("true")
                        .add_argument("true")
                        .expected_output("true")
                )
                .add_example(
                    Example::new()
                        .add_argument("true")
                        .add_argument("true")
                        .add_argument("false")
                        .add_argument("true")
                        .expected_output("false")
                )
                .add_example(
                    Example::new()
                        .add_argument("true")
                        .add_argument("true")
                        .add_argument("12")
                        .add_argument("true")
                )
        )

        .add_function(
            FunctionDefinitions::new("or", 2, usize::MAX, |args| {
                struct Impl(Vec<Box<dyn Get>>);
                impl Get for Impl {
                    fn get(&self, value: &Context) -> Option<JsonValue> {
                        for s in &self.0 {
                            match s.get(value) {
                                Some(JsonValue::Boolean(false)) => {}
                                Some(JsonValue::Boolean(true)) => {
                                    return Some(true.into());
                                }
                                _ => {
                                    return None;
                                }
                            }
                        }
                        Some(false.into())
                    }
                }
                Box::new(Impl(args))
            })
                .add_alias("||")
                .add_description_line(
                    "Return true if any of the arguments are true, nothing if there is a non boolean argument and false if all the arguments are false."
                )
                .add_example(
                    Example::new()
                        .add_argument("true")
                        .add_argument("true")
                        .add_argument("true")
                        .add_argument("true")
                        .expected_output("true")
                )
                .add_example(
                    Example::new()
                        .add_argument("false")
                        .add_argument("false")
                        .add_argument("false")
                        .add_argument("true")
                        .add_argument("true")
                        .add_argument("false")
                        .add_argument("true")
                        .expected_output("true")
                )
                .add_example(
                    Example::new()
                        .add_argument("false")
                        .add_argument("false")
                        .expected_output("false")
                )
                .add_example(
                    Example::new().add_argument("121").add_argument("true").add_argument("true")
                )
        )

        .add_function(
            FunctionDefinitions::new("xor", 2, 2, |args| {
                struct Impl(Vec<Box<dyn Get>>);
                impl Get for Impl {
                    fn get(&self, value: &Context) -> Option<JsonValue> {
                        if
                            let (Some(JsonValue::Boolean(val1)), Some(JsonValue::Boolean(val2))) = (
                                self.0.apply(value, 0),
                                self.0.apply(value, 1),
                            )
                        {
                            let eq = val1 ^ val2;
                            Some(eq.into())
                        } else {
                            None
                        }
                    }
                }
                Box::new(Impl(args))
            })
                .add_alias("^")
                .add_description_line("Return true if one, and only one, of the argument is true.")
                .add_example(
                    Example::new()
                        .add_argument("true")
                        .add_argument("true")
                        .expected_output("false")
                )
                .add_example(
                    Example::new()
                        .add_argument("true")
                        .add_argument("false")
                        .expected_output("true")
                )
                .add_example(
                    Example::new()
                        .add_argument("false")
                        .add_argument("true")
                        .expected_output("true")
                )
                .add_example(
                    Example::new()
                        .add_argument("false")
                        .add_argument("false")
                        .expected_output("false")
                )
                .add_example(Example::new().add_argument("null").add_argument("false"))
                .add_example(Example::new().add_argument("true").add_argument("12"))
        )

        .add_function(
            FunctionDefinitions::new("not", 1, 1, |args| {
                struct Impl(Vec<Box<dyn Get>>);
                impl Get for Impl {
                    fn get(&self, value: &Context) -> Option<JsonValue> {
                        if let Some(JsonValue::Boolean(val1)) = self.0.apply(value, 0) {
                            let eq = !val1;
                            Some(eq.into())
                        } else {
                            None
                        }
                    }
                }
                Box::new(Impl(args))
            })
                .add_alias("!")
                .add_description_line(
                    "Return false if the argument is true and true if the argument is false."
                )
                .add_example(Example::new().add_argument("true").expected_output("false"))
                .add_example(Example::new().add_argument("false").expected_output("true"))
                .add_example(Example::new().add_argument("(string? 12)").expected_output("true"))
                .add_example(Example::new().add_argument("12"))
        )
}
