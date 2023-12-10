use std::collections::BTreeMap;
use std::io::Error as IoError;
use std::{str::FromStr, sync::Arc};

use thiserror::Error;

use crate::json_value::JsonValue;
use crate::output::Output;
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
    sort_by: Arc<Box<dyn Get>>,
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
        let sort_by = Arc::new(sort_by);

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
    pub fn start(&self, output: Box<dyn Output>) -> Box<dyn Output> {
        let data = BTreeMap::new();
        let sorter = ActiveSorter {
            data,
            output,
            sort_by: self.sort_by.clone(),
            direction: self.direction,
        };
        Box::new(sorter)
    }
}
type SortedData = BTreeMap<JsonValue, Vec<(JsonValue, Vec<Option<JsonValue>>)>>;
struct ActiveSorter {
    data: SortedData,
    output: Box<dyn Output>,
    sort_by: Arc<Box<dyn Get>>,
    direction: Direction,
}

impl Output for ActiveSorter {
    fn output_row(&mut self, value: &JsonValue, row: Vec<Option<JsonValue>>) -> std::fmt::Result {
        if let Some(key) = self.sort_by.get(&Some(value.clone())) {
            self.data.entry(key).or_default().push((value.clone(), row));
        }
        Ok(())
    }
    fn done(&mut self) -> std::fmt::Result {
        match self.direction {
            Direction::Asc => {
                for items in self.data.values() {
                    for (value, row) in items {
                        self.output.output_row(value, row.clone())?;
                    }
                }
            }
            Direction::Desc => {
                for items in self.data.values().rev() {
                    for (value, row) in items {
                        self.output.output_row(value, row.clone())?;
                    }
                }
            }
        }
        self.output.done()
    }
    fn without_titles(&self) -> Option<Box<dyn Output>> {
        None
    }
}
#[cfg(test)]
mod tests {
    use std::sync::Mutex;

    use super::*;

    #[test]
    fn test_from_str_valid_name_choose_asc() -> Result<(), SorterParserError> {
        let str = ".a";

        let sorter = Sorter::from_str(str)?;

        assert_eq!(sorter.direction, Direction::Asc);

        Ok(())
    }

    #[test]
    fn test_from_str_valid_name_and_asc_choose_asc() -> Result<(), SorterParserError> {
        let str = ".a asc";

        let sorter = Sorter::from_str(str)?;

        assert_eq!(sorter.direction, Direction::Asc);

        Ok(())
    }

    #[test]
    fn test_from_str_valid_name_and_upercase_asc_choose_asc() -> Result<(), SorterParserError> {
        let str = ".a aSc";

        let sorter = Sorter::from_str(str)?;

        assert_eq!(sorter.direction, Direction::Asc);

        Ok(())
    }

    #[test]
    fn test_from_str_valid_name_and_desc_choose_desc() -> Result<(), SorterParserError> {
        let str = ".a DESC";

        let sorter = Sorter::from_str(str)?;

        assert_eq!(sorter.direction, Direction::Desc);

        Ok(())
    }

    #[test]
    fn test_from_str_valid_name_and_upercase_desc_choose_desc() -> Result<(), SorterParserError> {
        let str = ".a DEsC";

        let sorter = Sorter::from_str(str)?;

        assert_eq!(sorter.direction, Direction::Desc);

        Ok(())
    }

    #[test]
    fn test_from_str_invalid_name_return_error() -> Result<(), SorterParserError> {
        let str = "(";

        let result = Sorter::from_str(str);

        assert!(matches!(result, Err(_)));

        Ok(())
    }

    #[test]
    fn test_from_str_invalid_sort() -> Result<(), SorterParserError> {
        let str = ".a bla";

        let result = Sorter::from_str(str);

        assert!(matches!(result, Err(_)));

        Ok(())
    }

    #[test]
    fn test_from_str_something_after_sort() -> Result<(), SorterParserError> {
        let str = ".a desci";

        let result = Sorter::from_str(str);

        assert!(matches!(result, Err(_)));

        Ok(())
    }

