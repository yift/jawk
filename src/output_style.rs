use std::collections::HashMap;
use std::fmt::{Result as FmtResult, Write};

use std::{cell::RefCell, rc::Rc};

use clap::Args;
use indexmap::IndexMap;
use thiserror::Error;

use crate::json_value::NumberValue;
use crate::{
    json_value::JsonValue,
    processor::Result as ProcessResult,
    processor::{Context, Process, ProcessDesision, ProcessError, Titles},
};

#[derive(Args, Debug)]
#[group(required = false, multiple = true)]
pub struct OutputOptions {
    /// How to display the output
    #[arg(long, short, default_value_t = OutputStyle::Json)]
    #[clap(value_enum)]
    output_style: OutputStyle,

    /// Row seperator.
    ///
    /// How to seperate between each row. The default is new line, but one can use something like `--row_seperator="---\n" to use yaml style seperation.
    #[arg(long, short, default_value = "\n")]
    row_seperator: String,

    #[command(flatten)]
    json_options: Option<JsonOutputOptions>,

    #[command(flatten)]
    text_options: Option<TextOutputOptions>,
}

#[derive(clap::ValueEnum, Debug, Clone, PartialEq, Copy)]
#[clap(rename_all = "kebab_case")]
pub enum OutputStyle {
    /// pretty JSON output.
    Json,
    /// CSV file format. This must have selection and can not be a produce of group by as we need to know the columns.
    Csv,
    /// Raw text output.
    Text,
}

#[derive(Args, Debug, Clone)]
#[group(required = false, multiple = true)]
pub struct JsonOutputOptions {
    /// Json output style.
    #[arg(long, default_value_t = JsonStyle::OneLine)]
    #[clap(value_enum)]
    style: JsonStyle,

    /// Output string literal as UTF8 for JSON, (by default only ASCII will be used, and everything else will be escaped).
    #[arg(long, default_value_t = false)]
    utf8_strings: bool,
}

#[derive(clap::ValueEnum, Debug, Clone, PartialEq)]
#[clap(rename_all = "kebab_case")]
pub enum JsonStyle {
    /// One line JSON
    OneLine,
    /// Consise JSON output (no unneeded white spaces).
    Consise,
    /// Pretty JSON output.
    Pretty,
}

#[derive(Args, Debug, Clone)]
#[group(required = false, multiple = true)]
pub struct TextOutputOptions {
    /// Seperate items by (for text output).
    #[arg(long, default_value = "\t")]
    items_seperator: String,

    /// What to add before a String value (for text output).
    #[arg(long, default_value = "")]
    string_prefix: String,

    /// What to add after a String value (for text output).
    #[arg(long, default_value = "")]
    string_postfix: String,

    /// Include headers (for text output).
    #[arg(long, default_value_t = false)]
    headers: bool,

    /// Escape sequance. Can be set more than once. should be used with the escaped character followed by
    /// the excpaed sequance. For example, `"\"` will set the `"`` to be escaped by a `\"` (for text output).
    #[arg(long)]
    escape_sequance: Vec<String>,

    /// How to display nulls values (for text output).
    #[arg(long, default_value = "null")]
    null_keyword: String,

    /// How to display true values (for text output).
    #[arg(long, default_value = "true")]
    true_keyword: String,

    /// How to display false values (for text output).
    #[arg(long, default_value = "false")]
    false_keyword: String,

    /// How to display missing values (for text output).
    #[arg(long)]
    missing_value_keyword: Option<String>,
}

#[derive(Debug, Error)]
pub enum OutputStyleValidationError {
    #[error("Can not define JSON option for non JSON output style")]
    JsonOptionsShouldNotBeHere,
    #[error("Can not define text option for non text output style")]
    TextOptionsShouldNotBeHere,
}

