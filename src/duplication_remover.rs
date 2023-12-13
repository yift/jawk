use crate::processor::{Context, ContextKey, Process, Titles};
use std::collections::HashSet;

pub struct Uniquness {
    knwon_lines: HashSet<ContextKey>,
    next: Box<dyn Process>,
}

impl Uniquness {
    pub fn create_process(next: Box<dyn Process>) -> Box<dyn Process> {
        Box::new(Uniquness {
            knwon_lines: HashSet::new(),
            next,
        })
    }
}
impl Process for Uniquness {
    fn complete(&mut self) -> crate::processor::Result {
        self.knwon_lines.clear();
        self.next.complete()
    }
    fn start(&mut self, titles_so_far: Titles) -> crate::processor::Result {
        self.next.start(titles_so_far)
    }
    fn process(&mut self, context: Context) -> crate::processor::Result {
        if self.knwon_lines.insert(context.key()) {
            self.next.process(context)
        } else {
            Ok(())
        }
    }
}
