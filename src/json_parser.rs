use std::io::Error as IoError;
use std::num::{ParseFloatError, ParseIntError};
use std::{collections::HashMap, io::Read, string::FromUtf8Error};

use thiserror::Error;

use crate::json_value::{JsonValue, NumberValue};
use crate::reader::{Location, Reader};

pub type Result<T> = std::result::Result<T, JsonParserError>;

pub trait JsonParser {
    fn next_json_value(&mut self) -> Result<Option<JsonValue>>;
}

trait JsonParserUtils {
    fn read_reserved_word<const N: usize>(&mut self, chars: &[u8; N], word: &str) -> Result<()>;
    fn read_true(&mut self) -> Result<JsonValue>;
    fn read_false(&mut self) -> Result<JsonValue>;
    fn read_null(&mut self) -> Result<JsonValue>;
    fn read_array(&mut self) -> Result<JsonValue>;
    fn read_object(&mut self) -> Result<JsonValue>;
    fn read_number(&mut self) -> Result<JsonValue>;
    fn read_string(&mut self) -> Result<JsonValue>;
}

impl<R: Read> JsonParserUtils for Reader<R> {
    #[inline]
    fn read_reserved_word<const N: usize>(&mut self, chars: &[u8; N], word: &str) -> Result<()> {
        for expected in chars {
            match self.next()? {
                Some(ch) => {
                    if ch != *expected {
                        return Err(JsonParserError::IncompleteReservedWord(
                            self.where_am_i(),
                            word.to_string(),
                            ch as char,
                            *expected as char,
                        ));
                    }
                }
                None => return Err(JsonParserError::UnexpectedEof(self.where_am_i())),
            }
        }
        self.next()?;
        Ok(())
    }

    #[inline]
    fn read_true(&mut self) -> Result<JsonValue> {
        self.read_reserved_word(&[b'r', b'u', b'e'], "true")?;
        Ok(JsonValue::Boolean(true))
    }

    #[inline]
    fn read_false(&mut self) -> Result<JsonValue> {
        self.read_reserved_word(&[b'a', b'l', b's', b'e'], "false")?;
        Ok(JsonValue::Boolean(false))
    }

    #[inline]
    fn read_null(&mut self) -> Result<JsonValue> {
        self.read_reserved_word(&[b'u', b'l', b'l'], "null")?;
        Ok(JsonValue::Null)
    }
    #[inline]
    fn read_array(&mut self) -> Result<JsonValue> {
        self.next()?;
        self.eat_whitespace()?;
        if self.peek()? == Some(b']') {
            self.next()?;
            return Ok(JsonValue::Array(vec![]));
        }
        let mut array = Vec::new();
        loop {
            let value = match self.next_json_value()? {
                Some(value) => value,
                None => return Err(JsonParserError::UnexpectedEof(self.where_am_i())),
            };
            array.push(value);
            self.eat_whitespace()?;
            match self.peek()? {
                Some(b']') => {
                    self.next()?;
                    return Ok(JsonValue::Array(array));
                }
                Some(b',') => {
                    self.next()?;
                }
                Some(ch) => {
                    return Err(create_unexpected_character(self, ch, [',', ']']));
                }
                None => {
                    return Err(JsonParserError::UnexpectedEof(self.where_am_i()));
                }
            }
        }
    }
    #[inline]
    fn read_object(&mut self) -> Result<JsonValue> {
        self.next()?;
        self.eat_whitespace()?;
        let mut map = HashMap::new();
        if self.peek()? == Some(b'}') {
            self.next()?;
            return Ok(JsonValue::Object(map));
        }
        loop {
            match self.next_json_value()? {
                Some(value) => {
                    if let JsonValue::String(key) = value {
                        self.eat_whitespace()?;
                        match self.peek()? {
                            Some(b':') => {}
                            None => {
                                return Err(JsonParserError::UnexpectedEof(self.where_am_i()));
                            }
                            Some(ch) => {
                                return Err(create_unexpected_character(self, ch, [':']));
                            }
                        }
                        self.next()?;
                        if let Some(value) = self.next_json_value()? {
                            map.insert(key, value);
                        } else {
                            return Err(JsonParserError::UnexpectedEof(self.where_am_i()));
                        }
                    } else {
                        return Err(JsonParserError::StringKeyMissing(
                            self.where_am_i(),
                            value.type_name(),
                        ));
                    }
                }
                _ => {
                    return Err(JsonParserError::UnexpectedEof(self.where_am_i()));
                }
            }
            self.eat_whitespace()?;
            match self.peek()? {
                Some(b'}') => {
                    self.next()?;
                    return Ok(JsonValue::Object(map));
                }
                Some(b',') => {
                    self.next()?;
                }
                Some(ch) => {
                    return Err(create_unexpected_character(self, ch, [',', '}']));
                }
                None => {
                    return Err(JsonParserError::UnexpectedEof(self.where_am_i()));
                }
            }
        }
    }

