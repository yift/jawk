use std::rc::Rc;

use indexmap::IndexMap;

use crate::{
    functions_definitions::{Arguments, Example, FunctionDefinitions},
    json_value::JsonValue,
    processor::Context,
    selection::Get,
};

pub fn get() -> FunctionDefinitions {
    FunctionDefinitions::new("fold", 2, 3, |args| {
                struct Impl(Vec<Rc<dyn Get>>);
                impl Get for Impl {
                    fn get(&self, context: &Context) -> Option<JsonValue> {
                        if let Some(JsonValue::Array(list)) = self.0.apply(context, 0) {
                            let has_init = self.0.len() > 2;
                            let mut current = if has_init {
                                self.0.apply(context, 1)
                            } else {
                                None
                            };
                            let func = if has_init { self.0.get(2) } else { self.0.get(1) };
                            if let Some(func) = func {
                                for (index, value) in list.iter().enumerate() {
                                    let mut mp = IndexMap::with_capacity(3);
                                    if let Some(current) = &current {
                                        mp.insert("so_far".to_string(), current.clone());
                                    }
                                    mp.insert("value".to_string(), value.clone());
                                    mp.insert("index".to_string(), index.into());
                                    let input = context.with_inupt(mp.into());
                                    current = func.get(&input);
                                }
                                current
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
                .add_description_line("Fold all the items in a list into a new value.")
                .add_description_line(
                    "The first item should be the list, the second one the initial value and the third one a function that create the fold."
                )
                .add_description_line(
                    "If the fuinction has only two arguments, the initial value will not be set."
                )
                .add_description_line(
                    "The function will accespt as input an hash with `value`, `index` and `so_far` keys (if the previous run returned nothing, the `so_far` will be empty)."
                )
                .add_example(
                    Example::new()
                        .add_argument("[1, 10, 0.6]")
                        .add_argument("100")
                        .add_argument("(+ .index .so_far .value)")
                        .expected_output("114.6")
                        .explain(
                            "the first argument is 100, and then the fold will add all the argument in the list, 100 + 1 + 10 + 0.6 = 114.6."
                        )
                )
                .add_example(
                    Example::new()
                        .add_argument("[1, 10, 0.6]")
                        .add_argument("(? (number? .so_far) (+ .so_far .value) .value)")
                        .expected_output("11.6")
                        .explain(
                            "if `so_far` is not a number, we started the fold, so we can return the value, hene this is a simple sum, 1 + 10 + 0.6 = 11.6."
                        )
                )
                .add_example(Example::new().add_argument("{}").add_argument("1").add_argument("2"))
}
