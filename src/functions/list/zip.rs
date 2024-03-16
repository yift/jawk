use std::rc::Rc;

use indexmap::IndexMap;

use crate::{
    functions_definitions::{Example, FunctionDefinitions},
    json_value::JsonValue,
    processor::Context,
    selection::Get,
};

pub fn get() -> FunctionDefinitions {
    FunctionDefinitions::new("zip", 2, usize::MAX, |args| {
                struct Impl(Vec<Rc<dyn Get>>);
                impl Get for Impl {
                    fn get(&self, value: &Context) -> Option<JsonValue> {
                        let mut zipped_list = vec![];
                        let mut all_lists = Vec::with_capacity(self.0.len());
                        let mut max_size = 0;
                        for arg in &self.0 {
                            if let Some(JsonValue::Array(list)) = arg.get(value) {
                                if list.len() > max_size {
                                    max_size = list.len();
                                }
                                all_lists.push(list);
                            } else {
                                return None;
                            }
                        }
                        for index in 0..max_size {
                            let mut datum = IndexMap::new();
                            for (i, lst) in all_lists.iter().enumerate() {
                                if let Some(value) = lst.get(index) {
                                    datum.insert(format!(".{i}").to_string(), value.clone());
                                }
                            }
                            zipped_list.push(datum.into());
                        }
                        Some(zipped_list.into())
                    }
                }
                Rc::new(Impl(args))
            })
                .add_description_line("Zip a few list into a new list.")
                .add_description_line("All the arguments must be lists.")
                .add_description_line(
                    "The output will be a list of object, with keys in the format `\".i\"` where `i` is the index list."
                )
                .add_example(
                    Example::new()
                        .add_argument("[\"one\", \"two\", \"three\"]")
                        .add_argument("[1, 2, 3]")
                        .expected_output(
                            "[{\".0\": \"one\", \".1\": 1}, {\".0\": \"two\", \".1\": 2}, {\".0\": \"three\", \".1\": 3}]"
                        )
                )
                .add_example(
                    Example::new()
                        .add_argument("[\"one\", \"two\", \"three\"]")
                        .add_argument("[1, 2, 3]")
                        .add_argument("[false]")
                        .expected_output(
                            "[{\".0\": \"one\", \".1\": 1, \".2\": false}, {\".0\": \"two\", \".1\": 2}, {\".0\": \"three\", \".1\": 3}]"
                        )
                )
                .add_example(
                    Example::new()
                        .add_argument("[\"one\", \"two\", \"three\"]")
                        .add_argument("[1, 2, 3]")
                        .add_argument("6")
                )
}
