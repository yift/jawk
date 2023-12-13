use crate::{json_value::JsonValue, output::Output};
use core::hash::Hash;
use std::collections::HashSet;

pub struct DupilicationRemover {
    knwon_lines: HashSet<ValueOrRow>,
    output: Box<dyn Output>,
}

impl DupilicationRemover {
    pub fn new(output: Box<dyn Output>) -> Box<Self> {
        let knwon_lines = HashSet::new();
        Box::new(DupilicationRemover {
            knwon_lines,
            output,
        })
    }
}

#[derive(Hash, PartialEq, Eq)]
enum ValueOrRow {
    Value(JsonValue),
    Row(Vec<Option<JsonValue>>),
}

impl Output for DupilicationRemover {
    fn without_titles(&self) -> Option<Box<dyn Output>> {
        None
    }
    fn output_row(&mut self, value: &JsonValue, row: Vec<Option<JsonValue>>) -> std::fmt::Result {
        let to_keep = if row.is_empty() {
            ValueOrRow::Value(value.clone())
        } else {
            ValueOrRow::Row(row.clone())
        };
        if self.knwon_lines.insert(to_keep) {
            self.output.output_row(value, row)
        } else {
            Ok(())
        }
    }
}