pub trait Print<W: Write> {
    fn print(&self, f: &mut W, value: &Option<JsonValue>) -> FmtResult {
        match value {
            None => self.print_nothing(f),
            Some(value) => self.print_something(f, value),
        }
    }
    fn print_nothing(&self, f: &mut W) -> FmtResult;
    fn print_something(&self, f: &mut W, value: &JsonValue) -> FmtResult {
        match value {
            JsonValue::Null => self.print_null(f),
            JsonValue::Boolean(true) => self.print_true(f),
            JsonValue::Boolean(false) => self.print_false(f),
            JsonValue::Number(value) => self.print_number(f, value),
            JsonValue::String(value) => self.print_string(f, value),
            JsonValue::Array(value) => self.print_array(f, value),
            JsonValue::Object(value) => self.print_object(f, value),
        }
    }
    fn print_number(&self, f: &mut W, value: &NumberValue) -> FmtResult {
        match value {
            NumberValue::Float(value) => self.print_f64(f, *value),
            NumberValue::Negative(value) => self.print_i64(f, *value),
            NumberValue::Positive(value) => self.print_u64(f, *value),
        }
    }
    fn print_null(&self, f: &mut W) -> FmtResult;
    fn print_true(&self, f: &mut W) -> FmtResult;
    fn print_false(&self, f: &mut W) -> FmtResult;
    fn print_string(&self, f: &mut W, value: &str) -> FmtResult;
    fn print_f64(&self, f: &mut W, value: f64) -> FmtResult;
    fn print_i64(&self, f: &mut W, value: i64) -> FmtResult;
    fn print_u64(&self, f: &mut W, value: u64) -> FmtResult;
    fn print_array(&self, f: &mut W, value: &[JsonValue]) -> FmtResult;
    fn print_object(&self, f: &mut W, value: &IndexMap<String, JsonValue>) -> FmtResult;
}

impl OutputOptions {
    pub fn get_processor(
        &self,
        writer: Rc<RefCell<dyn std::io::Write + Send>>,
    ) -> Result<Box<dyn Process>, OutputStyleValidationError> {
        let processor: Box<dyn Process> = match self.output_style {
            OutputStyle::Csv => {
                if self.json_options.is_some() {
                    return Err(OutputStyleValidationError::JsonOptionsShouldNotBeHere);
                }
                if self.text_options.is_some() {
                    return Err(OutputStyleValidationError::TextOptionsShouldNotBeHere);
                }
                let options = TextOutputOptions::csv();
                Box::new(TextProcess::new(
                    writer,
                    self.row_seperator.clone(),
                    options,
                ))
            }
            OutputStyle::Text => {
                if self.json_options.is_some() {
                    return Err(OutputStyleValidationError::JsonOptionsShouldNotBeHere);
                }
                let options = self.text_options.as_ref().cloned().unwrap_or_default();
                Box::new(TextProcess::new(
                    writer,
                    self.row_seperator.clone(),
                    options,
                ))
            }
            OutputStyle::Json => {
                if self.text_options.is_some() {
                    return Err(OutputStyleValidationError::TextOptionsShouldNotBeHere);
                }
                let options = self.json_options.as_ref().cloned().unwrap_or_default();
                Box::new(JsonProcess {
                    line_seperator: self.row_seperator.clone(),
                    printer: options,
                    writer,
                })
            }
        };

        Ok(processor)
    }
}