    #[test]
    fn test_from_str_something_after_sort_with_space() -> Result<(), SorterParserError> {
        let str = ".a desc .a";

        let result = Sorter::from_str(str);

        assert!(matches!(result, Err(_)));

        Ok(())
    }

    #[test]
    fn test_asc_sort_correctly() -> Result<(), SorterParserError> {
        let str = ". asc";

        let sorter = Sorter::from_str(str)?;
        let got_values = Arc::new(Mutex::new(Vec::new()));
        struct MockOutput {
            got_values: Arc<Mutex<Vec<usize>>>,
        }
        impl Output for MockOutput {
            fn output_row(
                &mut self,
                value: &JsonValue,
                _: Vec<Option<JsonValue>>,
            ) -> std::fmt::Result {
                if let Ok(num) = value.clone().try_into() {
                    let mut got_values = self.got_values.lock().unwrap();
                    got_values.push(num);
                }
                Ok(())
            }
            fn without_titles(&self) -> Option<Box<dyn Output>> {
                None
            }
        }

        let mut pipe = sorter.start(Box::new(MockOutput {
            got_values: got_values.clone(),
        }));
        pipe.output_row(&JsonValue::Null, vec![]).unwrap();
        pipe.output_row(&(20).into(), vec![]).unwrap();
        pipe.output_row(&(21).into(), vec![]).unwrap();
        pipe.output_row(&(15).into(), vec![]).unwrap();
        pipe.output_row(&(15).into(), vec![]).unwrap();
        pipe.output_row(&(40).into(), vec![]).unwrap();
        pipe.output_row(&(12).into(), vec![]).unwrap();
        pipe.done().unwrap();

        let lst = got_values.lock().unwrap();
        assert_eq!(*lst, vec![12, 15, 15, 20, 21, 40]);

        Ok(())
    }

    #[test]
    fn test_desc_sort_correctly() -> Result<(), SorterParserError> {
        let str = ". desc";

        let sorter = Sorter::from_str(str)?;
        let got_values = Arc::new(Mutex::new(Vec::new()));
        struct MockOutput {
            got_values: Arc<Mutex<Vec<usize>>>,
        }
        impl Output for MockOutput {
            fn output_row(
                &mut self,
                value: &JsonValue,
                _: Vec<Option<JsonValue>>,
            ) -> std::fmt::Result {
                if let Ok(num) = value.clone().try_into() {
                    let mut got_values = self.got_values.lock().unwrap();
                    got_values.push(num);
                }
                Ok(())
            }
            fn without_titles(&self) -> Option<Box<dyn Output>> {
                None
            }
        }

        let mut pipe = sorter.start(Box::new(MockOutput {
            got_values: got_values.clone(),
        }));
        pipe.output_row(&JsonValue::Null, vec![]).unwrap();
        pipe.output_row(&(20).into(), vec![]).unwrap();
        pipe.output_row(&(21).into(), vec![]).unwrap();
        pipe.output_row(&(15).into(), vec![]).unwrap();
        pipe.output_row(&(15).into(), vec![]).unwrap();
        pipe.output_row(&(40).into(), vec![]).unwrap();
        pipe.output_row(&(12).into(), vec![]).unwrap();
        pipe.done().unwrap();

        let lst = got_values.lock().unwrap();
        assert_eq!(*lst, vec![40, 21, 20, 15, 15, 12]);

        Ok(())
    }

    #[test]
    fn test_without_titles_return_nothing() -> Result<(), SorterParserError> {
        let str = ".a";
        let sorter = Sorter::from_str(str)?;
        struct MockOutput;
        impl Output for MockOutput {
            fn output_row(&mut self, _: &JsonValue, _: Vec<Option<JsonValue>>) -> std::fmt::Result {
                Ok(())
            }
            fn without_titles(&self) -> Option<Box<dyn Output>> {
                None
            }
        }
        let first_object = MockOutput {};
        let started = sorter.start(Box::new(first_object));

        let ret = started.without_titles();

        assert_eq!(ret.is_none(), true);
        Ok(())
    }
}
