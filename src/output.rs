use std::{
    ops::Deref,
    sync::{Arc, Mutex},
};

use crate::{
    printer::{CsvPrinter, JsonPrinter, Print, RawTextPrinter},
    processor::Result as ProcessResult,
    processor::{Context, Process, ProcessError, Titles},
    OutputStyle,
};

impl OutputStyle {
    pub fn get_processor(&self, line_seperator: String) -> Box<dyn Process> {
        let writer = Arc::new(Mutex::new(std::io::stdout()));
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
