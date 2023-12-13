use std::str::FromStr;

use indexmap::IndexMap;

use crate::{
    json_value::JsonValue,
    processor::{Context, Process, Titles},
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
    pub fn create_process(&self, next: Box<dyn Process>) -> Box<dyn Process> {
        Box::new(GrouperProcess {
            data: IndexMap::new(),
            next,
            group_by: self.group_by.clone(),
            titles: None,
        })
    }
}

struct GrouperProcess {
    data: IndexMap<String, Vec<JsonValue>>,
    next: Box<dyn Process>,
    group_by: UnnamedSelection,
    titles: Option<Titles>,
}
impl Process for GrouperProcess {
    fn complete(&mut self) -> crate::processor::Result {
        let mut data = IndexMap::new();
        for (key, value) in self.data.iter() {
            let value = value.clone().into();
            data.insert(key.clone(), value);
        }

        let value = data.into();
        let context = Context::new(value);
        self.data.clear();
        self.next.process(context)
    }
    fn process(&mut self, context: Context) -> crate::processor::Result {
        if let (Some(key), Some(titles)) = (self.group_by.name(&context), &self.titles) {
            if let Some(value) = context.build(titles) {
                self.data.entry(key).or_default().push(value);
            }
        }
        Ok(())
    }
    fn start(&mut self, titles_so_far: Titles) -> crate::processor::Result {
        self.titles = Some(titles_so_far);
        self.next.start(Titles::default())
    }
}
