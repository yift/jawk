use std::{cmp::Ordering, fmt::Display, num::TryFromIntError};

use indexmap::IndexMap;
use thiserror::Error;

use crate::printer::{JsonPrinter, Print};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum JsonValue {
    Null,
    Boolean(bool),
    String(String),
    Number(NumberValue),
    Object(IndexMap<String, JsonValue>),
    Array(Vec<JsonValue>),
}

impl JsonValue {
    pub fn type_name(&self) -> String {
        match self {
            JsonValue::Null => "null".to_string(),
            JsonValue::Boolean(_) => "boolean".to_string(),
            JsonValue::String(_) => "string".to_string(),
            JsonValue::Number(_) => "number".to_string(),
            JsonValue::Object(_) => "object".to_string(),
            JsonValue::Array(_) => "array".to_string(),
        }
    }

    pub fn null() -> Self {
        JsonValue::Null
    }
    pub fn is_null(&self) -> bool {
        matches!(self, JsonValue::Null)
    }
    pub fn is_number(&self) -> bool {
        matches!(self, JsonValue::Number(_))
    }
    pub fn is_boolean(&self) -> bool {
        matches!(self, JsonValue::Boolean(_))
    }
    pub fn is_string(&self) -> bool {
        matches!(self, JsonValue::String(_))
    }
    pub fn is_array(&self) -> bool {
        matches!(self, JsonValue::Array(_))
    }
    pub fn is_object(&self) -> bool {
        matches!(self, JsonValue::Object(_))
    }

    fn inner_index(&self) -> usize {
        match self {
            JsonValue::Null => 0,
            JsonValue::Boolean(_) => 1,
            JsonValue::String(_) => 2,
            JsonValue::Number(_) => 3,
            JsonValue::Object(_) => 4,
            JsonValue::Array(_) => 5,
        }
    }
}
impl Display for JsonValue {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let printer = JsonPrinter::new(false, true);
        printer.print(f, self)
    }
}

#[derive(Debug, Clone)]
pub enum NumberValue {
    Negative(i64),
    Positive(u64),
    Float(f64),
}

impl NumberValue {
    pub fn type_name(&self) -> String {
        match self {
            NumberValue::Positive(_) => "positive number".to_string(),
            NumberValue::Negative(_) => "negative number".to_string(),
            NumberValue::Float(_) => "float number".to_string(),
        }
    }
}
impl From<String> for JsonValue {
    fn from(str: String) -> Self {
        JsonValue::String(str)
    }
}
impl From<bool> for JsonValue {
    fn from(b: bool) -> Self {
        JsonValue::Boolean(b)
    }
}
impl From<usize> for JsonValue {
    fn from(u: usize) -> Self {
        JsonValue::Number(u.into())
    }
}
impl From<usize> for NumberValue {
    fn from(value: usize) -> Self {
        NumberValue::Positive(value as u64)
    }
}