struct TextPrinter {
    options: TextOutputOptions,
    escape_sequandes: HashMap<char, String>,
}
impl From<TextOutputOptions> for TextPrinter {
    fn from(options: TextOutputOptions) -> Self {
        let mut escape_sequandes = HashMap::with_capacity(options.escape_sequance.capacity());
        for v in &options.escape_sequance {
            if let Some(c) = v.chars().next() {
                escape_sequandes.insert(c, v[1..].to_string());
            }
        }
        TextPrinter {
            options,
            escape_sequandes,
        }
    }
}
struct TextProcess {
    writer: Rc<RefCell<dyn std::io::Write + Send>>,
    length: usize,
    line_seperator: String,
    printer: TextPrinter,
}
impl Default for TextOutputOptions {
    fn default() -> Self {
        Self {
            items_seperator: "\t".to_string(),
            string_prefix: String::new(),
            string_postfix: String::new(),
            headers: false,
            escape_sequance: vec![],
            null_keyword: "null".to_string(),
            true_keyword: "true".to_string(),
            false_keyword: "false".to_string(),
            missing_value_keyword: None,
        }
    }
}
impl TextOutputOptions {
    fn csv() -> Self {
        Self {
            items_seperator: ", ".to_string(),
            string_prefix: "\"".to_string(),
            string_postfix: "\"".to_string(),
            headers: true,
            escape_sequance: vec!["\"\"\"".to_string()],
            null_keyword: "null".to_string(),
            true_keyword: "True".to_string(),
            false_keyword: "False".to_string(),
            missing_value_keyword: None,
        }
    }
}
impl<W: Write> Print<W> for TextPrinter {
    fn print_nothing(&self, f: &mut W) -> FmtResult {
        if let Some(missing_value_keyword) = &self.options.missing_value_keyword {
            write!(f, "{missing_value_keyword}")
        } else {
            Ok(())
        }
    }
    fn print_null(&self, f: &mut W) -> FmtResult {
        write!(f, "{}", self.options.null_keyword)
    }
    fn print_true(&self, f: &mut W) -> FmtResult {
        write!(f, "{}", self.options.true_keyword)
    }
    fn print_false(&self, f: &mut W) -> FmtResult {
        write!(f, "{}", self.options.false_keyword)
    }
    fn print_f64(&self, f: &mut W, value: f64) -> FmtResult {
        write!(f, "{value}")
    }
    fn print_u64(&self, f: &mut W, value: u64) -> FmtResult {
        write!(f, "{value}")
    }
    fn print_i64(&self, f: &mut W, value: i64) -> FmtResult {
        write!(f, "{value}")
    }
    fn print_string(&self, f: &mut W, value: &str) -> FmtResult {
        write!(f, "{}", self.options.string_prefix)?;
        for ch in value.chars() {
            if let Some(str) = self.escape_sequandes.get(&ch) {
                write!(f, "{str}")?;
            } else {
                write!(f, "{ch}")?;
            }
        }
        write!(f, "{}", self.options.string_postfix)
    }
    fn print_object(&self, f: &mut W, value: &IndexMap<String, JsonValue>) -> FmtResult {
        let json = JsonOutputOptions {
            style: JsonStyle::Consise,
            utf8_strings: true,
        };
        let mut str = String::new();
        json.print_object(&mut str, value)?;
        self.print_string(f, &str)
    }
    fn print_array(&self, f: &mut W, value: &[JsonValue]) -> FmtResult {
        let json = JsonOutputOptions {
            style: JsonStyle::Consise,
            utf8_strings: true,
        };
        let mut str = String::new();
        json.print_array(&mut str, value)?;
        self.print_string(f, &str)
    }
}

