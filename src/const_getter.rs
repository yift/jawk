use crate::{
    json_parser::JsonParser,
    json_value::JsonValue,
    processor::Context,
    reader::Reader,
    selection::{Get, Result},
};
use std::io::Read;

pub struct ConstGetters {
    value: JsonValue,
}

impl Get for ConstGetters {
    fn get(&self, _: &Context) -> Option<JsonValue> {
        let val = self.value.clone();
        Some(val)
    }
}

impl ConstGetters {
    pub fn parse<R: Read>(reader: &mut Reader<R>) -> Result<Option<Self>> {
        let value = reader.next_json_value()?;
        Ok(value.map(|value| ConstGetters { value }))
    }
}
