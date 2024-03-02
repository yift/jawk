use std::collections::{BTreeMap, VecDeque};
use std::io::Error as IoError;
use std::rc::Rc;
use std::str::FromStr;

use thiserror::Error;

use crate::json_value::JsonValue;
use crate::processor::{Context, Process, ProcessDesision, Result as ProcessResult, Titles};
use crate::{
    reader::{from_string, Reader},
    selection::{read_getter, Get, SelectionParseError},
};
use std::io::Read;

#[derive(Debug, Error)]
pub enum SorterParserError {
    #[error("{0}")]
    SelectionParseError(#[from] SelectionParseError),
    #[error("{0}")]
    IoError(#[from] IoError),
    #[error("Only ASC or DESC allowed, {0} is niether.")]
    UnknownOrder(String),
}

#[derive(Clone, Copy, PartialEq, Debug)]
enum Direction {
    Asc,
    Desc,
}

#[derive(Clone)]
pub struct Sorter {
    sort_by: Rc<dyn Get>,
    direction: Direction,
}

impl FromStr for Sorter {
    type Err = SorterParserError;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let source = s.to_string();
        let mut reader = from_string(&source);
        reader.eat_whitespace()?;
        let sort_by = read_getter(&mut reader)?;
        let direction = read_to_eof(&mut reader)?.to_uppercase();
        let direction = match direction.as_str() {
            "" | "ASC" => Direction::Asc,
            "DESC" => Direction::Desc,
            dir => {
                return Err(SorterParserError::UnknownOrder(dir.to_string()));
            }
        };

        Ok(Sorter { sort_by, direction })
    }
}

fn read_to_eof<R: Read>(r: &mut Reader<R>) -> Result<String, SelectionParseError> {
    let mut chars = Vec::new();
    loop {
        if let Some(ch) = r.next()? {
            chars.push(ch)
        } else {
            let str = String::from_utf8(chars)?;
            return Ok(str.trim().to_string());
        }
    }
}
impl Sorter {
    pub fn create_processor(
        &self,
        next: Box<dyn Process>,
        max_size: Option<usize>,
    ) -> Box<dyn Process> {
        Box::new(SortProcess {
            data: OrderedData::new(),
            next,
            sort_by: self.sort_by.clone(),
            direction: self.direction,
            space_left: max_size,
        })
    }
}
type OrderedData = BTreeMap<JsonValue, VecDeque<Context>>;

struct SortProcess {
    data: OrderedData,
    next: Box<dyn Process>,
    sort_by: Rc<dyn Get>,
    direction: Direction,
    space_left: Option<usize>,
}

impl Process for SortProcess {
    fn start(&mut self, titles_so_far: Titles) -> ProcessResult<()> {
        self.next.start(titles_so_far)
    }
    fn process(&mut self, context: Context) -> ProcessResult<ProcessDesision> {
        if let Some(key) = self.sort_by.get(&context) {
            self.data.entry(key).or_default().push_front(context);
            if let Some(space_left) = self.space_left {
                if space_left == 0 {
                    self.remove_last_item();
                } else {
                    self.space_left = Some(space_left - 1);
                }
            }
        }
        Ok(ProcessDesision::Continue)
    }
    fn complete(&mut self) -> ProcessResult<()> {
        match self.direction {
            Direction::Asc => {
                for items in self.data.values_mut() {
                    while let Some(value) = items.pop_back() {
                        self.next.process(value)?;
                    }
                }
            }
            Direction::Desc => {
                for items in self.data.values_mut().rev() {
                    while let Some(value) = items.pop_back() {
                        self.next.process(value)?;
                    }
                }
            }
        }
        self.data.clear();
        self.next.complete()
    }
}
impl SortProcess {
    fn remove_last_item(&mut self) {
        match self.direction {
            Direction::Asc => {
                if let Some(mut last_list) = self.data.last_entry() {
                    let v: &mut VecDeque<_> = last_list.get_mut();
                    v.pop_back();
                    if v.is_empty() {
                        last_list.remove();
                    }
                }
            }
            Direction::Desc => {
                if let Some(mut last_list) = self.data.first_entry() {
                    let v: &mut VecDeque<_> = last_list.get_mut();
                    v.pop_back();
                    if v.is_empty() {
                        last_list.remove();
                    }
                }
            }
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
    fn sort_will_read_asc_correctly() {
        let sorter = Sorter::from_str("1 AsC").unwrap();

        assert_eq!(sorter.direction, Direction::Asc);
    }

    #[test]
    fn sort_will_read_desc_correctly() {
        let sorter = Sorter::from_str("1 DeSc").unwrap();

        assert_eq!(sorter.direction, Direction::Desc);
    }

    #[test]
    fn sort_will_default_to_asc() {
        let sorter = Sorter::from_str(" 1 ").unwrap();

        assert_eq!(sorter.direction, Direction::Asc);
    }

    #[test]
    fn sort_will_fail_with_wrong_direction() {
        let error = Sorter::from_str(" 1 bla").err().unwrap();

        assert!(matches!(error, SorterParserError::UnknownOrder(_)));
    }

    #[test]
    fn sort_asc_will_sort_correctly() -> ProcessResult<()> {
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
        let next = Box::new(Next(data.clone()));
        let sorter = Sorter::from_str(". asc").unwrap();
        let mut sorter = sorter.create_processor(next, None);

        let context = Context::new_with_no_context("a".into());
        sorter.process(context)?;
        let context = Context::new_with_no_context("z".into());
        sorter.process(context)?;
        let context = Context::new_with_no_context("a".into());
        sorter.process(context)?;
        let context = Context::new_with_no_context("e".into());
        sorter.process(context)?;
        let context = Context::new_with_no_context((20).into());
        sorter.process(context)?;
        let context = Context::new_with_no_context((2).into());
        sorter.process(context)?;
        let context = Context::new_with_no_context((3.4).into());
        sorter.process(context)?;
        let context = Context::new_with_no_context(false.into());
        sorter.process(context)?;
        let context = Context::new_with_no_context(JsonValue::Null);
        sorter.process(context)?;

        assert_eq!(*data.deref().borrow(), vec![]);

        sorter.complete()?;

        assert_eq!(
            *data.deref().borrow(),
            vec![
                JsonValue::Null,
                false.into(),
                "a".into(),
                "a".into(),
                "e".into(),
                "z".into(),
                (2).into(),
                (3.4).into(),
                (20).into()
            ]
        );

        Ok(())
    }

    #[test]
    fn sort_desc_will_sort_correctly() -> ProcessResult<()> {
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
        let next = Box::new(Next(data.clone()));
        let sorter = Sorter::from_str(". DESC").unwrap();
        let mut sorter = sorter.create_processor(next, None);

        let context = Context::new_with_no_context("a".into());
        sorter.process(context)?;
        let context = Context::new_with_no_context("z".into());
        sorter.process(context)?;
        let context = Context::new_with_no_context("a".into());
        sorter.process(context)?;
        let context = Context::new_with_no_context("e".into());
        sorter.process(context)?;
        let context = Context::new_with_no_context((20).into());
        sorter.process(context)?;
        let context = Context::new_with_no_context((2).into());
        sorter.process(context)?;
        let context = Context::new_with_no_context((3.4).into());
        sorter.process(context)?;
        let context = Context::new_with_no_context(false.into());
        sorter.process(context)?;
        let context = Context::new_with_no_context(JsonValue::Null);
        sorter.process(context)?;

        assert_eq!(*data.deref().borrow(), vec![]);

        sorter.complete()?;

        assert_eq!(
            *data.deref().borrow(),
            vec![
                (20).into(),
                (3.4).into(),
                (2).into(),
                "z".into(),
                "e".into(),
                "a".into(),
                "a".into(),
                false.into(),
                JsonValue::Null
            ]
        );

        Ok(())
    }

    #[test]
    fn sort_with_mask_size_will_not_process_more_that_it_needs_to() -> ProcessResult<()> {
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
        let next = Box::new(Next(data.clone()));
        let sorter = Sorter::from_str(". asc").unwrap();
        let mut sorter = sorter.create_processor(next, Some(4));

        let context = Context::new_with_no_context("a".into());
        sorter.process(context)?;
        let context = Context::new_with_no_context("z".into());
        sorter.process(context)?;
        let context = Context::new_with_no_context("a".into());
        sorter.process(context)?;
        let context = Context::new_with_no_context("e".into());
        sorter.process(context)?;
        let context = Context::new_with_no_context((20).into());
        sorter.process(context)?;
        let context = Context::new_with_no_context((2).into());
        sorter.process(context)?;
        let context = Context::new_with_no_context((3.4).into());
        sorter.process(context)?;
        let context = Context::new_with_no_context(false.into());
        sorter.process(context)?;
        let context = Context::new_with_no_context(JsonValue::Null);
        sorter.process(context)?;

        assert_eq!(*data.deref().borrow(), vec![]);

        sorter.complete()?;

        assert_eq!(
            *data.deref().borrow(),
            vec![JsonValue::Null, false.into(), "a".into(), "a".into()]
        );

        Ok(())
    }

    #[test]
    fn sort_desc_with_mask_size_will_not_process_more_that_it_needs_to() -> ProcessResult<()> {
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
        let next = Box::new(Next(data.clone()));
        let sorter = Sorter::from_str(". desc").unwrap();
        let mut sorter = sorter.create_processor(next, Some(4));

        let context = Context::new_with_no_context("a".into());
        sorter.process(context)?;
        let context = Context::new_with_no_context("z".into());
        sorter.process(context)?;
        let context = Context::new_with_no_context("a".into());
        sorter.process(context)?;
        let context = Context::new_with_no_context("e".into());
        sorter.process(context)?;
        let context = Context::new_with_no_context((20).into());
        sorter.process(context)?;
        let context = Context::new_with_no_context((2).into());
        sorter.process(context)?;
        let context = Context::new_with_no_context((3.4).into());
        sorter.process(context)?;
        let context = Context::new_with_no_context(false.into());
        sorter.process(context)?;
        let context = Context::new_with_no_context(JsonValue::Null);
        sorter.process(context)?;

        assert_eq!(*data.deref().borrow(), vec![]);

        sorter.complete()?;

        assert_eq!(
            *data.deref().borrow(),
            vec![(20).into(), (3.4).into(), (2).into(), "z".into()]
        );

        Ok(())
    }
}