impl<W: Write> Print<W> for JsonOutputOptions {
    fn print_nothing(&self, _: &mut W) -> FmtResult {
        Ok(())
    }
    fn print_null(&self, f: &mut W) -> FmtResult {
        write!(f, "null")
    }
    fn print_true(&self, f: &mut W) -> FmtResult {
        write!(f, "true")
    }
    fn print_false(&self, f: &mut W) -> FmtResult {
        write!(f, "false")
    }
    fn print_f64(&self, f: &mut W, value: f64) -> FmtResult {
        write!(f, "{value}")
    }
    fn print_u64(&self, f: &mut W, value: u64) -> FmtResult {
        write!(f, "{value}")
    }
    fn print_i64(&self, f: &mut W, value: i64) -> FmtResult {
        write!(f, "{value}")
    }
    fn print_string(&self, f: &mut W, value: &str) -> FmtResult {
        write!(f, "\"")?;
        for ch in value.chars() {
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
                    if self.utf8_strings || (' '..='~').contains(&ch) {
                        write!(f, "{ch}")?;
                    } else {
                        write!(f, "\\u{:04x}", ch as u64)?;
                    }
                }
            }
        }
        write!(f, "\"")
    }
    fn print_object(&self, f: &mut W, value: &IndexMap<String, JsonValue>) -> FmtResult {
        self.print_object_with_indent(f, value, 0)
    }
    fn print_array(&self, f: &mut W, value: &[JsonValue]) -> FmtResult {
        self.print_array_with_indent(f, value, 0)
    }
}
impl Default for JsonOutputOptions {
    fn default() -> Self {
        Self {
            style: JsonStyle::OneLine,
            utf8_strings: false,
        }
    }
}
impl JsonOutputOptions {
    fn insert_indent<W: Write>(&self, f: &mut W, indent: usize) -> FmtResult {
        match self.style {
            JsonStyle::Pretty => {
                writeln!(f)?;
                for _ in 1..=indent {
                    write!(f, "  ")?;
                }
                Ok(())
            }
            _ => Ok(()),
        }
    }
    fn insert_comma<W: Write>(&self, f: &mut W) -> FmtResult {
        match self.style {
            JsonStyle::OneLine => write!(f, ", "),
            _ => write!(f, ","),
        }
    }
    fn print_object_with_indent<W: Write>(
        &self,
        f: &mut W,
        value: &IndexMap<String, JsonValue>,
        indent: usize,
    ) -> FmtResult {
        if value.is_empty() {
            return write!(f, "{{}}");
        }
        write!(f, "{{")?;
        let size = value.len();
        for (index, element) in value.iter().enumerate() {
            let (key, value) = element;
            self.insert_indent(f, indent + 1)?;
            self.print_string(f, key)?;
            write!(f, ":")?;
            match self.style {
                JsonStyle::Consise => {}
                _ => write!(f, " ")?,
            }
            match value {
                JsonValue::Array(value) => self.print_array_with_indent(f, value, indent + 1)?,
                JsonValue::Object(value) => self.print_object_with_indent(f, value, indent + 1)?,
                _ => self.print_something(f, value)?,
            }
            if index != size - 1 {
                self.insert_comma(f)?;
            }
        }
        self.insert_indent(f, indent)?;
        write!(f, "}}")
    }
    fn print_array_with_indent<W: Write>(
        &self,
        f: &mut W,
        value: &[JsonValue],
        indent: usize,
    ) -> FmtResult {
        if value.is_empty() {
            return write!(f, "[]");
        }
        write!(f, "[")?;
        let size = value.len();
        for (index, value) in value.iter().enumerate() {
            self.insert_indent(f, indent + 1)?;
            match value {
                JsonValue::Array(value) => self.print_array_with_indent(f, value, indent + 1)?,
                JsonValue::Object(value) => self.print_object_with_indent(f, value, indent + 1)?,
                _ => self.print_something(f, value)?,
            }
            if index != size - 1 {
                self.insert_comma(f)?;
            }
        }
        self.insert_indent(f, indent)?;
        write!(f, "]")
    }
}
impl TextProcess {
    fn new(
        writer: Rc<RefCell<dyn std::io::Write + Send>>,
        line_seperator: String,
        options: TextOutputOptions,
    ) -> Self {
        Self {
            writer,
            length: 0,
            line_seperator,
            printer: options.into(),
        }
    }
    fn print_list(&mut self, list: &[Option<JsonValue>]) -> ProcessResult<ProcessDesision> {
        for (index, value) in list.iter().enumerate() {
            let mut str = String::new();
            self.printer.print(&mut str, value)?;
            write!(self.writer.borrow_mut(), "{str}")?;
            if index < self.length - 1 {
                write!(
                    self.writer.borrow_mut(),
                    "{}",
                    self.printer.options.items_seperator
                )?;
            }
        }
        write!(self.writer.borrow_mut(), "{}", self.line_seperator)?;
        Ok(ProcessDesision::Continue)
    }
}

