use std::collections::{BTreeMap, VecDeque};
use std::io::Error as IoError;
use std::rc::Rc;
use std::str::FromStr;

use thiserror::Error;

use crate::json_value::JsonValue;
use crate::processor::{Context, Process, Titles};
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
            "" => Direction::Asc,
            "ASC" => Direction::Asc,
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
        match r.next()? {
            Some(ch) => chars.push(ch),
            None => {
                let str = String::from_utf8(chars)?;
                return Ok(str.trim().to_string());
            }
        }
    }
}
impl Sorter {
    pub fn create_processor(&self, next: Box<dyn Process>) -> Box<dyn Process> {
        Box::new(SortProcess {
            data: OrderedData::new(),
            next,
            sort_by: self.sort_by.clone(),
            direction: self.direction,
        })
    }
}
type OrderedData = BTreeMap<JsonValue, VecDeque<Context>>;

struct SortProcess {
    data: OrderedData,
    next: Box<dyn Process>,
    sort_by: Rc<dyn Get>,
    direction: Direction,
}

impl Process for SortProcess {
    fn start(&mut self, titles_so_far: Titles) -> crate::processor::Result {
        self.next.start(titles_so_far)
    }
    fn process(&mut self, context: Context) -> crate::processor::Result {
        if let Some(key) = self.sort_by.get(&context) {
            self.data.entry(key).or_default().push_front(context);
        }
        Ok(())
    }
    fn complete(&mut self) -> crate::processor::Result {
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

#[cfg(test)]
mod tests {
    use std::cell::RefCell;
    use std::ops::Deref;
    use std::rc::Rc;

    use super::*;
    use crate::json_value::JsonValue;
    use crate::processor::Result;

    #[test]
    fn sort_will_read_asc_correctly() -> Result {
        let sorter = Sorter::from_str("1 AsC").unwrap();

        assert_eq!(sorter.direction, Direction::Asc);

        Ok(())
    }

    #[test]
    fn sort_will_read_desc_correctly() -> Result {
        let sorter = Sorter::from_str("1 DeSc").unwrap();

        assert_eq!(sorter.direction, Direction::Desc);

        Ok(())
    }

    #[test]
    fn sort_will_default_to_asc() -> Result {
        let sorter = Sorter::from_str(" 1 ").unwrap();

        assert_eq!(sorter.direction, Direction::Asc);

        Ok(())
    }

    #[test]
    fn sort_will_fail_with_wrong_direction() -> Result {
        let error = Sorter::from_str(" 1 bla").err().unwrap();

        assert_eq!(matches!(error, SorterParserError::UnknownOrder(_)), true);

        Ok(())
    }

    #[test]
    fn sort_asc_will_sort_correctly() -> Result {
        struct Next(Rc<RefCell<Vec<JsonValue>>>);
        let data = Rc::new(RefCell::new(Vec::new()));
        impl Process for Next {
            fn complete(&mut self) -> Result {
                Ok(())
            }
            fn process(&mut self, context: Context) -> Result {
                let value = context.input().deref().clone();
                let mut vec = self.0.borrow_mut();
                vec.push(value);
                Ok(())
            }
            fn start(&mut self, _: Titles) -> Result {
                Ok(())
            }
        }
        let next = Box::new(Next(data.clone()));
        let sorter = Sorter::from_str(". asc").unwrap();
        let mut sorter = sorter.create_processor(next);

        let context = Context::new_with_input("a".into());
        sorter.process(context)?;
        let context = Context::new_with_input("z".into());
        sorter.process(context)?;
        let context = Context::new_with_input("a".into());
        sorter.process(context)?;
        let context = Context::new_with_input("e".into());
        sorter.process(context)?;
        let context = Context::new_with_input(20.into());
        sorter.process(context)?;
        let context = Context::new_with_input(2.into());
        sorter.process(context)?;
        let context = Context::new_with_input(3.4.into());
        sorter.process(context)?;
        let context = Context::new_with_input(false.into());
        sorter.process(context)?;
        let context = Context::new_with_input(JsonValue::Null);
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
                2.into(),
                3.4.into(),
                20.into(),
            ]
        );

        Ok(())
    }

    #[test]
    fn sort_desc_will_sort_correctly() -> Result {
        struct Next(Rc<RefCell<Vec<JsonValue>>>);
        let data = Rc::new(RefCell::new(Vec::new()));
        impl Process for Next {
            fn complete(&mut self) -> Result {
                Ok(())
            }
            fn process(&mut self, context: Context) -> Result {
                let value = context.input().deref().clone();
                let mut vec = self.0.borrow_mut();
                vec.push(value);
                Ok(())
            }
            fn start(&mut self, _: Titles) -> Result {
                Ok(())
            }
        }
        let next = Box::new(Next(data.clone()));
        let sorter = Sorter::from_str(". DESC").unwrap();
        let mut sorter = sorter.create_processor(next);

        let context = Context::new_with_input("a".into());
        sorter.process(context)?;
        let context = Context::new_with_input("z".into());
        sorter.process(context)?;
        let context = Context::new_with_input("a".into());
        sorter.process(context)?;
        let context = Context::new_with_input("e".into());
        sorter.process(context)?;
        let context = Context::new_with_input(20.into());
        sorter.process(context)?;
        let context = Context::new_with_input(2.into());
        sorter.process(context)?;
        let context = Context::new_with_input(3.4.into());
        sorter.process(context)?;
        let context = Context::new_with_input(false.into());
        sorter.process(context)?;
        let context = Context::new_with_input(JsonValue::Null);
        sorter.process(context)?;

        assert_eq!(*data.deref().borrow(), vec![]);

        sorter.complete()?;

        assert_eq!(
            *data.deref().borrow(),
            vec![
                20.into(),
                3.4.into(),
                2.into(),
                "z".into(),
                "e".into(),
                "a".into(),
                "a".into(),
                false.into(),
                JsonValue::Null,
            ]
        );

        Ok(())
    }
}