    #[inline]
    fn read_number(&mut self) -> Result<JsonValue> {
        let mut chars = Vec::new();
        let negative = if self.peek()? == Some(b'-') {
            if self.next()?.is_none() {
                return Err(JsonParserError::UnexpectedEof(self.where_am_i()));
            }
            chars.push(b'-');
            true
        } else {
            false
        };
        self.read_digits(&mut chars)?;
        let mut double = false;
        if self.peek()? == Some(b'.') {
            double = true;
            self.next()?;
            chars.push(b'.');
            self.read_digits(&mut chars)?;
        };
        if self.peek()? == Some(b'e' | b'E') {
            double = true;
            chars.push(b'E');
            self.next()?;
            match self.peek()? {
                Some(b'-') => {
                    chars.push(b'-');
                    self.next()?;
                }
                Some(b'+') => {
                    self.next()?;
                }
                _ => {}
            }
            self.read_digits(&mut chars)?;
        };

        let str = match String::from_utf8(chars) {
            Ok(chars) => chars,
            Err(e) => return Err(JsonParserError::StringUtfError(self.where_am_i(), e)),
        };

        if double {
            match str.parse::<f64>() {
                Ok(f) => Ok(JsonValue::Number(NumberValue::Float(f))),
                Err(e) => Err(JsonParserError::NumberParseFloatError(self.where_am_i(), e)),
            }
        } else if negative {
            match str.parse::<i64>() {
                Ok(i) => Ok(JsonValue::Number(NumberValue::Negative(i))),
                Err(e) => Err(JsonParserError::NumberParseIntError(self.where_am_i(), e)),
            }
        } else {
            match str.parse::<u64>() {
                Ok(u) => Ok(JsonValue::Number(NumberValue::Positive(u))),
                Err(e) => Err(JsonParserError::NumberParseIntError(self.where_am_i(), e)),
            }
        }
    }

