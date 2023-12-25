use std::{
    ops::Deref,
    sync::{Arc, Mutex},
};

use crate::{
    printer::{CsvPrinter, JsonPrinter, Print, RawTextPrinter},
    processor::Result as ProcessResult,
    processor::{Context, Process, ProcessError, Titles},
};

#[derive(clap::ValueEnum, Debug, Clone, PartialEq, Copy)]
#[clap(rename_all = "kebab_case")]
pub enum OutputStyle {
    Json,
    OneLineJson,
    ConsiseJson,
    Csv,
    Text,
}

impl OutputStyle {
    pub fn get_processor(
        &self,
        line_seperator: String,
        writer: Arc<Mutex<dyn std::io::Write + Send>>,
    ) -> Box<dyn Process> {
        match self {
            OutputStyle::ConsiseJson => Box::new(JsonProcess {
                titles: Titles::default(),
                line_seperator,
                printer: JsonPrinter::new(false, false),
                writer,
            }),
            OutputStyle::Json => Box::new(JsonProcess {
                titles: Titles::default(),
                line_seperator,
                printer: JsonPrinter::new(true, false),
                writer,
            }),
            OutputStyle::OneLineJson => Box::new(JsonProcess {
                titles: Titles::default(),
                line_seperator,
                printer: JsonPrinter::new(false, true),
                writer,
            }),
            OutputStyle::Csv => Box::new(CsvProcess {
                length: 0,
                line_seperator,
                printer: CsvPrinter {},
                writer,
            }),
            OutputStyle::Text => Box::new(RawProcess {
                length: 0,
                line_seperator,
                printer: RawTextPrinter {},
                writer,
            }),
        }
    }
}

struct CsvProcess {
    length: usize,
    line_seperator: String,
    printer: CsvPrinter,
    writer: Arc<Mutex<dyn std::io::Write + Send>>,
}
impl Process for CsvProcess {
    fn start(&mut self, titles: Titles) -> ProcessResult {
        self.length = titles.len();
        if self.length == 0 {
            return Err(ProcessError::InvalidInputError(
                "CSV output must have selection and can no group by",
            ));
        }
        let context = titles.as_context();
        self.process(context)
    }
    fn complete(&mut self) -> ProcessResult {
        Ok(())
    }
    fn process(&mut self, context: Context) -> ProcessResult {
        let mut string_row = Vec::new();
        for index in 0..self.length {
            let value = context.get(index);
            let str = if let Some(value) = value {
                let mut str = String::new();
                self.printer.print(&mut str, value)?;
                str
            } else {
                "".into()
            };
            string_row.push(str);
        }
        write!(
            self.writer.lock().unwrap(),
            "{}{}",
            string_row.join(", "),
            self.line_seperator
        )?;
        Ok(())
    }
}

struct JsonProcess {
    titles: Titles,
    line_seperator: String,
    printer: JsonPrinter,
    writer: Arc<Mutex<dyn std::io::Write + Send>>,
}

