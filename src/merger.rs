use crate::{
    json_value::JsonValue,
    processor::{Context, Process, ProcessDesision, Result, Titles},
};

pub struct Merger {
    data: Vec<JsonValue>,
    next: Box<dyn Process>,
    titles: Option<Titles>,
}

impl Merger {
    pub fn create_process(next: Box<dyn Process>) -> Box<dyn Process> {
        Box::new(Merger {
            data: Vec::new(),
            next,
            titles: None,
        })
    }
}

impl Process for Merger {
    fn complete(&mut self) -> Result<()> {
        let mut data = Vec::new();
        for value in &self.data {
            let value = value.clone();
            data.push(value);
        }

        let value = data.into();
        let context = Context::new_with_no_context(value);
        self.data.clear();
        self.next.process(context)?;
        Ok(())
    }
    fn process(&mut self, context: Context) -> Result<ProcessDesision> {
        let value = context.build();
        self.data.push(value);
        Ok(ProcessDesision::Continue)
    }
    fn start(&mut self, titles_so_far: Titles) -> Result<()> {
        self.titles = Some(titles_so_far);
        self.next.start(Titles::default())
    }
}

#[cfg(test)]
mod tests {
    use std::cell::RefCell;
    use std::ops::Deref;
    use std::rc::Rc;

    use super::*;
    use crate::json_value::JsonValue;
    use crate::processor::{Context, Titles};

    #[test]
    fn start_will_remove_the_title() -> Result<()> {
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
                assert_eq!(titles.len(), 0);
                *self.0.borrow_mut() = true;
                Ok(())
            }
        }
        let next = Box::new(Next(data.clone()));
        let mut merger = Merger::create_process(next);

        merger.start(titles)?;

        let binding = data.borrow();
        let data = &*binding;
        assert_eq!(data, &true);

        Ok(())
    }

    #[test]
    fn complete_will_complete_with_the_correct_values() -> Result<()> {
        struct Next {
            data: Rc<RefCell<Option<JsonValue>>>,
        }
        let data = Rc::new(RefCell::new(Option::None));
        impl Process for Next {
            fn complete(&mut self) -> Result<()> {
                Ok(())
            }
            fn process(&mut self, context: Context) -> Result<ProcessDesision> {
                let input = context.input().deref().clone();
                assert!(self.data.borrow().is_none());
                *self.data.borrow_mut() = Some(input);
                Ok(ProcessDesision::Continue)
            }
            fn start(&mut self, _: Titles) -> Result<()> {
                Ok(())
            }
        }
        let next = Box::new(Next { data: data.clone() });
        let one = Rc::new("one".into());
        let two = Rc::new("two".into());
        let titles = Titles::default().with_title(&one).with_title(&two);
        let mut merger = Merger::create_process(next);
        merger.start(titles)?;

        let context = Context::new_with_no_context("one".into())
            .with_result(&one, Some((1).into()))
            .with_result(&two, Some((2).into()));
        merger.process(context)?;
        let context = Context::new_with_no_context("one".into())
            .with_result(&one, Some((4).into()))
            .with_result(&two, Some((6).into()));
        merger.process(context)?;
        let context = Context::new_with_no_context("three".into())
            .with_result(&one, Some((3).into()))
            .with_result(&two, Some((4).into()));
        merger.process(context)?;

        merger.complete()?;

        let binding = data.borrow();
        let data = binding.deref().clone().unwrap();
        let data = format!("{data}");
        assert_eq!(
            data,
            r#"[{"one": 1, "two": 2}, {"one": 4, "two": 6}, {"one": 3, "two": 4}]"#
        );

        Ok(())
    }
}
