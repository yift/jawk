use std::{rc::Rc, str::FromStr};

use indexmap::IndexMap;

use crate::{
    json_value::JsonValue,
    processor::{Context, Process, ProcessDesision, Result as ProcessResult, Titles},
    reader::from_string,
    selection::{read_getter, Get, SelectionParseError},
};

#[derive(Clone)]
pub struct Grouper {
    group_by: Rc<dyn Get>,
}

impl FromStr for Grouper {
    type Err = SelectionParseError;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let source = s.to_string();
        let mut reader = from_string(&source);
        reader.eat_whitespace()?;
        let group_by = read_getter(&mut reader)?;
        reader.eat_whitespace()?;
        if let Some(ch) = reader.peek()? {
            return Err(SelectionParseError::ExpectingEof(
                reader.where_am_i(),
                ch as char,
            ));
        }
        Ok(Grouper { group_by })
    }
}

impl Grouper {
    pub fn create_process(&self, next: Box<dyn Process>) -> Box<dyn Process> {
        Box::new(GrouperProcess {
            data: IndexMap::new(),
            next,
            group_by: self.group_by.clone(),
        })
    }
}

struct GrouperProcess {
    data: IndexMap<String, Vec<JsonValue>>,
    next: Box<dyn Process>,
    group_by: Rc<dyn Get>,
}
impl Process for GrouperProcess {
    fn complete(&mut self) -> ProcessResult<()> {
        let mut data = IndexMap::new();
        for (key, value) in &self.data {
            let value = value.clone().into();
            data.insert(key.clone(), value);
        }

        let value = data.into();
        let context = Context::new_with_no_context(value);
        self.data.clear();
        self.next.process(context)?;
        Ok(())
    }
    fn process(&mut self, context: Context) -> ProcessResult<ProcessDesision> {
        if let Some(key) = self.name(&context) {
            let value = context.build();
            self.data.entry(key).or_default().push(value);
        }
        Ok(ProcessDesision::Continue)
    }
    fn start(&mut self, _: Titles) -> ProcessResult<()> {
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

#[cfg(test)]
mod tests {
    use std::cell::RefCell;
    use std::ops::Deref;
    use std::rc::Rc;

    use super::*;
    use crate::json_value::JsonValue;
    use crate::processor::{Context, Titles};

    #[test]
    fn parse_parse_correctly() {
        let str = "(.len)";
        let grouper = Grouper::from_str(str).unwrap();

        let input = Context::new_with_no_context("test".into());

        assert_eq!(grouper.group_by.get(&input), Some((4).into()));
    }
    #[test]
    fn parse_fail_if_too_long() {
        let str = "(.len)3";
        let err = Grouper::from_str(str).err().unwrap();

        assert!(matches!(err, SelectionParseError::ExpectingEof(_, _)));
    }

    #[test]
    fn start_will_remove_the_title() -> ProcessResult<()> {
        struct Next(Rc<RefCell<bool>>);
        let data = Rc::new(RefCell::new(false));
        let one = Rc::new("one".into());
        let two = Rc::new("two".into());
        let titles = Titles::default().with_title(&one).with_title(&two);
        impl Process for Next {
            fn complete(&mut self) -> ProcessResult<()> {
                Ok(())
            }
            fn process(&mut self, _: Context) -> ProcessResult<ProcessDesision> {
                Ok(ProcessDesision::Continue)
            }
            fn start(&mut self, titles: Titles) -> ProcessResult<()> {
                assert_eq!(titles.len(), 0);
                *self.0.borrow_mut() = true;
                Ok(())
            }
        }
        {
            let str = "(.len)";
            let grouper = Grouper::from_str(str).unwrap();
            let next = Box::new(Next(data.clone()));
            let mut grouper = grouper.create_process(next);

            grouper.start(titles)?;
        }

        let binding = data.borrow();
        let data = &*binding;
        assert_eq!(data, &true);

        Ok(())
    }

    #[test]
    fn complete_will_complete_with_the_correct_values() -> ProcessResult<()> {
        struct Next {
            data: Rc<RefCell<Option<JsonValue>>>,
        }
        let data = Rc::new(RefCell::new(Option::None));
        impl Process for Next {
            fn complete(&mut self) -> ProcessResult<()> {
                Ok(())
            }
            fn process(&mut self, context: Context) -> ProcessResult<ProcessDesision> {
                let input = context.input().deref().clone();
                assert!(self.data.borrow().is_none());
                *self.data.borrow_mut() = Some(input);
                Ok(ProcessDesision::Continue)
            }
            fn start(&mut self, _: Titles) -> ProcessResult<()> {
                Ok(())
            }
        }
        {
            let str = ".";
            let grouper = Grouper::from_str(str).unwrap();
            let next = Box::new(Next { data: data.clone() });
            let one = Rc::new("one".into());
            let two = Rc::new("two".into());
            let titles = Titles::default().with_title(&one).with_title(&two);
            let mut grouper = grouper.create_process(next);

            grouper.start(titles)?;

            let context = Context::new_with_no_context("one".into())
                .with_result(&one, Some((1).into()))
                .with_result(&two, Some((2).into()));
            grouper.process(context)?;
            let context = Context::new_with_no_context("one".into())
                .with_result(&one, Some((4).into()))
                .with_result(&two, Some((6).into()));
            grouper.process(context)?;
            let context = Context::new_with_no_context("three".into())
                .with_result(&one, Some((3).into()))
                .with_result(&two, Some((4).into()));
            grouper.process(context)?;
            let context = Context::new_with_no_context((1).into())
                .with_result(&one, Some((10).into()))
                .with_result(&two, Some((20).into()));
            grouper.process(context)?;
            let context = Context::new_with_no_context("one".into())
                .with_result(&one, Some((10).into()))
                .with_result(&two, Some((20).into()));
            grouper.process(context)?;

            grouper.complete()?;
        }

        let binding = data.borrow();
        let data = binding.deref().clone().unwrap();
        let data = format!("{data}");
        assert_eq!(
            data,
            r#"{"one": [{"one": 1, "two": 2}, {"one": 4, "two": 6}, {"one": 10, "two": 20}], "three": [{"one": 3, "two": 4}]}"#
        );

        Ok(())
    }
}
