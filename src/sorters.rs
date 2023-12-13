use std::collections::{BTreeMap, VecDeque};
use std::io::Error as IoError;
use std::ops::Deref;
use std::{str::FromStr, sync::Arc};

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
    sort_by: Arc<Box<dyn Get>>,
    direction: Direction,
}

impl Process for SortProcess {
    fn start(&mut self, titles_so_far: Titles) -> crate::processor::Result {
        self.next.start(titles_so_far)
    }
    fn process(&mut self, context: Context) -> crate::processor::Result {
        let input = context.input().as_ref().map(|val| val.deref().clone());
        if let Some(key) = self.sort_by.get(&input) {
            self.data.entry(key).or_default().push_back(context);
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