impl Process for TextProcess {
    fn complete(&mut self) -> ProcessResult<()> {
        Ok(())
    }
    fn start(&mut self, titles_so_far: Titles) -> ProcessResult<()> {
        self.length = titles_so_far.len();
        if self.printer.options.headers {
            if self.length > 0 {
                self.print_list(&titles_so_far.to_list())?;
            } else {
                return Err(ProcessError::InvalidInputError(
                    "Missing headers. This output style must have selection and can no group by",
                ));
            }
        }

        Ok(())
    }
    fn process(&mut self, context: Context) -> ProcessResult<ProcessDesision> {
        if self.length != 0 {
            self.print_list(&context.to_list())
        } else {
            let mut str = String::new();
            self.printer.print_something(&mut str, context.input())?;
            write!(self.writer.borrow_mut(), "{}{}", str, self.line_seperator)?;
            Ok(ProcessDesision::Continue)
        }
    }
}

struct JsonProcess {
    line_seperator: String,
    printer: JsonOutputOptions,
    writer: Rc<RefCell<dyn std::io::Write + Send>>,
}

impl Process for JsonProcess {
    fn start(&mut self, _: Titles) -> ProcessResult<()> {
        Ok(())
    }
    fn complete(&mut self) -> ProcessResult<()> {
        Ok(())
    }
    fn process(&mut self, context: Context) -> ProcessResult<ProcessDesision> {
        let value = context.build();
        let mut str = String::new();
        self.printer.print_something(&mut str, &value)?;
        write!(self.writer.borrow_mut(), "{}{}", str, self.line_seperator)?;
        Ok(ProcessDesision::Continue)
    }
}

#[cfg(test)]
mod tests {
    use std::str::FromStr;

    use super::*;

    #[test]
    fn get_processor_will_fail_when_csv_has_json_options() {
        let options = OutputOptions {
            output_style: OutputStyle::Csv,
            row_seperator: String::new(),
            json_options: Some(JsonOutputOptions::default()),
            text_options: None,
        };
        let text = Vec::new();
        let writer = Rc::new(RefCell::new(text));

        let error = options.get_processor(writer);

        assert!(error.is_err());
    }

    #[test]
    fn get_processor_will_fail_when_csv_has_text_options() {
        let options = OutputOptions {
            output_style: OutputStyle::Csv,
            row_seperator: String::new(),
            json_options: None,
            text_options: Some(TextOutputOptions::default()),
        };
        let text = Vec::new();
        let writer = Rc::new(RefCell::new(text));

        let error = options.get_processor(writer);

        assert!(error.is_err());
    }

    #[test]
    fn get_processor_will_pass_when_csv_has_no_options() {
        let options = OutputOptions {
            output_style: OutputStyle::Csv,
            row_seperator: String::new(),
            json_options: None,
            text_options: None,
        };
        let text = Vec::new();
        let writer = Rc::new(RefCell::new(text));

        let result = options.get_processor(writer);

        assert!(result.is_ok());
    }

    #[test]
    fn get_processor_will_fail_when_text_has_json_options() {
        let options = OutputOptions {
            output_style: OutputStyle::Text,
            row_seperator: String::new(),
            json_options: Some(JsonOutputOptions::default()),
            text_options: None,
        };
        let text = Vec::new();
        let writer = Rc::new(RefCell::new(text));

        let error = options.get_processor(writer);

        assert!(error.is_err());
    }

    #[test]
    fn get_processor_will_pass_when_text_has_no_options() {
        let options = OutputOptions {
            output_style: OutputStyle::Text,
            row_seperator: String::new(),
            json_options: None,
            text_options: None,
        };
        let text = Vec::new();
        let writer = Rc::new(RefCell::new(text));

        let result = options.get_processor(writer);

        assert!(result.is_ok());
    }

    #[test]
    fn get_processor_will_pass_when_text_has_text_options() {
        let options = OutputOptions {
            output_style: OutputStyle::Text,
            row_seperator: String::new(),
            json_options: None,
            text_options: Some(TextOutputOptions::default()),
        };
        let text = Vec::new();
        let writer = Rc::new(RefCell::new(text));

        let result = options.get_processor(writer);

        assert!(result.is_ok());
    }

