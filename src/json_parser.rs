use std::io::Error as IoError;
use std::num::{ParseFloatError, ParseIntError};
use std::{io::Read, string::FromUtf8Error};

use indexmap::IndexMap;
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
                None => {
                    return Err(JsonParserError::UnexpectedEof(self.where_am_i()));
                }
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
            let Some(value) = self.next_json_value()? else {
                return Err(JsonParserError::UnexpectedEof(self.where_am_i()));
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
        let mut map = IndexMap::new();
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
        }
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
        }

        let str = match String::from_utf8(chars) {
            Ok(chars) => chars,
            Err(e) => {
                return Err(JsonParserError::StringUtfError(self.where_am_i(), e));
            }
        };

        if double {
            match str.parse::<f64>() {
                Ok(f) => Ok(f.into()),
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
                            return Err(JsonParserError::StringUtfError(self.where_am_i(), e));
                        }
                    }
                }
                Some(b'\\') => match self.next()? {
                    None => {
                        return Err(JsonParserError::UnexpectedEof(self.where_am_i()));
                    }
                    Some(b'\"') => chars.push(b'\"'),
                    Some(b'\\') => chars.push(b'\\'),
                    Some(b'/') => chars.push(b'/'),
                    Some(b'b') => chars.push(0x08),
                    Some(b'f') => chars.push(0x0c),
                    Some(b'n') => chars.push(b'\n'),
                    Some(b'r') => chars.push(b'\r'),
                    Some(b't') => chars.push(b'\t'),
                    Some(b'u') => {
                        let mut chr: u32 = 0;
                        for _ in 0..4 {
                            match self.next()? {
                                None => {
                                    return Err(JsonParserError::UnexpectedEof(self.where_am_i()));
                                }
                                Some(c) => {
                                    let d = match c {
                                        b'0'..=b'9' => u32::from(c - b'0'),
                                        b'a'..=b'f' => u32::from(c - b'a' + 10),
                                        b'A'..=b'F' => u32::from(c - b'A' + 10),
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
                                ));
                            }
                        }
                    }
                    Some(ch) => {
                        return Err(create_unexpected_character(
                            self,
                            ch,
                            ['\"', '\\', '/', 'b', 'f', 'n', 'r', 't', 'u'],
                        ));
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
            .map(|c| format!("{c}"))
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

#[cfg(test)]
mod tests {
    use crate::reader::from_string;

    use super::*;

    #[test]
    fn parse_null() -> Result<()> {
        let str = "null".to_string();
        let mut reader = from_string(&str);

        assert_eq!(reader.next_json_value()?, Some(JsonValue::Null));

        Ok(())
    }

    #[test]
    fn parse_string() -> Result<()> {
        let str = "\"null\"".to_string();
        let mut reader = from_string(&str);

        assert_eq!(
            reader.next_json_value()?,
            Some(JsonValue::String("null".into()))
        );

        Ok(())
    }

    #[test]
    fn parse_true() -> Result<()> {
        let str = "true".to_string();
        let mut reader = from_string(&str);

        assert_eq!(reader.next_json_value()?, Some(JsonValue::Boolean(true)));

        Ok(())
    }

    #[test]
    fn parse_false() -> Result<()> {
        let str = "false".to_string();
        let mut reader = from_string(&str);

        assert_eq!(reader.next_json_value()?, Some(JsonValue::Boolean(false)));

        Ok(())
    }

    #[test]
    fn parse_array() -> Result<()> {
        let str = "[false, 1]".to_string();
        let mut reader = from_string(&str);

        assert_eq!(
            reader.next_json_value()?,
            Some(JsonValue::Array(vec![
                JsonValue::Boolean(false),
                JsonValue::Number(NumberValue::Positive(1)),
            ]))
        );

        Ok(())
    }

    #[test]
    fn parse_object() -> Result<()> {
        let str = "{\"key\": \"value\"}".to_string();
        let mut reader = from_string(&str);

        let mut expected = IndexMap::new();
        expected.insert("key".into(), JsonValue::String("value".into()));
        assert_eq!(reader.next_json_value()?, Some(JsonValue::Object(expected)));

        Ok(())
    }

    #[test]
    fn parse_float() -> Result<()> {
        let str = "0.44".to_string();
        let mut reader = from_string(&str);

        assert_eq!(
            reader.next_json_value()?,
            Some(JsonValue::Number(NumberValue::Float(0.44)))
        );

        Ok(())
    }

    #[test]
    fn parse_negative_float() -> Result<()> {
        let str = "-0.44".to_string();
        let mut reader = from_string(&str);

        assert_eq!(
            reader.next_json_value()?,
            Some(JsonValue::Number(NumberValue::Float(-0.44)))
        );

        Ok(())
    }

    #[test]
    fn parse_exp_float() -> Result<()> {
        let str = "1e-4".to_string();
        let mut reader = from_string(&str);

        assert_eq!(
            reader.next_json_value()?,
            Some(JsonValue::Number(NumberValue::Float(0.0001)))
        );

        Ok(())
    }

    #[test]
    fn parse_positive_exp_float() -> Result<()> {
        let str = "1e+2".to_string();
        let mut reader = from_string(&str);

        assert_eq!(
            reader.next_json_value()?,
            Some(JsonValue::Number(NumberValue::Float(100.0)))
        );

        Ok(())
    }

    #[test]
    fn parse_negative_exp_float() -> Result<()> {
        let str = "-1e4".to_string();
        let mut reader = from_string(&str);

        assert_eq!(
            reader.next_json_value()?,
            Some(JsonValue::Number(NumberValue::Float(-10000.0)))
        );

        Ok(())
    }

    #[test]
    fn parse_int() -> Result<()> {
        let str = "100".to_string();
        let mut reader = from_string(&str);

        assert_eq!(
            reader.next_json_value()?,
            Some(JsonValue::Number(NumberValue::Positive(100)))
        );

        Ok(())
    }

    #[test]
    fn parse_zero() -> Result<()> {
        let str = "0".to_string();
        let mut reader = from_string(&str);

        assert_eq!(
            reader.next_json_value()?,
            Some(JsonValue::Number(NumberValue::Positive(0)))
        );

        Ok(())
    }

    #[test]
    fn parse_negative_int() -> Result<()> {
        let str = "-100".to_string();
        let mut reader = from_string(&str);

        assert_eq!(
            reader.next_json_value()?,
            Some(JsonValue::Number(NumberValue::Negative(-100)))
        );

        Ok(())
    }

    #[test]
    fn valid_escpaes() -> Result<()> {
        let str = "\"\\\"\\\\\\/\\b\\f\\n\\r\\t\\u263a\\u263A\"".to_string();
        let mut reader = from_string(&str);

        assert_eq!(
            reader.next_json_value()?,
            Some(JsonValue::String(
                "\"\\/\u{0008}\u{000c}\n\r\t\u{263A}\u{263A}".into()
            ))
        );

        Ok(())
    }

    #[test]
    fn bad_reserve_word() {
        let str = "falke".to_string();
        let mut reader = from_string(&str);

        assert!(matches!(
            reader.next_json_value(),
            Err(JsonParserError::IncompleteReservedWord(_, _, _, _))
        ));
    }

    #[test]
    fn unexpected_end_reserve_word() {
        let str = "fal".to_string();
        let mut reader = from_string(&str);

        assert!(matches!(
            reader.next_json_value(),
            Err(JsonParserError::UnexpectedEof(_))
        ));
    }

    #[test]
    fn never_ended_array() {
        let str = "[1, 2".to_string();
        let mut reader = from_string(&str);

        assert!(matches!(
            reader.next_json_value(),
            Err(JsonParserError::UnexpectedEof(_))
        ));
    }

    #[test]
    fn never_ended_array_with_comma() {
        let str = "[1, 2, ".to_string();
        let mut reader = from_string(&str);

        assert!(matches!(
            reader.next_json_value(),
            Err(JsonParserError::UnexpectedEof(_))
        ));
    }

    #[test]
    fn array_with_no_comma() {
        let str = "[1 2]".to_string();
        let mut reader = from_string(&str);

        assert!(matches!(
            reader.next_json_value(),
            Err(JsonParserError::UnexpectedCharacter(_, _, _))
        ));
    }

    #[test]
    fn missing_colon_after_key() {
        let str = "{\"key\" 1}".to_string();
        let mut reader = from_string(&str);

        assert!(matches!(
            reader.next_json_value(),
            Err(JsonParserError::UnexpectedCharacter(_, _, _))
        ));
    }

    #[test]
    fn noting_after_key() {
        let str = "{\"key\"".to_string();
        let mut reader = from_string(&str);

        assert!(matches!(
            reader.next_json_value(),
            Err(JsonParserError::UnexpectedEof(_))
        ));
    }

    #[test]
    fn noting_after_colon() {
        let str = "{\"key\" :".to_string();
        let mut reader = from_string(&str);

        assert!(matches!(
            reader.next_json_value(),
            Err(JsonParserError::UnexpectedEof(_))
        ));
    }

    #[test]
    fn not_a_string_key() {
        let str = "{1 : 2}".to_string();
        let mut reader = from_string(&str);

        assert!(matches!(
            reader.next_json_value(),
            Err(JsonParserError::StringKeyMissing(_, _))
        ));
    }

    #[test]
    fn missing_next_key() {
        let str = "{\"1\" : 2, ".to_string();
        let mut reader = from_string(&str);

        assert!(matches!(
            reader.next_json_value(),
            Err(JsonParserError::UnexpectedEof(_))
        ));
    }

    #[test]
    fn eof_after_value() {
        let str = "{\"1\" : 2".to_string();
        let mut reader = from_string(&str);

        assert!(matches!(
            reader.next_json_value(),
            Err(JsonParserError::UnexpectedEof(_))
        ));
    }

    #[test]
    fn unexpected_char_after_value() {
        let str = "{\"1\" : 2 44".to_string();
        let mut reader = from_string(&str);

        assert!(matches!(
            reader.next_json_value(),
            Err(JsonParserError::UnexpectedCharacter(_, _, _))
        ));
    }

    #[test]
    fn eof_after_minus() {
        let str = "-".to_string();
        let mut reader = from_string(&str);

        assert!(matches!(
            reader.next_json_value(),
            Err(JsonParserError::UnexpectedEof(_))
        ));
    }

    #[test]
    fn invalid_number() {
        let str = "111111111111111111111111111111119999999999999999999999".to_string();
        let mut reader = from_string(&str);

        assert!(matches!(
            reader.next_json_value(),
            Err(JsonParserError::NumberParseIntError(_, _))
        ));
    }

    #[test]
    fn invalid_negative_number() {
        let str = "-111111111111111111111111111111119999999999999999999999".to_string();
        let mut reader = from_string(&str);

        assert!(matches!(
            reader.next_json_value(),
            Err(JsonParserError::NumberParseIntError(_, _))
        ));
    }

    #[test]
    fn never_ending_string() {
        let str = "\"test".to_string();
        let mut reader = from_string(&str);

        assert!(matches!(
            reader.next_json_value(),
            Err(JsonParserError::UnexpectedEof(_))
        ));
    }

    #[test]
    fn never_ending_escape() {
        let str = "\"\\".to_string();
        let mut reader = from_string(&str);

        assert!(matches!(
            reader.next_json_value(),
            Err(JsonParserError::UnexpectedEof(_))
        ));
    }

    #[test]
    fn invalid_hex() {
        let str = "\"\\u263P\"".to_string();
        let mut reader = from_string(&str);

        assert!(matches!(
            reader.next_json_value(),
            Err(JsonParserError::UnexpectedCharacter(_, _, _))
        ));
    }

    #[test]
    fn incomplete_hex() {
        let str = "\"\\u263".to_string();
        let mut reader = from_string(&str);

        assert!(matches!(
            reader.next_json_value(),
            Err(JsonParserError::UnexpectedEof(_))
        ));
    }

    #[test]
    fn incomplete_uncide() {
        let str = "\"\\uD807\"".to_string();
        let mut reader = from_string(&str);

        assert!(matches!(
            reader.next_json_value(),
            Err(JsonParserError::InvalidChacterHex(_, _))
        ));
    }

    #[test]
    fn unknonw_escape() {
        let str = "\"\\q\"".to_string();
        let mut reader = from_string(&str);

        assert!(matches!(
            reader.next_json_value(),
            Err(JsonParserError::UnexpectedCharacter(_, _, _))
        ));
    }

    #[test]
    fn unknonw_char() {
        let str = "hello".to_string();
        let mut reader = from_string(&str);

        assert!(matches!(
            reader.next_json_value(),
            Err(JsonParserError::UnexpectedCharacter(_, _, _))
        ));
    }
}
