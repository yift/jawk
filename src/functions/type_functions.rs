use crate::{
    functions_definitions::{Arguments, Example, FunctionDefinitions, FunctionsGroup},
    json_value::JsonValue,
    selection::Get,
};

pub fn get_type_functions() -> FunctionsGroup {
    FunctionsGroup::new("Type functions")
        .add_function(
            FunctionDefinitions::new("array?", 1, 1, |args| {
                struct Impl(Vec<Box<dyn Get>>);
                impl Get for Impl {
                    fn get(&self, value: &Option<JsonValue>) -> Option<JsonValue> {
                        match self.0.apply(value, 0) {
                            Some(JsonValue::Array(_)) => Some(true.into()),
                            _ => Some(false.into()),
                        }
                    }
                }
                Box::new(Impl(args))
            })
            .add_alias("list?")
            .add_description_line("return true if the argument is an array.")
            .add_example(
                Example::new()
                    .add_argument("[1, 2, 3, 4]")
                    .expected_output("true"),
            )
            .add_example(Example::new().add_argument("312").expected_output("false")),
        )
        .add_function(
            FunctionDefinitions::new("object?", 1, 1, |args| {
                struct Impl(Vec<Box<dyn Get>>);
                impl Get for Impl {
                    fn get(&self, value: &Option<JsonValue>) -> Option<JsonValue> {
                        match self.0.apply(value, 0) {
                            Some(JsonValue::Object(_)) => Some(true.into()),
                            _ => Some(false.into()),
                        }
                    }
                }
                Box::new(Impl(args))
            })
            .add_alias("map?")
            .add_alias("hash?")
            .add_description_line("return true if the argument is an object.")
            .add_example(
                Example::new()
                    .add_argument("[1, 2, 3, 4]")
                    .expected_output("false"),
            )
            .add_example(
                Example::new()
                    .add_argument("{\"key\": 12}")
                    .expected_output("true"),
            ),
        )
        .add_function(
            FunctionDefinitions::new("null?", 1, 1, |args| {
                struct Impl(Vec<Box<dyn Get>>);
                impl Get for Impl {
                    fn get(&self, value: &Option<JsonValue>) -> Option<JsonValue> {
                        match self.0.apply(value, 0) {
                            Some(JsonValue::Null) => Some(true.into()),
                            _ => Some(false.into()),
                        }
                    }
                }
                Box::new(Impl(args))
            })
            .add_alias("nil?")
            .add_description_line("return true if the argument is a null.")
            .add_example(Example::new().add_argument("null").expected_output("true"))
            .add_example(Example::new().add_argument("1").expected_output("false")),
        )
        .add_function(
            FunctionDefinitions::new("bool?", 1, 1, |args| {
                struct Impl(Vec<Box<dyn Get>>);
                impl Get for Impl {
                    fn get(&self, value: &Option<JsonValue>) -> Option<JsonValue> {
                        match self.0.apply(value, 0) {
                            Some(JsonValue::Boolean(_)) => Some(true.into()),
                            _ => Some(false.into()),
                        }
                    }
                }
                Box::new(Impl(args))
            })
            .add_alias("boolean?")
            .add_description_line("return true if the argument is a boolean.")
            .add_example(Example::new().add_argument("false").expected_output("true"))
            .add_example(
                Example::new()
                    .add_argument("\"false\"")
                    .expected_output("false"),
            ),
        )
        .add_function(
            FunctionDefinitions::new("number?", 1, 1, |args| {
                struct Impl(Vec<Box<dyn Get>>);
                impl Get for Impl {
                    fn get(&self, value: &Option<JsonValue>) -> Option<JsonValue> {
                        match self.0.apply(value, 0) {
                            Some(JsonValue::Number(_)) => Some(true.into()),
                            _ => Some(false.into()),
                        }
                    }
                }
                Box::new(Impl(args))
            })
            .add_description_line("return true if the argument is a number.")
            .add_example(
                Example::new()
                    .add_argument("\"str\"")
                    .expected_output("false"),
            )
            .add_example(Example::new().add_argument("1.32").expected_output("true")),
        )
        .add_function(
            FunctionDefinitions::new("string?", 1, 1, |args| {
                struct Impl(Vec<Box<dyn Get>>);
                impl Get for Impl {
                    fn get(&self, value: &Option<JsonValue>) -> Option<JsonValue> {
                        match self.0.apply(value, 0) {
                            Some(JsonValue::String(_)) => Some(true.into()),
                            _ => Some(false.into()),
                        }
                    }
                }
                Box::new(Impl(args))
            })
            .add_description_line("return true if the argument is a string.")
            .add_example(
                Example::new()
                    .add_argument("\"one\"")
                    .expected_output("true"),
            )
            .add_example(Example::new().add_argument("1.32").expected_output("false")),
        )
        .add_function(
            FunctionDefinitions::new("empty?", 1, 1, |args| {
                struct Impl(Vec<Box<dyn Get>>);
                impl Get for Impl {
                    fn get(&self, value: &Option<JsonValue>) -> Option<JsonValue> {
                        match self.0.apply(value, 0) {
                            Some(_) => Some(false.into()),
                            _ => Some(true.into()),
                        }
                    }
                }
                Box::new(Impl(args))
            })
            .add_alias("nothing?")
            .add_description_line("return true if the argument is nothing.")
            .add_example(
                Example::new()
                    .add_argument("\"one\"")
                    .expected_output("false"),
            )
            .add_example(
                Example::new()
                    .add_argument(".key")
                    .expected_output("true")
                    .input("{}"),
            ),
        )
}