    #[test]
    fn get_processor_will_fail_when_json_has_text_options() {
        let options = OutputOptions {
            output_style: OutputStyle::Json,
            row_seperator: String::new(),
            json_options: None,
            text_options: Some(TextOutputOptions::csv()),
        };
        let text = Vec::new();
        let writer = Rc::new(RefCell::new(text));

        let error = options.get_processor(writer);

        assert!(error.is_err());
    }

    #[test]
    fn get_processor_will_pass_when_json_has_no_options() {
        let options = OutputOptions {
            output_style: OutputStyle::Json,
            row_seperator: String::new(),
            json_options: None,
            text_options: None,
        };
        let text = Vec::new();
        let writer = Rc::new(RefCell::new(text));

        let result = options.get_processor(writer);

        assert!(result.is_ok());
    }

    #[test]
    fn get_processor_will_pass_when_json_has_json_options() {
        let options = OutputOptions {
            output_style: OutputStyle::Json,
            row_seperator: String::new(),
            json_options: Some(JsonOutputOptions::default()),
            text_options: None,
        };
        let text = Vec::new();
        let writer = Rc::new(RefCell::new(text));

        let result = options.get_processor(writer);

        assert!(result.is_ok());
    }

    #[test]
    fn text_printer_will_not_print_missing_values_by_default() {
        let options = TextOutputOptions {
            missing_value_keyword: None,
            ..TextOutputOptions::default()
        };
        let mut text = String::new();

        let printer = TextPrinter::from(options);
        printer.print(&mut text, &None).unwrap();

        assert_eq!(text.as_str(), "");
    }

    #[test]
    fn text_printer_will_print_missing_values() {
        let options = TextOutputOptions {
            missing_value_keyword: Some("val".into()),
            ..TextOutputOptions::default()
        };
        let mut text = String::new();

        let printer = TextPrinter::from(options);
        printer.print(&mut text, &None).unwrap();

        assert_eq!(text.as_str(), "val");
    }

    #[test]
    fn text_printer_will_print_null() {
        let options = TextOutputOptions {
            null_keyword: "NULL".into(),
            ..TextOutputOptions::default()
        };
        let mut text = String::new();

        let printer = TextPrinter::from(options);
        printer.print(&mut text, &Some(JsonValue::Null)).unwrap();

        assert_eq!(text.as_str(), "NULL");
    }

    #[test]
    fn text_printer_will_print_true() {
        let options = TextOutputOptions {
            true_keyword: "YES".into(),
            ..TextOutputOptions::default()
        };
        let mut text = String::new();

        let printer = TextPrinter::from(options);
        printer
            .print(&mut text, &Some(JsonValue::Boolean(true)))
            .unwrap();

        assert_eq!(text.as_str(), "YES");
    }

    #[test]
    fn text_printer_will_print_false() {
        let options = TextOutputOptions {
            false_keyword: "NO".into(),
            ..TextOutputOptions::default()
        };
        let mut text = String::new();

        let printer = TextPrinter::from(options);
        printer
            .print(&mut text, &Some(JsonValue::Boolean(false)))
            .unwrap();

        assert_eq!(text.as_str(), "NO");
    }

    #[test]
    fn text_printer_will_print_float() {
        let options = TextOutputOptions::default();
        let mut text = String::new();

        let printer = TextPrinter::from(options);
        printer
            .print(&mut text, &Some(JsonValue::Number(NumberValue::Float(1.5))))
            .unwrap();

        assert_eq!(text.as_str(), "1.5");
    }

    #[test]
    fn text_printer_will_print_int() {
        let options = TextOutputOptions::default();
        let mut text = String::new();

        let printer = TextPrinter::from(options);
        printer
            .print(
                &mut text,
                &Some(JsonValue::Number(NumberValue::Negative(-10))),
            )
            .unwrap();

        assert_eq!(text.as_str(), "-10");
    }

