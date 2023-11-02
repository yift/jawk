use std::{collections::HashMap, io::Bytes, io::Read, io::Result};

pub struct JsonReader<R: Read> {
    bytes: Bytes<R>,
    current_byte: Option<u8>,
}

impl<R: Read> JsonReader<R> {
    pub fn new(reader: R) -> Self {
        JsonReader {
            bytes: reader.bytes(),
            current_byte: Option::None,
        }
    }

    #[inline]
    fn next(&mut self) -> Result<Option<u8>> {
        match self.bytes.next() {
            None => {
                self.current_byte = None;
                Ok(None)
            }
            Some(ch) => {
                let ch = ch?;
                self.current_byte = Some(ch);
                Ok(Some(ch))
            }
        }
    }

    #[inline]
    fn peek(&mut self) -> Result<Option<u8>> {
        match self.current_byte {
            Some(ch) => Ok(Some(ch)),
            None => self.next(),
        }
    }

    #[inline]
    fn eat_whitespace(&mut self) -> Result<()> {
        loop {
            match self.peek()? {
                None => {
                    return Ok(());
                }

                Some(b' ' | b'\n' | b'\t' | b'\r') => {
                    self.next()?;
                }
                _ => {
                    return Ok(());
                }
            }
        }
    }

    #[inline]
    fn read_true(&mut self) -> Result<ReadValue> {
        if self.next()? == Some(b'r') && self.next()? == Some(b'u') && self.next()? == Some(b'e') {
            self.next()?;
            Ok(ReadValue::Value(JsonValue::Boolean(true)))
        } else {
            Ok(ReadValue::Error)
        }
    }

    #[inline]
    fn read_false(&mut self) -> Result<ReadValue> {
        if self.next()? == Some(b'a')
            && self.next()? == Some(b'l')
            && self.next()? == Some(b's')
            && self.next()? == Some(b'e')
        {
            self.next()?;
            Ok(ReadValue::Value(JsonValue::Boolean(false)))
        } else {
            Ok(ReadValue::Error)
        }
    }

    #[inline]
    fn read_null(&mut self) -> Result<ReadValue> {
        if self.next()? == Some(b'u') && self.next()? == Some(b'l') && self.next()? == Some(b'l') {
            self.next()?;
            Ok(ReadValue::Value(JsonValue::Null))
        } else {
            Ok(ReadValue::Error)
        }
    }

    #[inline]
    fn read_next_value(&mut self) -> Result<ReadValue> {
        self.eat_whitespace()?;

        match self.peek()? {
            None => Ok(ReadValue::Eof),
            Some(b't') => self.read_true(),
            Some(b'f') => self.read_false(),
            Some(b'n') => self.read_null(),
            Some(b'\"') => self.read_string(),
            Some(b'-' | b'0'..=b'9') => self.read_number(),
            Some(b'[') => self.read_array(),
            Some(b'{') => self.read_object(),
            _ => {
                self.next()?;
                Ok(ReadValue::Error)
            }
        }
    }
    #[inline]
    fn read_array(&mut self) -> Result<ReadValue> {
        self.next()?;
        self.eat_whitespace()?;
        if self.peek()? == Some(b']') {
            self.next()?;
            return Ok(ReadValue::Value(JsonValue::Array(vec![])));
        }
        let mut array = Vec::new();
        loop {
            if let ReadValue::Value(value) = self.read_next_value()? {
                array.push(value);
            } else {
                return Ok(ReadValue::Error);
            }
            self.eat_whitespace()?;
            match self.peek()? {
                Some(b']') => {
                    self.next()?;
                    return Ok(ReadValue::Value(JsonValue::Array(array)));
                }
                Some(b',') => {
                    self.next()?;
                }
                _ => {
                    return Ok(ReadValue::Error);
                }
            }
        }
    }
    #[inline]
    fn read_object(&mut self) -> Result<ReadValue> {
        self.next()?;
        self.eat_whitespace()?;
        let mut map = HashMap::new();
        if self.peek()? == Some(b'}') {
            self.next()?;
            return Ok(ReadValue::Value(JsonValue::Object(map)));
        }
        loop {
            if let ReadValue::Value(JsonValue::String(key)) = self.read_next_value()? {
                self.eat_whitespace()?;
                if self.peek()? != Some(b':') {
                    return Ok(ReadValue::Error);
                }
                self.next()?;
                if let ReadValue::Value(value) = self.read_next_value()? {
                    map.insert(key, value);
                } else {
                    return Ok(ReadValue::Error);
                }
            } else {
                return Ok(ReadValue::Error);
            }
            self.eat_whitespace()?;
            match self.peek()? {
                Some(b'}') => {
                    self.next()?;
                    return Ok(ReadValue::Value(JsonValue::Object(map)));
                }
                Some(b',') => {
                    self.next()?;
                }
                _ => {
                    return Ok(ReadValue::Error);
                }
            }
        }
    }

