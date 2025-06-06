use std::{
    fmt::Display,
    fs::File,
    io::{BufReader, Bytes, Read, Result},
    path::PathBuf,
};

#[derive(Debug, Clone, PartialEq)]
pub struct Location {
    pub input: Option<String>,
    pub line_number: usize,
    pub char_number: usize,
}

pub struct Reader<R: Read> {
    bytes: Bytes<BufReader<R>>,
    current_byte: Option<u8>,
    location: Location,
    eof: bool,
}

pub fn from_file(file_name: &PathBuf) -> Result<Reader<File>> {
    let file = File::open(file_name)?;
    Ok(Reader::new(
        file,
        file_name.to_str().map(ToString::to_string),
    ))
}

pub fn from_std_in<R: Read>(stdin: R) -> Reader<R> {
    Reader::new(stdin, None)
}

pub fn from_string(source: &String) -> Reader<&[u8]> {
    let reader = source.as_bytes();
    let mut name = source.clone();
    name.truncate(32);
    Reader::new(reader, Some(name))
}

impl<R: Read> Reader<R> {
    fn new(reader: R, name: Option<String>) -> Self {
        let location = Location {
            input: name,
            line_number: 1,
            char_number: 1,
        };
        Reader {
            bytes: BufReader::new(reader).bytes(),
            current_byte: Option::None,
            location,
            eof: false,
        }
    }

    #[inline]
    pub fn next(&mut self) -> Result<Option<u8>> {
        if self.eof {
            return Ok(None);
        }
        match self.bytes.next() {
            None => {
                self.eof = true;
                self.current_byte = None;
                Ok(None)
            }
            Some(ch) => {
                let ch = ch?;
                if ch == b'\n' {
                    self.location.line_number += 1;
                    self.location.char_number = 1;
                } else {
                    self.location.char_number += 1;
                }
                self.current_byte = Some(ch);
                Ok(self.current_byte)
            }
        }
    }

    #[inline]
    pub fn peek(&mut self) -> Result<Option<u8>> {
        match self.current_byte {
            Some(ch) => Ok(Some(ch)),
            None => self.next(),
        }
    }

    #[inline]
    pub fn eat_whitespace(&mut self) -> Result<()> {
        loop {
            match self.peek()? {
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
    pub fn read_digits(&mut self, digits: &mut Vec<u8>) -> Result<()> {
        loop {
            let letter = self.peek()?;
            match letter {
                Some(b'0'..=b'9') => {
                    digits.push(letter.unwrap());
                    self.next()?;
                }
                _ => {
                    return Ok(());
                }
            }
        }
    }

    pub fn where_am_i(&self) -> Location {
        self.location.clone()
    }
}

impl Display for Location {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match &self.input {
            Some(name) => write!(f, "{}:{}:{}", name, self.line_number, self.char_number),
            None => write!(f, "{}:{}", self.line_number, self.char_number),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_from_string() -> Result<()> {
        let str = "abc".to_string();
        let mut reader = from_string(&str);

        assert_eq!(reader.next()?, Some(b'a'));

        Ok(())
    }

    #[test]
    fn test_next() -> Result<()> {
        let str = "abc".to_string();
        let mut reader = from_string(&str);
        reader.next()?;
        reader.next()?;

        assert_eq!(reader.next()?, Some(b'c'));

        Ok(())
    }

    #[test]
    fn test_next_eof() -> Result<()> {
        let str = "ab".to_string();
        let mut reader = from_string(&str);
        reader.next()?;
        reader.next()?;

        assert_eq!(reader.next()?, None);

        Ok(())
    }

    #[test]
    fn test_next_after_eof() -> Result<()> {
        let str = "ab".to_string();
        let mut reader = from_string(&str);
        reader.next()?;
        reader.next()?;
        reader.next()?;
        reader.next()?;

        assert_eq!(reader.next()?, None);

        Ok(())
    }

    #[test]
    fn test_peak_starts() -> Result<()> {
        let str = "abc".to_string();
        let mut reader = from_string(&str);

        assert_eq!(reader.peek()?, Some(b'a'));

        Ok(())
    }

    #[test]
    fn test_peak_middle() -> Result<()> {
        let str = "abc".to_string();
        let mut reader = from_string(&str);
        reader.next()?;
        reader.next()?;

        assert_eq!(reader.peek()?, Some(b'b'));

        Ok(())
    }

    #[test]
    fn test_eat_whitespaces() -> Result<()> {
        let str = "a     \t\nbc".to_string();
        let mut reader = from_string(&str);
        reader.next()?;
        reader.next()?;
        reader.eat_whitespace()?;

        assert_eq!(reader.peek()?, Some(b'b'));

        Ok(())
    }

    #[test]
    fn test_location_is_correct() -> Result<()> {
        let str = "a\nb\ncde".to_string();
        let mut reader = from_string(&str);
        reader.next()?;
        reader.next()?;
        reader.next()?;
        reader.next()?;

        assert_eq!(
            reader.where_am_i(),
            Location {
                input: Some("a\nb\ncde".to_string()),
                line_number: 3,
                char_number: 1,
            }
        );

        Ok(())
    }
}