    #[test]
    fn text_printer_will_print_uint() {
        let options = TextOutputOptions::default();
        let mut text = String::new();

        let printer = TextPrinter::from(options);
        printer
            .print(
                &mut text,
                &Some(JsonValue::Number(NumberValue::Positive(12))),
            )
            .unwrap();

        assert_eq!(text.as_str(), "12");
    }

    #[test]
    fn text_printer_will_print_string() {
        let options = TextOutputOptions {
            string_prefix: "<".into(),
            string_postfix: ">".into(),
            escape_sequance: vec!["<&lt;".into(), ">&gt;".into(), String::new(), "T".into()],
            ..TextOutputOptions::default()
        };
        let mut text = String::new();

        let printer = TextPrinter::from(options);
        printer
            .print(&mut text, &Some(JsonValue::String("test<>this".into())))
            .unwrap();

        assert_eq!(text.as_str(), "<test&lt;&gt;this>");
    }

    #[test]
    fn text_printer_will_print_object() {
        let options = TextOutputOptions::default();
        let mut text = String::new();

        let printer = TextPrinter::from(options);
        printer
            .print(&mut text, &JsonValue::from_str("{\"key\": 100}").ok())
            .unwrap();

        assert_eq!(text.as_str(), "{\"key\":100}");
    }

    #[test]
    fn text_printer_will_print_array() {
        let options = TextOutputOptions::default();
        let mut text = String::new();

        let printer = TextPrinter::from(options);
        printer
            .print(&mut text, &JsonValue::from_str("[1, 2, 3]").ok())
            .unwrap();

        assert_eq!(text.as_str(), "[1,2,3]");
    }

    #[test]
    fn json_printer_will_not_print_missing_values() {
        let printer = JsonOutputOptions::default();
        let mut text = String::new();

        printer.print(&mut text, &None).unwrap();

        assert_eq!(text.as_str(), "");
    }

    #[test]
    fn json_printer_will_print_null_value() {
        let printer = JsonOutputOptions::default();
        let mut text = String::new();

        printer
            .print(&mut text, &JsonValue::from_str("null").ok())
            .unwrap();

        assert_eq!(text.as_str(), "null");
    }
    #[test]
    fn json_printer_will_print_true_value() {
        let printer = JsonOutputOptions::default();
        let mut text = String::new();

        printer
            .print(&mut text, &JsonValue::from_str("true").ok())
            .unwrap();

        assert_eq!(text.as_str(), "true");
    }
    #[test]
    fn json_printer_will_print_false_value() {
        let printer = JsonOutputOptions::default();
        let mut text = String::new();

        printer
            .print(&mut text, &JsonValue::from_str("false").ok())
            .unwrap();

        assert_eq!(text.as_str(), "false");
    }

    #[test]
    fn json_printer_will_print_float_value() {
        let printer = JsonOutputOptions::default();
        let mut text = String::new();

        printer
            .print(&mut text, &JsonValue::from_str("1.5").ok())
            .unwrap();

        assert_eq!(text.as_str(), "1.5");
    }
    #[test]
    fn json_printer_will_print_uint_value() {
        let printer = JsonOutputOptions::default();
        let mut text = String::new();

        printer
            .print(&mut text, &JsonValue::from_str("15").ok())
            .unwrap();

        assert_eq!(text.as_str(), "15");
    }
    #[test]
    fn json_printer_will_print_int_value() {
        let printer = JsonOutputOptions::default();
        let mut text = String::new();

        printer
            .print(&mut text, &JsonValue::from_str("-15").ok())
            .unwrap();

        assert_eq!(text.as_str(), "-15");
    }

    #[test]
    fn json_printer_will_print_string_value_ascii() {
        let printer = JsonOutputOptions {
            utf8_strings: false,
            style: JsonStyle::OneLine,
        };
        let mut text = String::new();

        printer
            .print(
                &mut text,
                &Some(JsonValue::String(
                    "test\n\"\\/\u{08}\u{0c}\r\t\u{1f603}".into(),
                )),
            )
            .unwrap();

        assert_eq!(text.as_str(), "\"test\\n\\\"\\\\\\/\\b\\f\\r\\t\\u1f603\"");
    }

