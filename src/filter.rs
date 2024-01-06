use std::{rc::Rc, str::FromStr};

use crate::{
    json_value::JsonValue,
    processor::{Process, ProcessDesision, Result as ProcessResult},
    reader::from_string,
    selection::{read_getter, Get, SelectionParseError},
};

#[derive(Clone)]
pub struct Filter {
    filter: Rc<dyn Get>,
}

impl FromStr for Filter {
    type Err = SelectionParseError;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let source = s.to_string();
        let mut reader = from_string(&source);
        reader.eat_whitespace()?;
        let filter = read_getter(&mut reader)?;
        reader.eat_whitespace()?;
        if let Some(ch) = reader.peek()? {
            return Err(SelectionParseError::ExpectingEof(
                reader.where_am_i(),
                ch as char,
            ));
        }
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
    filter: Rc<dyn Get>,
    next: Box<dyn Process>,
}

impl Process for ActiveFilter {
    fn complete(&mut self) -> ProcessResult<()> {
        self.next.complete()
    }
    fn start(&mut self, titles_so_far: crate::processor::Titles) -> ProcessResult<()> {
        self.next.start(titles_so_far)
    }
    fn process(&mut self, context: crate::processor::Context) -> ProcessResult<ProcessDesision> {
        if self.filter.get(&context) == Some(JsonValue::Boolean(true)) {
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
    use crate::processor::{Context, Titles};

    #[test]
    fn parse_parse_correctly() -> ProcessResult<()> {
        let str = "(> . 0)     ";
        let filter = Filter::from_str(str).unwrap();

        let input = Context::new_with_no_context((5).into());

        assert_eq!(filter.filter.get(&input), Some(true.into()));

        Ok(())
    }

    #[test]
    fn parse_fail_if_too_long() -> ProcessResult<()> {
        let str = "(> . 0)   3";
        let err = Filter::from_str(str).err().unwrap();

        assert_eq!(matches!(err, SelectionParseError::ExpectingEof(_, _)), true);

        Ok(())
    }

    #[test]
    fn start_will_keep_the_title() -> ProcessResult<()> {
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
                assert_eq!(titles.len(), 2);
                *self.0.borrow_mut() = true;
                Ok(())
            }
        }
        {
            let str = "(> . 0)     ";
            let filter = Filter::from_str(str).unwrap();
            let next = Box::new(Next(data.clone()));
            let mut filter = filter.create_process(next);

            filter.start(titles)?;
        }

        let binding = data.borrow();
        let data = binding.deref();
        assert_eq!(data, &true);

        Ok(())
    }

    #[test]
    fn complete_will_complete() -> ProcessResult<()> {
        struct Next(Rc<RefCell<bool>>);
        let data = Rc::new(RefCell::new(false));
        impl Process for Next {
            fn complete(&mut self) -> ProcessResult<()> {
                *self.0.borrow_mut() = true;
                Ok(())
            }
            fn process(&mut self, _: Context) -> ProcessResult<ProcessDesision> {
                Ok(ProcessDesision::Continue)
            }
            fn start(&mut self, _: Titles) -> ProcessResult<()> {
                Ok(())
            }
        }
        {
            let str = "(> . 0)";
            let filter = Filter::from_str(str).unwrap();
            let next = Box::new(Next(data.clone()));
            let mut filter = filter.create_process(next);

            filter.complete()?;
        }

        let binding = data.borrow();
        let data = binding.deref();
        assert_eq!(data, &true);

        Ok(())
    }

    #[test]
    fn filter_will_only_pass_passing_values() -> ProcessResult<()> {
        struct Next(Rc<RefCell<Vec<JsonValue>>>);
        let data = Rc::new(RefCell::new(Vec::new()));
        impl Process for Next {
            fn complete(&mut self) -> ProcessResult<()> {
                Ok(())
            }
            fn process(&mut self, context: Context) -> ProcessResult<ProcessDesision> {
                let value = context.input().deref().clone();
                let mut vec = self.0.borrow_mut();
                vec.push(value);
                Ok(ProcessDesision::Continue)
            }
            fn start(&mut self, _: Titles) -> ProcessResult<()> {
                Ok(())
            }
        }
        {
            let str = "(> . 6)";
            let filter = Filter::from_str(str).unwrap();
            let next = Box::new(Next(data.clone()));
            let mut filter = filter.create_process(next);

            let context = Context::new_with_no_context((100).into());
            filter.process(context)?;
            let context = Context::new_with_no_context((100).into());
            filter.process(context)?;
            let context = Context::new_with_no_context((10).into());
            filter.process(context)?;
            let context = Context::new_with_no_context((6).into());
            filter.process(context)?;
            let context = Context::new_with_no_context((5).into());
            filter.process(context)?;
            let context = Context::new_with_no_context((2).into());
            filter.process(context)?;
            let context = Context::new_with_no_context((0).into());
            filter.process(context)?;
            let context = Context::new_with_no_context((10).into());
            filter.process(context)?;
        }

        assert_eq!(
            *data.deref().borrow(),
            vec![(100).into(), (100).into(), (10).into(), (10).into()]
        );

        Ok(())
    }
}
