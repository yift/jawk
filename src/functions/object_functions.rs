use crate::{
    functions_definitions::{Arguments, Example, FunctionDefinitions, FunctionsGroup},
    json_value::JsonValue,
    selection::Get,
};

pub fn get_object_functions() -> FunctionsGroup {
    FunctionsGroup::new("Object functions")

        .add_function(
            FunctionDefinitions::new("keys", 1, 1, |args| {
                struct Impl(Arguments);
                impl Get for Impl {
                    fn get(&self, value: &Option<JsonValue>) -> Option<JsonValue> {
                        if let Some(JsonValue::Object(map)) = self.0.apply(value, 0) {
                            Some(
                                JsonValue::Array(
                                    map.keys().cloned().map(JsonValue::String).collect()
                                )
                            )
                        } else {
                            None
                        }
                    }
                }
                Box::new(Impl(Arguments::new(args)))
            })
                .add_description_line("Get the list of keys from an object.")
                .add_example(
                    Example::new()
                        .add_argument("{\"key-1\": 1, \"key-2\": false}")
                        .expected_output("[\"key-1\",\"key-2\"]")
                )
                .add_example(Example::new().add_argument("[1, 2, 4]"))
        )

        .add_function(
            FunctionDefinitions::new("values", 1, 1, |args| {
                struct Impl(Arguments);
                impl Get for Impl {
                    fn get(&self, value: &Option<JsonValue>) -> Option<JsonValue> {
                        if let Some(JsonValue::Object(map)) = self.0.apply(value, 0) {
                            Some(JsonValue::Array(map.values().cloned().collect()))
                        } else {
                            None
                        }
                    }
                }
                Box::new(Impl(Arguments::new(args)))
            })
                .add_alias("vals")
                .add_description_line("Get the list of values from an object.")
                .add_example(
                    Example::new()
                        .add_argument("{\"key-1\": 1, \"key-2\": false}")
                        .expected_output("[1, false]")
                )
                .add_example(Example::new().add_argument("[1, 2, 4]"))
        )

        .add_function(
            FunctionDefinitions::new("sort_by_keys", 1, 1, |args| {
                struct Impl(Arguments);
                impl Get for Impl {
                    fn get(&self, value: &Option<JsonValue>) -> Option<JsonValue> {
                        match self.0.apply(value, 0) {
                            Some(JsonValue::Object(map)) => {
                                let mut map = map.clone();
                                map.sort_keys();

                                Some(map.into())
                            }
                            _ => None,
                        }
                    }
                }
                Box::new(Impl(Arguments::new(args)))
            })
                .add_alias("order_by_keys")
                .add_description_line("Sort an object by it's keys.")
                .add_description_line(
                    "If the first argument is an object, return object sorted by it's keys."
                )
                .add_example(
                    Example::new()
                        .add_argument("{\"z\": 1, \"x\": 2, \"w\": null}")
                        .expected_output("{\"w\":null,\"x\":2,\"z\":1}")
                )
                .add_example(Example::new().add_argument("false"))
        )

        .add_function(
            FunctionDefinitions::new("sort_by_values", 1, 1, |args| {
                struct Impl(Arguments);
                impl Get for Impl {
                    fn get(&self, value: &Option<JsonValue>) -> Option<JsonValue> {
                        match self.0.apply(value, 0) {
                            Some(JsonValue::Object(map)) => {
                                let mut map = map.clone();
                                map.sort_by(|_, v1, _, v2| v1.cmp(v2));

                                Some(map.into())
                            }
                            _ => None,
                        }
                    }
                }
                Box::new(Impl(Arguments::new(args)))
            })
                .add_alias("order_by_values")
                .add_description_line("Sort an object by it's values.")
                .add_description_line(
                    "If the first argument is an object, return object sorted by it's values."
                )
                .add_example(
                    Example::new()
                        .add_argument("{\"z\": 5, \"x\": 2, \"w\": null}")
                        .expected_output("{\"w\":null,\"x\":2,\"z\":5}")
                )
                .add_example(Example::new().add_argument("false"))
        )

        .add_function(
            FunctionDefinitions::new("sort_by_values_by", 2, 2, |args| {
                struct Impl(Arguments);
                impl Get for Impl {
                    fn get(&self, value: &Option<JsonValue>) -> Option<JsonValue> {
                        match self.0.apply(value, 0) {
                            Some(JsonValue::Object(map)) => {
                                let mut map = map.clone();
                                map.sort_by(|_, v1, _, v2| {
                                    let v1 = Some(v1.clone());
                                    let v1 = self.0.apply(&v1, 1);
                                    let v2 = Some(v2.clone());
                                    let v2 = self.0.apply(&v2, 1);
                                    v1.cmp(&v2)
                                });

                                Some(map.into())
                            }
                            _ => None,
                        }
                    }
                }
                Box::new(Impl(Arguments::new(args)))
            })
                .add_alias("order_by_values_by")
                .add_description_line("Sort an object by a function to it's values.")
                .add_description_line(
                    "If the first argument is an object, return object sorted by applying the second argumetn to it's values."
                )
                .add_example(
                    Example::new()
                        .add_argument(
                            "{\"a\": [1, 2, 3], \"b\": [1], \"c\": [2], \"d\": [3], \"e\": [0, null, 0]}"
                        )
                        .add_argument("(.len)")
                        .expected_output(
                            "{\"b\":[1],\"c\":[2],\"d\":[3],\"a\":[1,2,3],\"e\":[0,null,0]}"
                        )
                )
                .add_example(Example::new().add_argument("false").add_argument("."))
        )
}
