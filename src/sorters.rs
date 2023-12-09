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

#[derive(Clone, Copy)]
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
            dir => return Err(SorterParserError::UnknownOrder(dir.to_string())),
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
            self.data.entry(key).or_default().push((value.clone(), row))
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

        Ok(())
    }
    fn without_titles(&self) -> Option<Box<dyn Output>> {
        None
    }
}
