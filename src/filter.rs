use std::str::FromStr;

use crate::{
    processor::Process,
    selection::{SelectionParseError, UnnamedSelection},
};

#[derive(Clone)]
pub struct Filter {
    filter: UnnamedSelection,
}

impl FromStr for Filter {
    type Err = SelectionParseError;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let filter = UnnamedSelection::from_str(s)?;
        Ok(Filter { filter })
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
    filter: UnnamedSelection,
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
        if self.filter.pass(context.input()) {
            self.next.process(context)?;
        }
        Ok(())
    }
}