    #[inline]
    fn read_string(&mut self) -> Result<JsonValue> {
        let mut chars = Vec::new();
        loop {
            match self.next()? {
                None => {
                    return Err(JsonParserError::UnexpectedEof(self.where_am_i()));
                }
                Some(b'\"') => {
                    self.next()?;
                    match String::from_utf8(chars) {
                        Ok(str) => {
                            return Ok(JsonValue::String(str));
                        }
                        Err(e) => {
                            return Err(JsonParserError::StringUtfError(self.where_am_i(), e))
                        }
                    }
                }
                Some(b'\\') => match self.next()? {
                    None => return Err(JsonParserError::UnexpectedEof(self.where_am_i())),
                    Some(b'\"') => chars.push(b'\"'),
                    Some(b'\\') => chars.push(b'\\'),
                    Some(b'/') => chars.push(b'/'),
                    Some(b'b') => chars.push(0x08),
                    Some(b'f') => chars.push(0x0C),
                    Some(b'n') => chars.push(b'\n'),
                    Some(b'r') => chars.push(b'\r'),
                    Some(b't') => chars.push(b'\t'),
                    Some(b'u') => {
                        let mut chr: u32 = 0;
                        for _ in 0..4 {
                            match self.next()? {
                                None => {
                                    return Err(JsonParserError::UnexpectedEof(self.where_am_i()))
                                }
                                Some(c) => {
                                    let d = match c {
                                        b'0'..=b'9' => (c - b'0') as u32,
                                        b'a'..=b'f' => (c - b'a' + 10) as u32,
                                        b'A'..=b'F' => (c - b'A' + 10) as u32,
                                        ch => {
                                            let mut expected: Vec<char> = ('0'..='9').collect();
                                            let letters: Vec<char> = ('a'..='f').collect();
                                            expected.extend(letters);
                                            let letters: Vec<char> = ('A'..='F').collect();
                                            expected.extend(letters);
                                            return Err(create_unexpected_character(
                                                self, ch, expected,
                                            ));
                                        }
                                    };
                                    chr = (chr << 4) | d;
                                }
                            };
                        }
                        match char::from_u32(chr) {
                            Some(ch) => {
                                let mut buffer = [0; 4];
                                let str = ch.encode_utf8(&mut buffer);
                                for b in str.as_bytes() {
                                    chars.push(*b);
                                }
                            }
                            None => {
                                return Err(JsonParserError::InvalidChacterHex(
                                    self.where_am_i(),
                                    chr,
                                ))
                            }
                        }
                    }
                    Some(ch) => {
                        return Err(create_unexpected_character(
                            self,
                            ch,
                            ['\"', '\\', '/', 'b', 'f', 'n', 'r', 't', 'u'],
                        ))
                    }
                },
                Some(c) => {
                    chars.push(c);
                }
            }
        }
    }
}

impl<R: Read> JsonParser for Reader<R> {
    #[inline]
    fn next_json_value(&mut self) -> Result<Option<JsonValue>> {
        self.eat_whitespace()?;

        match self.peek()? {
            None => Ok(None),
            Some(b't') => Ok(Some(self.read_true()?)),
            Some(b'f') => Ok(Some(self.read_false()?)),
            Some(b'n') => Ok(Some(self.read_null()?)),
            Some(b'\"') => Ok(Some(self.read_string()?)),
            Some(b'-' | b'0'..=b'9') => Ok(Some(self.read_number()?)),
            Some(b'[') => Ok(Some(self.read_array()?)),
            Some(b'{') => Ok(Some(self.read_object()?)),
            Some(ch) => {
                self.next()?;
                let mut expected = vec!['n', 't', 'f', '\"', '-', '[', '{'];
                let digits: Vec<char> = ('0'..='9').collect();
                expected.extend(digits);
                Err(create_unexpected_character(self, ch, expected))
            }
        }
    }
}

fn create_unexpected_character<R: Read, T: IntoIterator<Item = char>>(
    reader: &Reader<R>,
    ch: u8,
    expected: T,
) -> JsonParserError {
    JsonParserError::UnexpectedCharacter(
        reader.where_am_i(),
        ch as char,
        expected
            .into_iter()
            .map(|c| format!("{}", c))
            .collect::<Vec<_>>()
            .join(", "),
    )
}

#[derive(Debug, Error)]
pub enum JsonParserError {
    #[error("{0}")]
    IoError(#[from] IoError),
    #[error("{0}: {1}")]
    StringUtfError(Location, FromUtf8Error),
    #[error("{0}: {1}")]
    NumberParseIntError(Location, ParseIntError),
    #[error("{0}: {1}")]
    NumberParseFloatError(Location, ParseFloatError),
    #[error(
        "{0}: Reserved word '{1}' started but was not completed, got '{2}', should have been '{3}'"
    )]
    IncompleteReservedWord(Location, String, char, char),
    #[error("{0}: Got character '{1}', expecting one of [{2}]")]
    UnexpectedCharacter(Location, char, String),
    #[error("{0}: unkonw character with hex: {1:#04x}")]
    InvalidChacterHex(Location, u32),
    #[error("{0}: Unexpected end of file")]
    UnexpectedEof(Location),
    #[error("{0}: Only string keys are supported, not keys of type: {1}")]
    StringKeyMissing(Location, String),
}

impl JsonParserError {
    pub fn can_recover(&self) -> bool {
        !matches!(self, JsonParserError::IoError(_))
    }
}
