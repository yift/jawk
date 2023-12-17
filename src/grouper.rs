use std::{str::FromStr, sync::Arc};

use indexmap::IndexMap;

use crate::{
    json_value::JsonValue,
    processor::{Context, Process, Titles},
    reader::from_string,
    selection::{read_getter, Get, SelectionParseError},
};

#[derive(Clone)]
pub struct Grouper {
    group_by: Arc<Box<dyn Get>>,
}

impl FromStr for Grouper {
    type Err = SelectionParseError;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let source = s.to_string();
        let mut reader = from_string(&source);
        reader.eat_whitespace()?;
        let group_by = read_getter(&mut reader)?;
        reader.eat_whitespace()?;
        if let Some(ch) = reader.next()? {
            return Err(SelectionParseError::ExpectingEof(
                reader.where_am_i(),
                ch as char,
            ));
        }
        Ok(Grouper {
            group_by: Arc::new(group_by),
        })
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
    group_by: Arc<Box<dyn Get>>,
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
        let context = Context::new_with_input(value);
        self.data.clear();
        self.next.process(context)
    }
    fn process(&mut self, context: Context) -> crate::processor::Result {
        if let (Some(key), Some(titles)) = (self.name(&context), &self.titles) {
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
impl GrouperProcess {
    fn name(&self, context: &Context) -> Option<String> {
        if let Some(JsonValue::String(str)) = self.group_by.get(context) {
            Some(str)
        } else {
            None
        }
    }
}
