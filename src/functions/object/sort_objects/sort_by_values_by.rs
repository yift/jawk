use std::rc::Rc;

use crate::{
    functions_definitions::{Arguments, Example, FunctionDefinitions},
    json_value::JsonValue,
    processor::Context,
    selection::Get,
};

pub fn get() -> FunctionDefinitions {
    FunctionDefinitions::new("sort_by_values_by", 2, 2, |args| {
        struct Impl(Vec<Rc<dyn Get>>);
        impl Get for Impl {
            fn get(&self, value: &Context) -> Option<JsonValue> {
                match self.0.apply(value, 0) {
                    Some(JsonValue::Object(map)) => {
                        let mut map = map.clone();
                        map.sort_by(|_, v1, _, v2| {
                            let v1 = value.with_inupt(v1.clone());
                            let v1 = self.0.apply(&v1, 1);
                            let v2 = value.with_inupt(v2.clone());
                            let v2 = self.0.apply(&v2, 1);
                            v1.cmp(&v2)
                        });

                        Some(map.into())
                    }
                    _ => None,
                }
            }
        }
        Rc::new(Impl(args))
    })
        .add_alias("order_by_values_by")
        .add_description_line("Sort an object by a function to it's values.")
        .add_description_line(
            "If the first argument is an object, return object sorted by applying the second argument to it's values."
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
}