    #[inline]
    fn read_number(&mut self) -> Result<ReadValue> {
        let mut chars = Vec::new();
        let negative = if self.peek()? == Some(b'-') {
            if self.next()?.is_none() {
                return Ok(ReadValue::Error);
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

        match String::from_utf8(chars) {
            Ok(str) => {
                if double {
                    match str.parse::<f64>() {
                        Ok(f) => Ok(ReadValue::Value(JsonValue::Number(NumberValue::Float(f)))),
                        Err(_) => Ok(ReadValue::Error),
                    }
                } else if negative {
                    match str.parse::<i64>() {
                        Ok(i) => Ok(ReadValue::Value(JsonValue::Number(NumberValue::Negative(
                            i,
                        )))),
                        Err(_) => Ok(ReadValue::Error),
                    }
                } else {
                    match str.parse::<u64>() {
                        Ok(u) => Ok(ReadValue::Value(JsonValue::Number(NumberValue::Positive(
                            u,
                        )))),
                        Err(_) => Ok(ReadValue::Error),
                    }
                }
            }
            Err(_) => Ok(ReadValue::Error),
        }
    }

    #[inline]
    fn read_digits(&mut self, digits: &mut Vec<u8>) -> Result<()> {
        loop {
            let letter = self.peek()?;
            match letter {
                Some(b'0'..=b'9') => {
                    digits.push(letter.unwrap());
                    self.next()?;
                }
                _ => return Ok(()),
            }
        }
    }

    #[inline]
    fn read_string(&mut self) -> Result<ReadValue> {
        let mut chars = Vec::new();
        loop {
            match self.next()? {
                None => {
                    return Ok(ReadValue::Error);
                }
                Some(b'\"') => {
                    self.next()?;
                    match String::from_utf8(chars) {
                        Ok(str) => {
                            return Ok(ReadValue::Value(JsonValue::String(str)));
                        }
                        Err(_) => return Ok(ReadValue::Error),
                    }
                }
                Some(b'\\') => match self.next()? {
                    None => return Ok(ReadValue::Error),
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
                                None => return Ok(ReadValue::Error),
                                Some(c) => {
                                    let d = match c {
                                        b'0'..=b'9' => (c - b'0') as u32,
                                        b'a'..=b'f' => (c - b'a' + 10) as u32,
                                        b'A'..=b'F' => (c - b'A' + 10) as u32,
                                        _ => return Ok(ReadValue::Error),
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
                            None => return Ok(ReadValue::Error),
                        }
                    }
                    _ => return Ok(ReadValue::Error),
                },
                Some(c) => {
                    chars.push(c);
                }
            }
        }
    }

    pub fn next_value(&mut self) -> Result<Option<JsonValue>> {
        loop {
            match self.read_next_value()? {
                ReadValue::Eof => {
                    return Ok(None);
                }
                ReadValue::Value(value) => {
                    return Ok(Some(value));
                }
                ReadValue::Error => {
                    // Do nothing
                }
            }
        }
    }
}

#[derive(Debug)]
pub enum JsonValue {
    Null,
    Boolean(bool),
    String(String),
    Number(NumberValue),
    Object(HashMap<String, JsonValue>),
    Array(Vec<JsonValue>),
}

#[derive(Debug)]
pub enum NumberValue {
    Negative(i64),
    Positive(u64),
    Float(f64),
}

enum ReadValue {
    Eof,
    Error,
    Value(JsonValue),
}
