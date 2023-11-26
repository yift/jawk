use std::fmt::{Result, Write};

use indexmap::IndexMap;

use crate::json_value::{JsonValue, NumberValue};

pub trait Print<W: Write> {
    fn print(&self, f: &mut W, value: &JsonValue) -> Result;
}

pub struct JsonPrinter {
    indent: Option<usize>,
    space: bool,
}
impl JsonPrinter {
    pub fn new(pretty: bool, space: bool) -> Self {
        let indent = if pretty { Some(0) } else { None };
        JsonPrinter { indent, space }
    }

    fn print_string<W: Write>(f: &mut W, str: &str) -> Result {
        write!(f, "\"")?;
        for ch in str.chars() {
            match ch {
                '\"' => write!(f, "\\\"")?,
                '\\' => write!(f, "\\\\")?,
                '/' => write!(f, "\\/")?,
                '\u{08}' => write!(f, "\\b")?,
                '\u{0c}' => write!(f, "\\f")?,
                '\n' => write!(f, "\\n")?,
                '\r' => write!(f, "\\r")?,
                '\t' => write!(f, "\\t")?,
                ch => {
                    if (' '..='~').contains(&ch) {
                        write!(f, "{}", ch)?
                    } else {
                        write!(f, "\\u{:#04x}", ch as u64)?
                    }
                }
            }
        }
        write!(f, "\"")
    }

    fn insert_indent<W: Write>(&self, f: &mut W) -> Result {
        match self.indent {
            None => Ok(()),
            Some(indent) => {
                writeln!(f)?;
                for _ in 1..=indent {
                    write!(f, "  ")?;
                }
                Ok(())
            }
        }
    }
    fn insert_comma<W: Write>(&self, f: &mut W) -> Result {
        write!(f, ",")?;
        if self.space {
            write!(f, " ")?;
        }
        Ok(())
    }

    fn print_array<W: Write>(&self, f: &mut W, array: &Vec<JsonValue>) -> Result {
        if array.is_empty() {
            return write!(f, "[]");
        }
        let size = array.len();
        let printer = JsonPrinter {
            indent: self.indent.map(|f| f + 1),
            space: self.space,
        };
        write!(f, "[")?;
        for (index, element) in array.iter().enumerate() {
            printer.insert_indent(f)?;
            printer.print(f, element)?;
            if index != size - 1 {
                self.insert_comma(f)?;
            }
        }
        self.insert_indent(f)?;
        write!(f, "]")
    }

    fn print_object<W: Write>(&self, f: &mut W, object: &IndexMap<String, JsonValue>) -> Result {
        if object.is_empty() {
            return write!(f, "{{}}");
        }
        let size = object.len();
        let printer = JsonPrinter {
            indent: self.indent.map(|f| f + 1),
            space: self.space,
        };
        write!(f, "{{")?;
        for (index, element) in object.iter().enumerate() {
            let (key, value) = element;
            printer.insert_indent(f)?;
            Self::print_string(f, key)?;
            write!(f, ":")?;
            if self.indent.is_some() || self.space {
                write!(f, " ")?;
            }
            printer.print(f, value)?;
            if index != size - 1 {
                self.insert_comma(f)?;
            }
        }
        self.insert_indent(f)?;
        write!(f, "}}")
    }
}

impl<W: Write> Print<W> for JsonPrinter {
    fn print(&self, f: &mut W, value: &JsonValue) -> Result {
        match value {
            JsonValue::Null => write!(f, "null"),
            JsonValue::Boolean(true) => write!(f, "true"),
            JsonValue::Boolean(false) => write!(f, "false"),
            JsonValue::Number(n) => print_number(f, n),
            JsonValue::String(str) => Self::print_string(f, str),
            JsonValue::Array(a) => self.print_array(f, a),
            JsonValue::Object(o) => self.print_object(f, o),
        }
    }
}

fn print_number<W: Write>(f: &mut W, value: &NumberValue) -> Result {
    match value {
        NumberValue::Float(n) => write!(f, "{}", n),
        NumberValue::Negative(n) => write!(f, "{}", n),
        NumberValue::Positive(n) => write!(f, "{}", n),
    }
}

pub struct RawTextPrinter;

impl<W: Write> Print<W> for RawTextPrinter {
    fn print(&self, f: &mut W, value: &JsonValue) -> Result {
        match value {
            JsonValue::Null => write!(f, "null"),
            JsonValue::Boolean(true) => write!(f, "true"),
            JsonValue::Boolean(false) => write!(f, "false"),
            JsonValue::Number(n) => print_number(f, n),
            JsonValue::String(str) => write!(f, "{}", str),
            JsonValue::Array(_) => {
                let json = JsonPrinter::new(false, false);
                json.print(f, value)
            }
            JsonValue::Object(_) => {
                let json = JsonPrinter::new(false, false);
                json.print(f, value)
            }
        }
    }
}

pub struct CsvPrinter;

impl<W: Write> Print<W> for CsvPrinter {
    fn print(&self, f: &mut W, value: &JsonValue) -> Result {
        match value {
            JsonValue::Null => write!(f, "null"),
            JsonValue::Boolean(true) => write!(f, "True"),
            JsonValue::Boolean(false) => write!(f, "False"),
            JsonValue::Number(n) => print_number(f, n),
            JsonValue::String(str) => Self::print_string(f, str),
            JsonValue::Array(_) => {
                let json = JsonPrinter::new(false, false);
                let mut str = String::new();
                json.print(&mut str, value)?;
                Self::print_string(f, &str)
            }

            JsonValue::Object(_) => {
                let json = JsonPrinter::new(false, false);
                let mut str = String::new();
                json.print(&mut str, value)?;
                Self::print_string(f, &str)
            }
        }
    }
}

impl CsvPrinter {
    fn print_string<W: Write>(f: &mut W, str: &str) -> Result {
        write!(f, "\"")?;
        for ch in str.chars() {
            if ch == '\"' {
                write!(f, "\"\"")?
            } else {
                write!(f, "{}", ch)?
            };
        }
        write!(f, "\"")
    }
}
