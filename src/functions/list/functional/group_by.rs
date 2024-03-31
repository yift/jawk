use std::rc::Rc;

use indexmap::IndexMap;

use crate::{
    functions_definitions::{Arguments, Example, FunctionDefinitions},
    json_value::JsonValue,
    processor::Context,
    selection::Get,
};

pub fn get() -> FunctionDefinitions {
    FunctionDefinitions::new("group_by", 2, 2, |args| {
        struct Impl(Vec<Rc<dyn Get>>);
        impl Get for Impl {
            fn get(&self, value: &Context) -> Option<JsonValue> {
                match self.0.apply(value, 0) {
                    Some(JsonValue::Array(list)) => {
                        let mut groups = IndexMap::new();
                        for item in list {
                            let value = value.with_inupt(item.clone());
                            let Some(JsonValue::String(key)) = self.0.apply(&value, 1) else {
                                return None;
                            };
                            let values = groups.entry(key).or_insert_with(Vec::new);
                            values.push(item);
                        }

                        Some(
                            groups
                                .iter()
                                .map(|(k, v)| {
                                    (k.clone(), Into::<JsonValue>::into(v.clone()))
                                })
                                .collect::<IndexMap<_, _>>()
                                .into()
                        )
                    }
                    _ => None,
                }
            }
        }
        Rc::new(Impl(args))
    })
        .add_description_line("Group items by function.")
        .add_description_line(
            "If the first argument is a list, return list grouped by the second argument."
        )
        .add_example(
            Example::new()
                .add_argument("[\"11\", \"5\", \"23\", \"ab\", \"1\", \"\", \"100\", {}]")
                .add_argument("(stringify (len .))")
                .expected_output(
                    "{\"2\":[\"11\",\"23\",\"ab\"],\"1\":[\"5\",\"1\"],\"0\":[\"\",{}],\"3\":[\"100\"]}"
                )
        )
        .add_example(
            Example::new()
                .add_argument(
                    "[{\"g\": \"one\", \"v\": 1}, {\"g\": \"two\", \"v\": 2}, {\"g\": \"one\", \"v\": 33}, {\"g\": \"two\", \"v\": false}]"
                )
                .add_argument(".g")
                .expected_output(
                    "{\"one\":[{\"g\":\"one\",\"v\":1},{\"g\":\"one\",\"v\":33}],\"two\":[{\"g\":\"two\",\"v\":2},{\"g\":\"two\",\"v\":false}]}"
                )
        )
        .add_example(
            Example::new()
                .input("{\"key\": \"g\"}")
                .add_argument(
                    "[{\"g\": \"one\", \"v\": 1}, {\"g\": \"two\", \"v\": 2}, {\"g\": \"one\", \"v\": 33}, {\"g\": \"two\", \"v\": false}]"
                )
                .add_argument("(get . ^.key)")
                .expected_output(
                    "{\"one\":[{\"g\":\"one\",\"v\":1},{\"g\":\"one\",\"v\":33}],\"two\":[{\"g\":\"two\",\"v\":2},{\"g\":\"two\",\"v\":false}]}"
                )
                .explain(
                    "this group the element by a key that is taken from the input (`\"key\"`)."
                )
        )
        .add_example(Example::new().add_argument("344").add_argument("(stringify (len .))"))
        .add_example(
            Example::new()
                .add_argument("[\"11\", \"5\", \"23\", \"ab\", \"1\", \"\", \"100\", {}]")
                .add_argument("(len .)")
        )
}
