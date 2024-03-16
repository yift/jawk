use std::rc::Rc;

use indexmap::IndexMap;

use crate::{
    functions_definitions::{Example, FunctionDefinitions},
    json_value::JsonValue,
    processor::Context,
    selection::Get,
};

pub fn get() -> FunctionDefinitions {
    FunctionDefinitions::new("cross", 2, usize::MAX, |args| {
                struct Impl(Vec<Rc<dyn Get>>);
                impl Get for Impl {
                    fn get(&self, value: &Context) -> Option<JsonValue> {
                        let mut joined_list = vec![IndexMap::new()];
                        let mut all_lists = Vec::with_capacity(self.0.len());
                        for arg in &self.0 {
                            if let Some(JsonValue::Array(list)) = arg.get(value) {
                                all_lists.push(list);
                            } else {
                                return None;
                            }
                        }
                        for (i, lst) in all_lists.iter().enumerate() {
                            let key = format!(".{i}");
                            let mut new_joined_list = vec![];
                            for val in lst {
                                for so_far in &joined_list {
                                    let mut datum = so_far.clone();
                                    datum.insert(key.clone(), val.clone());
                                    new_joined_list.push(datum);
                                }
                            }
                            joined_list = new_joined_list;
                        }
                        let joined_list: Vec<_> = joined_list
                            .iter()
                            .map(|f| f.clone().into())
                            .collect();
                        Some(joined_list.into())
                    }
                }
                Rc::new(Impl(args))
            })
                .add_description_line("Join a few list (i.e. Cartesian product) into a new list.")
                .add_description_line("All the arguments must be lists.")
                .add_description_line(
                    "The output will be a list of object, with keys in the format \".i\"."
                )
                .add_example(
                    Example::new()
                        .add_argument("[\"one\", \"two\"]")
                        .add_argument("[1, 2]")
                        .add_argument("[true]")
                        .add_argument("[false]")
                        .expected_output(
                            "[{\".0\": \"one\", \".1\": 1, \".2\": true, \".3\": false}, {\".0\": \"two\", \".1\": 1, \".2\": true, \".3\": false}, {\".0\": \"one\", \".1\": 2, \".2\": true, \".3\": false}, {\".0\": \"two\", \".1\": 2, \".2\": true, \".3\": false}]"
                        )
                )
                .add_example(
                    Example::new()
                        .add_argument("[\"one\", \"two\", \"three\"]")
                        .add_argument("[1, 2, 3]")
                        .add_argument("6")
                )
}
