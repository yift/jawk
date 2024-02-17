use std::{rc::Rc, str::FromStr};

use crate::{
    json_value::JsonValue,
    processor::{Context, Process, ProcessDesision, Result as ProcessResult, Titles},
    reader::from_string,
    selection::{read_getter, Get, SelectionParseError},
};

#[derive(Clone)]
pub struct Splitter {
    split_by: Rc<dyn Get>,
}

impl FromStr for Splitter {
    type Err = SelectionParseError;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let source = s.to_string();
        let mut reader = from_string(&source);
        reader.eat_whitespace()?;
        let split_by = read_getter(&mut reader)?;
        reader.eat_whitespace()?;
        if let Some(ch) = reader.peek()? {
            return Err(SelectionParseError::ExpectingEof(
                reader.where_am_i(),
                ch as char,
            ));
        }
        Ok(Splitter { split_by })
    }
}

impl Splitter {
    pub fn create_process(&self, next: Box<dyn Process>) -> Box<dyn Process> {
        Box::new(SplitterProcess {
            next,
            split_by: self.split_by.clone(),
        })
    }
}

struct SplitterProcess {
    next: Box<dyn Process>,
    split_by: Rc<dyn Get>,
}
impl Process for SplitterProcess {
    fn complete(&mut self) -> ProcessResult<()> {
        self.next.complete()
    }
    fn process(&mut self, context: Context) -> ProcessResult<ProcessDesision> {
        if let Some(JsonValue::Array(lst)) = self.split_by.get(&context) {
            for val in lst {
                let context = context.with_inupt(val);
                self.next.process(context)?;
            }
        }
        Ok(ProcessDesision::Continue)
    }
    fn start(&mut self, titles_so_far: Titles) -> ProcessResult<()> {
        self.next.start(titles_so_far)
    }
}

#[cfg(test)]
mod tests {
    use std::cell::RefCell;

    use std::rc::Rc;

    use super::*;
    use crate::processor::{Context, Titles};

    #[test]
    fn parse_parse_correctly() -> ProcessResult<()> {
        let str = "(.len)";
        let splitter = Splitter::from_str(str).unwrap();

        let input = Context::new_with_no_context("test".into());

        assert_eq!(splitter.split_by.get(&input), Some((4).into()));

        Ok(())
    }
    #[test]
    fn parse_fail_if_too_long() -> ProcessResult<()> {
        let str = "(.len)3";
        let err = Splitter::from_str(str).err().unwrap();

        assert!(matches!(err, SelectionParseError::ExpectingEof(_, _)));

        Ok(())
    }

    #[test]
    fn start_will_call_next() -> ProcessResult<()> {
        struct Next(Rc<RefCell<bool>>);
        let data = Rc::new(RefCell::new(false));
        let titles = Titles::default();
        impl Process for Next {
            fn complete(&mut self) -> ProcessResult<()> {
                Ok(())
            }
            fn process(&mut self, _: Context) -> ProcessResult<ProcessDesision> {
                Ok(ProcessDesision::Continue)
            }
            fn start(&mut self, _: Titles) -> ProcessResult<()> {
                *self.0.borrow_mut() = true;
                Ok(())
            }
        }
        let str = ".results";
        let splitter = Splitter::from_str(str).unwrap();
        let next = Box::new(Next(data.clone()));
        let mut splitter = splitter.create_process(next);

        splitter.start(titles)?;

        let binding = data.borrow();
        let data = &*binding;
        assert_eq!(data, &true);

        Ok(())
    }

    #[test]
    fn complete_will_complete_next() -> ProcessResult<()> {
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
        let str = ".results";
        let splitter = Splitter::from_str(str).unwrap();
        let next = Box::new(Next(data.clone()));
        let mut splitter = splitter.create_process(next);

        splitter.complete()?;

        let binding = data.borrow();
        let data = &*binding;
        assert_eq!(data, &true);

        Ok(())
    }

    #[test]
    fn process_will_split_and_call_next() -> ProcessResult<()> {
        struct Next(Rc<RefCell<usize>>);
        let data = Rc::new(RefCell::new(0));
        impl Process for Next {
            fn complete(&mut self) -> ProcessResult<()> {
                Ok(())
            }
            fn process(&mut self, _: Context) -> ProcessResult<ProcessDesision> {
                *self.0.borrow_mut() += 1;
                Ok(ProcessDesision::Continue)
            }
            fn start(&mut self, _: Titles) -> ProcessResult<()> {
                Ok(())
            }
        }
        let str = "(range 10)";
        let splitter = Splitter::from_str(str).unwrap();
        let next = Box::new(Next(data.clone()));
        let mut splitter = splitter.create_process(next);
        let context = Context::new_empty();

        splitter.process(context)?;

        let binding = data.borrow();
        let data = &*binding;
        assert_eq!(data, &10);

        Ok(())
    }
}
