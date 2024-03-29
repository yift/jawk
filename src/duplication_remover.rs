use crate::processor::{Context, ContextKey, Process, ProcessDesision, Result, Titles};
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
    fn complete(&mut self) -> Result<()> {
        self.knwon_lines.clear();
        self.next.complete()
    }
    fn start(&mut self, titles_so_far: Titles) -> Result<()> {
        self.next.start(titles_so_far)
    }
    fn process(&mut self, context: Context) -> Result<ProcessDesision> {
        if self.knwon_lines.insert(context.key()) {
            self.next.process(context)
        } else {
            Ok(ProcessDesision::Continue)
        }
    }
}

#[cfg(test)]
mod tests {
    use std::cell::RefCell;
    use std::ops::Deref;
    use std::rc::Rc;

    use super::*;
    use crate::json_value::JsonValue;

    #[test]
    fn duplicate_lines_follow_only_once() -> Result<()> {
        struct Next(Rc<RefCell<Vec<JsonValue>>>);
        let data = Rc::new(RefCell::new(Vec::new()));
        impl Process for Next {
            fn complete(&mut self) -> Result<()> {
                Ok(())
            }
            fn process(&mut self, context: Context) -> Result<ProcessDesision> {
                let value = context.input().deref().clone();
                let mut vec = self.0.borrow_mut();
                vec.push(value);
                Ok(ProcessDesision::Continue)
            }
            fn start(&mut self, _: Titles) -> Result<()> {
                Ok(())
            }
        }
        {
            let next = Box::new(Next(data.clone()));
            let mut uniquness = Uniquness::create_process(next);

            let context = Context::new_with_no_context("text".into());
            uniquness.process(context)?;
            let context = Context::new_with_no_context("text".into());
            uniquness.process(context)?;
            let context = Context::new_with_no_context((100).into());
            uniquness.process(context)?;
            let context = Context::new_with_no_context("text2".into());
            uniquness.process(context)?;
            let context = Context::new_with_no_context((200).into());
            uniquness.process(context)?;
            let context = Context::new_with_no_context((100).into());
            uniquness.process(context)?;
            let context = Context::new_with_no_context((200).into());
            uniquness.process(context)?;
            let context = Context::new_with_no_context("text2".into());
            uniquness.process(context)?;
            let context = Context::new_with_no_context("text".into());
            uniquness.process(context)?;
            let context = Context::new_with_no_context((100).into());
            uniquness.process(context)?;
            let context = Context::new_with_no_context((200).into());
            uniquness.process(context)?;
            let context = Context::new_with_no_context("text2".into());
            uniquness.process(context)?;
        }

        assert_eq!(
            *data.deref().borrow(),
            vec!["text".into(), (100).into(), "text2".into(), (200).into()]
        );

        Ok(())
    }

    #[test]
    fn start_will_keep_the_title() -> Result<()> {
        struct Next(Rc<RefCell<bool>>);
        let data = Rc::new(RefCell::new(false));
        let one = Rc::new("one".into());
        let two = Rc::new("two".into());
        let titles = Titles::default().with_title(&one).with_title(&two);
        impl Process for Next {
            fn complete(&mut self) -> Result<()> {
                Ok(())
            }
            fn process(&mut self, _: Context) -> Result<ProcessDesision> {
                Ok(ProcessDesision::Continue)
            }
            fn start(&mut self, titles: Titles) -> Result<()> {
                assert_eq!(titles.len(), 2);
                *self.0.borrow_mut() = true;
                Ok(())
            }
        }
        {
            let next = Box::new(Next(data.clone()));
            let mut uniquness = Uniquness::create_process(next);

            uniquness.start(titles)?;
        }

        let binding = data.borrow();
        let data = &*binding;
        assert_eq!(data, &true);

        Ok(())
    }

    #[test]
    fn complete_will_complete() -> Result<()> {
        struct Next(Rc<RefCell<bool>>);
        let data = Rc::new(RefCell::new(false));
        impl Process for Next {
            fn complete(&mut self) -> Result<()> {
                *self.0.borrow_mut() = true;
                Ok(())
            }
            fn process(&mut self, _: Context) -> Result<ProcessDesision> {
                Ok(ProcessDesision::Continue)
            }
            fn start(&mut self, _: Titles) -> Result<()> {
                Ok(())
            }
        }
        {
            let next = Box::new(Next(data.clone()));
            let mut uniquness = Uniquness::create_process(next);

            uniquness.complete()?;
        }

        let binding = data.borrow();
        let data = &*binding;
        assert_eq!(data, &true);

        Ok(())
    }
}
