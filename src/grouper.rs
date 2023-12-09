use std::{str::FromStr, sync::Arc};

use indexmap::IndexMap;

use crate::{
    json_value::JsonValue,
    output::{get_value_or_values, Output},
    selection::{SelectionParseError, UnnamedSelection},
};

#[derive(Clone)]
pub struct Grouper {
    group_by: UnnamedSelection,
}

impl FromStr for Grouper {
    type Err = SelectionParseError;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let group_by = UnnamedSelection::from_str(s)?;
        Ok(Grouper { group_by })
    }
}

impl Grouper {
    pub fn start(&self, rows_titles: Arc<Vec<String>>, output: Box<dyn Output>) -> Box<dyn Output> {
        let data = IndexMap::new();
        let output = if let Some(output) = output.without_titles() {
            output
        } else {
            output
        };
        let grouper = ActiveGrouper {
            data,
            rows_titles: rows_titles.clone(),
            output,
            group_by: self.group_by.clone(),
        };
        Box::new(grouper)
    }
}
struct ActiveGrouper {
    data: IndexMap<String, Vec<JsonValue>>,
    rows_titles: Arc<Vec<String>>,
    output: Box<dyn Output>,
    group_by: UnnamedSelection,
}
impl Output for ActiveGrouper {
    fn output_row(&mut self, value: &JsonValue, row: Vec<Option<JsonValue>>) -> std::fmt::Result {
        if let Some(key) = self.group_by.name(value) {
            let data = get_value_or_values(value, row, &self.rows_titles);
            self.data.entry(key).or_default().push(data);
        }

        Ok(())
    }

    fn done(&mut self) -> std::fmt::Result {
        let mut data = IndexMap::new();
        for (key, value) in self.data.iter() {
            let value = value.clone().into();
            data.insert(key.clone(), value);
        }

        let value = data.into();
        self.output.output_row(&value, vec![])
    }

    fn without_titles(&self) -> Option<Box<dyn Output>> {
        None
    }
}
