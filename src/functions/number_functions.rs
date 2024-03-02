use std::rc::Rc;

use crate::{
    functions_definitions::{Arguments, Example, FunctionDefinitions, FunctionsGroup},
    json_value::JsonValue,
    processor::Context,
    selection::Get,
};

pub fn get_number_functions() -> FunctionsGroup {
    FunctionsGroup::new("number")

        .add_function(
            FunctionDefinitions::new("+", 2, usize::MAX, |args| {
                struct Impl(Vec<Rc<dyn Get>>);
                impl Get for Impl {
                    fn get(&self, value: &Context) -> Option<JsonValue> {
                        let mut sum = 0.0;
                        for s in &self.0 {
                            if let Some(JsonValue::Number(num)) = s.get(value) {
                                let num: f64 = num.into();
                                sum += num;
                            } else {
                                return None;
                            }
                        }
                        Some(sum.into())
                    }
                }
                Rc::new(Impl(args))
            })
                .add_alias("add")
                .add_alias("plus")
                .add_description_line("If all the arguments are number, add them.")
                .add_example(
                    Example::new().add_argument("1").add_argument("3").expected_output("4")
                )
                .add_example(
                    Example::new()
                        .add_argument("1")
                        .add_argument("10")
                        .add_argument("-4.1")
                        .add_argument("0.1")
                        .expected_output("7")
                )
                .add_example(
                    Example::new().add_argument("1").add_argument("3").add_argument("false")
                )
        )

        .add_function(
            FunctionDefinitions::new("-", 2, 2, |args| {
                struct Impl(Vec<Rc<dyn Get>>);
                impl Get for Impl {
                    fn get(&self, value: &Context) -> Option<JsonValue> {
                        if
                            let (Some(JsonValue::Number(num1)), Some(JsonValue::Number(num2))) = (
                                self.0.apply(value, 0),
                                self.0.apply(value, 1),
                            )
                        {
                            let num1: f64 = num1.into();
                            let num2: f64 = num2.into();
                            Some((num1 - num2).into())
                        } else {
                            None
                        }
                    }
                }
                Rc::new(Impl(args))
            })
                .add_alias("take_away")
                .add_alias("minus")
                .add_alias("substruct")
                .add_description_line(
                    "Substract the second argument from the first one if both are number."
                )
                .add_example(
                    Example::new().add_argument("100").add_argument("3").expected_output("97")
                )
                .add_example(
                    Example::new().add_argument("10").add_argument("3.2").expected_output("6.8")
                )
                .add_example(Example::new().add_argument("10").add_argument("\"text\""))
                .add_example(Example::new().add_argument("null").add_argument("6"))
        )

        .add_function(
            FunctionDefinitions::new("*", 2, usize::MAX, |args| {
                struct Impl(Vec<Rc<dyn Get>>);
                impl Get for Impl {
                    fn get(&self, value: &Context) -> Option<JsonValue> {
                        let mut sum = 1.0;
                        for s in &self.0 {
                            if let Some(JsonValue::Number(num)) = s.get(value) {
                                let num: f64 = num.into();
                                sum *= num;
                            } else {
                                return None;
                            }
                        }
                        Some(sum.into())
                    }
                }
                Rc::new(Impl(args))
            })
                .add_alias("times")
                .add_alias("multiple")
                .add_description_line("If all the arguments are number, multiply them.")
                .add_example(
                    Example::new().add_argument("2").add_argument("3").expected_output("6")
                )
                .add_example(
                    Example::new()
                        .add_argument("2")
                        .add_argument("15")
                        .add_argument("0.1")
                        .expected_output("3")
                )
                .add_example(Example::new().add_argument("2").add_argument("true"))
        )

        .add_function(
            FunctionDefinitions::new("/", 2, 2, |args| {
                struct Impl(Vec<Rc<dyn Get>>);
                impl Get for Impl {
                    fn get(&self, value: &Context) -> Option<JsonValue> {
                        if
                            let (Some(JsonValue::Number(num1)), Some(JsonValue::Number(num2))) = (
                                self.0.apply(value, 0),
                                self.0.apply(value, 1),
                            )
                        {
                            let num1: f64 = num1.into();
                            let num2: f64 = num2.into();
                            if num2 == 0.0 {
                                None
                            } else {
                                Some((num1 / num2).into())
                            }
                        } else {
                            None
                        }
                    }
                }
                Rc::new(Impl(args))
            })
                .add_alias("divide")
                .add_description_line(
                    "Divide the firs argument by the second argument. If the second argument is 0 will return nothing"
                )
                .add_example(
                    Example::new().add_argument("100").add_argument("25").expected_output("4")
                )
                .add_example(
                    Example::new().add_argument("7").add_argument("2").expected_output("3.5")
                )
                .add_example(Example::new().add_argument("7").add_argument("[]"))
                .add_example(Example::new().add_argument("{}").add_argument("5"))
        )

        .add_function(
            FunctionDefinitions::new("%", 2, 2, |args| {
                struct Impl(Vec<Rc<dyn Get>>);
                impl Get for Impl {
                    fn get(&self, value: &Context) -> Option<JsonValue> {
                        if
                            let (Some(JsonValue::Number(num1)), Some(JsonValue::Number(num2))) = (
                                self.0.apply(value, 0),
                                self.0.apply(value, 1),
                            )
                        {
                            let num1: f64 = num1.into();
                            let num2: f64 = num2.into();
                            if num2 == 0.0 {
                                None
                            } else {
                                Some((num1 % num2).into())
                            }
                        } else {
                            None
                        }
                    }
                }
                Rc::new(Impl(args))
            })
                .add_alias("mod")
                .add_alias("modulu")
                .add_description_line(
                    "Find the modulu of the division of the firs argument by the second argument. If the second argument is 0 will return nothing"
                )
                .add_example(
                    Example::new().add_argument("5").add_argument("3").expected_output("2")
                )
                .add_example(
                    Example::new().add_argument("7").add_argument("2").expected_output("1")
                )
                .add_example(Example::new().add_argument("7").add_argument("false"))
                .add_example(Example::new().add_argument("[1]").add_argument("4"))
        )
}
