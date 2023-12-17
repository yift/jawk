use std::{str::FromStr, sync::Arc};

use crate::{
    json_value::JsonValue,
    processor::Process,
    reader::from_string,
    selection::{read_getter, Get, SelectionParseError},
};

#[derive(Clone)]
pub struct Filter {
    filter: Arc<Box<dyn Get>>,
}

impl FromStr for Filter {
    type Err = SelectionParseError;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let source = s.to_string();
        let mut reader = from_string(&source);
        reader.eat_whitespace()?;
        let filter = read_getter(&mut reader)?;
        reader.eat_whitespace()?;
        if let Some(ch) = reader.next()? {
            return Err(SelectionParseError::ExpectingEof(
                reader.where_am_i(),
                ch as char,
            ));
        }
        Ok(Filter {
            filter: Arc::new(filter),
        })
    }
}

impl Filter {
    pub fn create_process(&self, next: Box<dyn Process>) -> Box<dyn Process> {
        Box::new(ActiveFilter {
            filter: self.filter.clone(),
            next,
        })
    }
}
struct ActiveFilter {
    filter: Arc<Box<dyn Get>>,
    next: Box<dyn Process>,
}

impl Process for ActiveFilter {
    fn complete(&mut self) -> crate::processor::Result {
        self.next.complete()
    }
    fn start(&mut self, titles_so_far: crate::processor::Titles) -> crate::processor::Result {
        self.next.start(titles_so_far)
    }
    fn process(&mut self, context: crate::processor::Context) -> crate::processor::Result {
        if self.filter.get(&context) == Some(JsonValue::Boolean(true)) {
            self.next.process(context)?;
        }
        Ok(())
    }
}