impl Process for JsonProcess {
    fn start(&mut self, titles: Titles) -> ProcessResult {
        self.titles = titles;
        Ok(())
    }
    fn complete(&mut self) -> ProcessResult {
        Ok(())
    }
    fn process(&mut self, context: Context) -> ProcessResult {
        if let Some(value) = context.build(&self.titles) {
            let mut str = String::new();
            self.printer.print(&mut str, &value)?;
            write!(
                self.writer.lock().unwrap(),
                "{}{}",
                str,
                self.line_seperator
            )?;
        }
        Ok(())
    }
}
struct RawProcess {
    length: usize,
    line_seperator: String,
    printer: RawTextPrinter,
    writer: Arc<Mutex<dyn std::io::Write + Send>>,
}
impl Process for RawProcess {
    fn complete(&mut self) -> ProcessResult {
        Ok(())
    }
    fn start(&mut self, titles: Titles) -> ProcessResult {
        self.length = titles.len();
        if self.length > 0 {
            let context = titles.as_context();
            self.process(context)
        } else {
            Ok(())
        }
    }
    fn process(&mut self, context: Context) -> ProcessResult {
        if self.length != 0 {
            for index in 0..self.length {
                let value = context.get(index);
                if let Some(value) = value {
                    let mut str = String::new();
                    self.printer.print(&mut str, value)?;
                    write!(self.writer.lock().unwrap(), "{}", str)?;
                    if index < self.length - 1 {
                        write!(self.writer.lock().unwrap(), "\t")?;
                    }
                }
            }
            write!(self.writer.lock().unwrap(), "{}", self.line_seperator)?;
        } else {
            let mut str = String::new();
            self.printer.print(&mut str, context.input().deref())?;
            write!(
                self.writer.lock().unwrap(),
                "{}{}",
                str,
                self.line_seperator
            )?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use std::str::FromStr;

    use crate::json_value::JsonValue;

    use super::*;

    #[test]
    fn csv_writer_start_fail_if_no_titles() -> ProcessResult {
        let style = OutputStyle::Csv;
        let text = Vec::new();
        let writer = Arc::new(Mutex::new(text));
        let mut proccsor = style.get_processor(String::new(), writer);

        let error = proccsor.start(Titles::default()).err().unwrap();

        assert_eq!(matches!(error, ProcessError::InvalidInputError(_)), true);

        Ok(())
    }

    #[test]
    fn csv_writer_start_will_print_titles_and_content() -> ProcessResult {
        let style = OutputStyle::Csv;
        let text = Vec::new();
        let writer = Arc::new(Mutex::new(text));
        let mut proccsor = style.get_processor("\n".into(), writer.clone());
        let titles = Titles::default()
            .with_title("one".into())
            .with_title("two".into());

        proccsor.start(titles)?;
        let result = Context::new_empty()
            .with_result(JsonValue::from_str("true").ok())
            .with_result(JsonValue::from_str(r#""te\"st""#).ok());
        proccsor.process(result)?;
        let result = Context::new_empty()
            .with_result(JsonValue::from_str("12").ok())
            .with_result(JsonValue::from_str("1.2").ok());
        proccsor.process(result)?;
        let result = Context::new_empty()
            .with_result(JsonValue::from_str(r#"{"key": 12}"#).ok())
            .with_result(JsonValue::from_str("[1, 2, 3]").ok());
        proccsor.process(result)?;
        let result = Context::new_empty()
            .with_result(JsonValue::from_str("null").ok())
            .with_result(JsonValue::from_str("false").ok());
        proccsor.process(result)?;
        let result = Context::new_empty()
            .with_result(JsonValue::from_str("-20").ok())
            .with_result(None);
        proccsor.process(result)?;

        let vec = writer.lock().unwrap().clone();
        let lines = String::from_utf8(vec).unwrap();
        let lines = lines.lines().collect::<Vec<_>>();

        assert_eq!(lines[0], "\"one\", \"two\"");
        assert_eq!(lines[1], "True, \"te\"\"st\"");
        assert_eq!(lines[2], "12, 1.2");
        assert_eq!(lines[3], "\"{\"\"key\"\":12}\", \"[1,2,3]\"");
        assert_eq!(lines[4], "null, False");
        assert_eq!(lines[5], "-20, ");

        Ok(())
    }

    #[test]
    fn consise_json_writer_start_will_print_titles_and_content() -> ProcessResult {
        let style = OutputStyle::ConsiseJson;
        let text = Vec::new();
        let writer = Arc::new(Mutex::new(text));
        let mut proccsor = style.get_processor("\n".into(), writer.clone());
        let titles = Titles::default()
            .with_title("one".into())
            .with_title("two".into());

        proccsor.start(titles)?;
        let result = Context::new_empty()
            .with_result(JsonValue::from_str("true").ok())
            .with_result(JsonValue::from_str(r#""te\"st""#).ok());
        proccsor.process(result)?;
        let result = Context::new_empty()
            .with_result(JsonValue::from_str("12").ok())
            .with_result(JsonValue::from_str("1.2").ok());
        proccsor.process(result)?;
        let result = Context::new_empty()
            .with_result(JsonValue::from_str(r#"{"key": 12}"#).ok())
            .with_result(JsonValue::from_str("[1, 2, 3]").ok());
        proccsor.process(result)?;
        let result = Context::new_empty()
            .with_result(JsonValue::from_str("null").ok())
            .with_result(JsonValue::from_str("false").ok());
        proccsor.process(result)?;
        let result = Context::new_empty()
            .with_result(JsonValue::from_str("-20").ok())
            .with_result(None);
        proccsor.process(result)?;

        let vec = writer.lock().unwrap().clone();
        let lines = String::from_utf8(vec).unwrap();
        let lines = lines.lines().collect::<Vec<_>>();

        assert_eq!(lines[0], r#"{"one":true,"two":"te\"st"}"#);
        assert_eq!(lines[1], r#"{"one":12,"two":1.2}"#);
        assert_eq!(lines[2], r#"{"one":{"key":12},"two":[1,2,3]}"#);
        assert_eq!(lines[3], r#"{"one":null,"two":false}"#);
        assert_eq!(lines[4], r#"{"one":-20}"#);

        Ok(())
    }

    #[test]
    fn json_writer_start_will_print_titles_and_content() -> ProcessResult {
        let style = OutputStyle::Json;
        let text = Vec::new();
        let writer = Arc::new(Mutex::new(text));
        let mut proccsor = style.get_processor("---".into(), writer.clone());
        let titles = Titles::default()
            .with_title("one".into())
            .with_title("two".into());

        proccsor.start(titles)?;
        let result = Context::new_empty()
            .with_result(JsonValue::from_str("true").ok())
            .with_result(JsonValue::from_str(r#""te\"st""#).ok());
        proccsor.process(result)?;
        let result = Context::new_empty()
            .with_result(JsonValue::from_str("12").ok())
            .with_result(JsonValue::from_str("1.2").ok());
        proccsor.process(result)?;
        let result = Context::new_empty()
            .with_result(JsonValue::from_str(r#"{"key": 12}"#).ok())
            .with_result(JsonValue::from_str("[1, 2, 3]").ok());
        proccsor.process(result)?;
        let result = Context::new_empty()
            .with_result(JsonValue::from_str("null").ok())
            .with_result(JsonValue::from_str("false").ok());
        proccsor.process(result)?;
        let result = Context::new_empty()
            .with_result(JsonValue::from_str("-20").ok())
            .with_result(None);
        proccsor.process(result)?;

        let vec = writer.lock().unwrap().clone();
        let lines = String::from_utf8(vec).unwrap();
        let lines = lines.split("---").collect::<Vec<_>>();

        assert_eq!(
            lines[0],
            r#"{
  "one": true,
  "two": "te\"st"
}"#
        );
        assert_eq!(
            lines[1],
            r#"{
  "one": 12,
  "two": 1.2
}"#
        );
        assert_eq!(
            lines[2],
            r#"{
  "one": {
    "key": 12
  },
  "two": [
    1,
    2,
    3
  ]
}"#
        );
        assert_eq!(
            lines[3],
            r#"{
  "one": null,
  "two": false
}"#
        );
        assert_eq!(
            lines[4],
            r#"{
  "one": -20
}"#
        );

        Ok(())
    }

    #[test]
    fn oneline_json_writer_start_will_print_titles_and_content() -> ProcessResult {
        let style = OutputStyle::OneLineJson;
        let text = Vec::new();
        let writer = Arc::new(Mutex::new(text));
        let mut proccsor = style.get_processor("\n".into(), writer.clone());
        let titles = Titles::default()
            .with_title("one".into())
            .with_title("two".into());

        proccsor.start(titles)?;
        let result = Context::new_empty()
            .with_result(JsonValue::from_str("true").ok())
            .with_result(JsonValue::from_str(r#""te\"st""#).ok());
        proccsor.process(result)?;
        let result = Context::new_empty()
            .with_result(JsonValue::from_str("12").ok())
            .with_result(JsonValue::from_str("1.2").ok());
        proccsor.process(result)?;
        let result = Context::new_empty()
            .with_result(JsonValue::from_str(r#"{"key": 12}"#).ok())
            .with_result(JsonValue::from_str("[1, 2, 3]").ok());
        proccsor.process(result)?;
        let result = Context::new_empty()
            .with_result(JsonValue::from_str("null").ok())
            .with_result(JsonValue::from_str("false").ok());
        proccsor.process(result)?;
        let result = Context::new_empty()
            .with_result(JsonValue::from_str("-20").ok())
            .with_result(None);
        proccsor.process(result)?;

        let vec = writer.lock().unwrap().clone();
        let lines = String::from_utf8(vec).unwrap();
        let lines = lines.lines().collect::<Vec<_>>();

        assert_eq!(lines[0], r#"{"one": true, "two": "te\"st"}"#);
        assert_eq!(lines[1], r#"{"one": 12, "two": 1.2}"#);
        assert_eq!(lines[2], r#"{"one": {"key": 12}, "two": [1, 2, 3]}"#);
        assert_eq!(lines[3], r#"{"one": null, "two": false}"#);
        assert_eq!(lines[4], r#"{"one": -20}"#);

        Ok(())
    }

    #[test]
    fn no_titles_raw_writer_start_will_print_no_titles_and_content() -> ProcessResult {
        let style = OutputStyle::Text;
        let text = Vec::new();
        let writer = Arc::new(Mutex::new(text));
        let mut proccsor = style.get_processor("\n".into(), writer.clone());
        let titles = Titles::default();

        proccsor.start(titles)?;
        let result = Context::new_with_input(JsonValue::from_str("true").unwrap());
        proccsor.process(result)?;
        let result = Context::new_with_input(JsonValue::from_str("1200").unwrap());
        proccsor.process(result)?;
        let result = Context::new_with_input(
            JsonValue::from_str(r#"{"one": {"key": 12}, "two": [1, 2, 3]}"#).unwrap(),
        );
        proccsor.process(result)?;
        let result = Context::new_with_input(JsonValue::from_str("[1, 2, 4]").unwrap());
        proccsor.process(result)?;

        let vec = writer.lock().unwrap().clone();
        let lines = String::from_utf8(vec).unwrap();
        let lines = lines.lines().collect::<Vec<_>>();

        assert_eq!(lines[0], "true");
        assert_eq!(lines[1], "1200");
        assert_eq!(lines[2], r#"{"one":{"key":12},"two":[1,2,3]}"#);
        assert_eq!(lines[3], "[1,2,4]");

        Ok(())
    }
    #[test]
    fn titled_raw_writer_start_will_print_titles_and_content() -> ProcessResult {
        let style = OutputStyle::Text;
        let text = Vec::new();
        let writer = Arc::new(Mutex::new(text));
        let mut proccsor = style.get_processor("\n".into(), writer.clone());
        let titles = Titles::default()
            .with_title("one".into())
            .with_title("two".into());

        proccsor.start(titles)?;
        let result = Context::new_empty()
            .with_result(JsonValue::from_str("true").ok())
            .with_result(JsonValue::from_str(r#""te\"st""#).ok());
        proccsor.process(result)?;
        let result = Context::new_empty()
            .with_result(JsonValue::from_str("12").ok())
            .with_result(JsonValue::from_str("1.2").ok());
        proccsor.process(result)?;
        let result = Context::new_empty()
            .with_result(JsonValue::from_str(r#"{"key": 12}"#).ok())
            .with_result(JsonValue::from_str("[1, 2, 3]").ok());
        proccsor.process(result)?;
        let result = Context::new_empty()
            .with_result(JsonValue::from_str("null").ok())
            .with_result(JsonValue::from_str("false").ok());
        proccsor.process(result)?;
        let result = Context::new_empty()
            .with_result(JsonValue::from_str("-20").ok())
            .with_result(None);
        proccsor.process(result)?;

        let vec = writer.lock().unwrap().clone();
        let lines = String::from_utf8(vec).unwrap();
        let lines = lines.lines().collect::<Vec<_>>();

        assert_eq!(lines[0], "one\ttwo");
        assert_eq!(lines[1], "true\tte\"st");
        assert_eq!(lines[2], "12\t1.2");
        assert_eq!(lines[3], "{\"key\":12}\t[1,2,3]");
        assert_eq!(lines[4], "null\tfalse");
        assert_eq!(lines[5], "-20\t");

        Ok(())
    }
}