#[derive(Debug, Error)]
pub enum CastError {
    #[error("Unexpected type. Got `{0}`.")]
    IncorrectType(String),
    #[error("{0}")]
    TryFromIntError(#[from] TryFromIntError),
}

impl TryFrom<NumberValue> for usize {
    type Error = CastError;
    fn try_from(value: NumberValue) -> Result<Self, Self::Error> {
        match value {
            NumberValue::Positive(p) => {
                let size = p.try_into()?;
                Ok(size)
            }
            _ => Err(CastError::IncorrectType(value.type_name())),
        }
    }
}
impl TryFrom<JsonValue> for f64 {
    type Error = CastError;
    fn try_from(value: JsonValue) -> Result<Self, Self::Error> {
        match value {
            JsonValue::Number(NumberValue::Float(f)) => Ok(f),
            JsonValue::Number(NumberValue::Positive(f)) => Ok(f as f64),
            JsonValue::Number(NumberValue::Negative(f)) => Ok(f as f64),
            _ => Err(CastError::IncorrectType(value.type_name())),
        }
    }
}

impl TryFrom<JsonValue> for usize {
    type Error = CastError;
    fn try_from(value: JsonValue) -> Result<Self, Self::Error> {
        match value {
            JsonValue::Number(p) => {
                let size = p.try_into()?;
                Ok(size)
            }
            _ => Err(CastError::IncorrectType(value.type_name())),
        }
    }
}

impl TryFrom<JsonValue> for String {
    type Error = CastError;
    fn try_from(value: JsonValue) -> Result<Self, Self::Error> {
        match value {
            JsonValue::String(s) => Ok(s),
            _ => Err(CastError::IncorrectType(value.type_name())),
        }
    }
}
impl TryFrom<JsonValue> for bool {
    type Error = CastError;
    fn try_from(value: JsonValue) -> Result<Self, Self::Error> {
        match value {
            JsonValue::Boolean(s) => Ok(s),
            _ => Err(CastError::IncorrectType(value.type_name())),
        }
    }
}
impl TryFrom<JsonValue> for NumberValue {
    type Error = CastError;
    fn try_from(value: JsonValue) -> Result<Self, Self::Error> {
        match value {
            JsonValue::Number(s) => Ok(s),
            _ => Err(CastError::IncorrectType(value.type_name())),
        }
    }
}
impl TryFrom<JsonValue> for Vec<JsonValue> {
    type Error = CastError;
    fn try_from(value: JsonValue) -> Result<Self, Self::Error> {
        match value {
            JsonValue::Array(s) => Ok(s),
            _ => Err(CastError::IncorrectType(value.type_name())),
        }
    }
}
impl TryFrom<JsonValue> for IndexMap<String, JsonValue> {
    type Error = CastError;
    fn try_from(value: JsonValue) -> Result<Self, Self::Error> {
        match value {
            JsonValue::Object(s) => Ok(s),
            _ => Err(CastError::IncorrectType(value.type_name())),
        }
    }
}
impl From<IndexMap<String, JsonValue>> for JsonValue {
    fn from(value: IndexMap<String, JsonValue>) -> Self {
        JsonValue::Object(value)
    }
}
impl From<&String> for JsonValue {
    fn from(value: &String) -> Self {
        JsonValue::String(value.clone())
    }
}
impl From<&str> for JsonValue {
    fn from(value: &str) -> Self {
        JsonValue::String(value.to_string())
    }
}
impl From<Vec<JsonValue>> for JsonValue {
    fn from(value: Vec<JsonValue>) -> Self {
        JsonValue::Array(value)
    }
}
impl From<&NumberValue> for f64 {
    fn from(value: &NumberValue) -> Self {
        match *value {
            NumberValue::Float(f) => f,
            NumberValue::Negative(f) => f as f64,
            NumberValue::Positive(f) => f as f64,
        }
    }
}
impl From<NumberValue> for f64 {
    fn from(value: NumberValue) -> Self {
        match value {
            NumberValue::Float(f) => f,
            NumberValue::Negative(f) => f as f64,
            NumberValue::Positive(f) => f as f64,
        }
    }
}
impl From<f64> for JsonValue {
    fn from(value: f64) -> Self {
        if value.fract() == 0.0 {
            if value >= 0.0 && value < (u64::MAX as f64) {
                JsonValue::Number(NumberValue::Positive(value as u64))
            } else if value < 0.0 && value > (i64::MIN as f64) {
                JsonValue::Number(NumberValue::Negative(value as i64))
            } else {
                JsonValue::Number(NumberValue::Float(value))
            }
        } else {
            JsonValue::Number(NumberValue::Float(value))
        }
    }
}

impl PartialEq for NumberValue {
    fn eq(&self, other: &Self) -> bool {
        match self {
            NumberValue::Float(me) => match other {
                NumberValue::Float(other) => me == other,
                NumberValue::Negative(other) => {
                    me.fract() == 0.0 && *me <= 0.0 && (*other as f64) == *me
                }
                NumberValue::Positive(other) => {
                    me.fract() == 0.0 && *me >= 0.0 && (*other as f64) == *me
                }
            },
            NumberValue::Negative(me) => match other {
                NumberValue::Float(other) => {
                    other.fract() == 0.0 && *other <= 0.0 && (*me as f64) == *other
                }
                NumberValue::Negative(other) => me == other,
                NumberValue::Positive(other) => *me == 0 && *other == 0,
            },
            NumberValue::Positive(me) => match other {
                NumberValue::Float(other) => {
                    other.fract() == 0.0 && *other >= 0.0 && (*me as f64) == *other
                }
                NumberValue::Positive(other) => me == other,
                NumberValue::Negative(other) => *me == 0 && *other == 0,
            },
        }
    }
}
impl Eq for NumberValue {}
impl PartialOrd for NumberValue {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for NumberValue {
    fn cmp(&self, other: &Self) -> Ordering {
        let me: f64 = self.into();
        let other: f64 = other.into();
        me.total_cmp(&other)
    }
}
impl PartialOrd for JsonValue {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for JsonValue {
    fn cmp(&self, other: &Self) -> Ordering {
        let my_index = self.inner_index();
        let other_index = other.inner_index();
        let index_cmp = my_index.cmp(&other_index);
        if index_cmp != Ordering::Equal {
            return index_cmp;
        }
        match self {
            JsonValue::Null => Ordering::Equal,
            JsonValue::Boolean(v) => {
                if let JsonValue::Boolean(other) = other {
                    v.cmp(other)
                } else {
                    Ordering::Equal
                }
            }
            JsonValue::Array(v) => {
                if let JsonValue::Array(other) = other {
                    v.cmp(other)
                } else {
                    Ordering::Equal
                }
            }
            JsonValue::Number(v) => {
                if let JsonValue::Number(other) = other {
                    v.cmp(other)
                } else {
                    Ordering::Equal
                }
            }
            JsonValue::String(v) => {
                if let JsonValue::String(other) = other {
                    v.cmp(other)
                } else {
                    Ordering::Equal
                }
            }
            JsonValue::Object(v) => {
                if let JsonValue::Object(other_value) = other {
                    let len_cmp = v.len().cmp(&other_value.len());
                    if len_cmp != Ordering::Equal {
                        return len_cmp;
                    }
                    let mut my_keys: Vec<_> = v.keys().cloned().collect();
                    my_keys.sort();
                    let mut other_keys: Vec<_> = other_value.keys().cloned().collect();
                    other_keys.sort();
                    let keys_cmp = my_keys.cmp(&other_keys);
                    if keys_cmp != Ordering::Equal {
                        return keys_cmp;
                    }
                    let me = format!("{}", self);
                    let other = format!("{}", other);
                    me.cmp(&other)
                } else {
                    Ordering::Equal
                }
            }
        }
    }
}