    #[test]
    fn json_printer_will_print_string_value_utf() {
        let printer = JsonOutputOptions {
            utf8_strings: true,
            style: JsonStyle::OneLine,
        };
        let mut text = String::new();

        printer
            .print(
                &mut text,
                &Some(JsonValue::String(
                    "test\n\"\\/\u{08}\u{0c}\r\t\u{1f603}".into(),
                )),
            )
            .unwrap();

        assert_eq!(text.as_str(), "\"test\\n\\\"\\\\\\/\\b\\f\\r\\t\u{1f603}\"");
    }

    #[test]
    fn json_printer_will_print_consise_object() {
        let printer = JsonOutputOptions {
            utf8_strings: true,
            style: JsonStyle::Consise,
        };
        let mut text = String::new();

        printer
            .print(
                &mut text,
                &JsonValue::from_str("{\"a\": [1, 2, 3, 4, 5, []]}").ok(),
            )
            .unwrap();

        assert_eq!(text.as_str(), "{\"a\":[1,2,3,4,5,[]]}");
    }

    #[test]
    fn json_printer_will_print_one_line_object() {
        let printer = JsonOutputOptions {
            utf8_strings: true,
            style: JsonStyle::OneLine,
        };
        let mut text = String::new();

        printer
            .print(
                &mut text,
                &JsonValue::from_str("{\"a\": [1, 2, 3, 4, 5, []]}").ok(),
            )
            .unwrap();

        assert_eq!(text.as_str(), "{\"a\": [1, 2, 3, 4, 5, []]}");
    }

    #[test]
    fn json_printer_will_print_pretty_object() {
        let printer = JsonOutputOptions {
            utf8_strings: true,
            style: JsonStyle::Pretty,
        };
        let mut text = String::new();

        printer
            .print(
                &mut text,
                &JsonValue::from_str("{\"a\": [1, 2, 3, 4, 5, []]}").ok(),
            )
            .unwrap();

        assert_eq!(
            text.as_str(),
            "{\n  \"a\": [\n    1,\n    2,\n    3,\n    4,\n    5,\n    []\n  ]\n}"
        );
    }

    #[test]
    fn json_printer_will_print_consise_array() {
        let printer = JsonOutputOptions {
            utf8_strings: true,
            style: JsonStyle::Consise,
        };
        let mut text = String::new();

        printer
            .print(
                &mut text,
                &JsonValue::from_str("[1, 2, 3, 4, 5, {\"a\": 12}, {}]").ok(),
            )
            .unwrap();

        assert_eq!(text.as_str(), "[1,2,3,4,5,{\"a\":12},{}]");
    }

    #[test]
    fn json_printer_will_print_one_line_array() {
        let printer = JsonOutputOptions {
            utf8_strings: true,
            style: JsonStyle::OneLine,
        };
        let mut text = String::new();

        printer
            .print(
                &mut text,
                &JsonValue::from_str("[1, 2, 3, 4, 5, {\"a\": 12}, {}]").ok(),
            )
            .unwrap();

        assert_eq!(text.as_str(), "[1, 2, 3, 4, 5, {\"a\": 12}, {}]");
    }

    #[test]
    fn json_printer_will_print_pretty_array() {
        let printer = JsonOutputOptions {
            utf8_strings: true,
            style: JsonStyle::Pretty,
        };
        let mut text = String::new();

        printer
            .print(
                &mut text,
                &JsonValue::from_str("[1, 2, 3, 4, 5, {\"a\": 12}, {}]").ok(),
            )
            .unwrap();

        assert_eq!(
            text.as_str(),
            "[\n  1,\n  2,\n  3,\n  4,\n  5,\n  {\n    \"a\": 12\n  },\n  {}\n]"
        );
    }
}
