use std::fmt::Result;

use indexmap::IndexMap;

use crate::{
    json_value::JsonValue,
    printer::{CsvPrinter, JsonPrinter, Print, RawTextPrinter},
    OutputStyle,
};

pub trait Output: Sync + Send {
    fn output_row(&self, row: Vec<Option<JsonValue>>) -> Result;
}

struct JsonOutput {
    line_seperator: String,
    rows_titles: Vec<String>,
    printer: JsonPrinter,
}

impl Output for JsonOutput {
    fn output_row(&self, row: Vec<Option<JsonValue>>) -> Result {
        let mut data = IndexMap::new();
        self.rows_titles.iter().zip(row).for_each(|(title, value)| {
            if let Some(value) = value {
                data.insert(title.clone(), value);
            }
        });
        let mut str = String::new();
        let value = data.into();
        self.printer.print(&mut str, &value)?;
        print!("{}", str);
        print!("{}", self.line_seperator);
        Ok(())
    }
}

struct EmptyJsonOutput {
    line_seperator: String,
    printer: JsonPrinter,
}

impl Output for EmptyJsonOutput {
    fn output_row(&self, row: Vec<Option<JsonValue>>) -> Result {
        for value in row.iter().flatten() {
            let mut str = String::new();
            self.printer.print(&mut str, value)?;
            print!("{}", str);
            print!("{}", self.line_seperator);
        }
        Ok(())
    }
}

struct CsvOutut {
    line_seperator: String,
    printer: CsvPrinter,
}
impl CsvOutut {
    fn new(line_seperator: String, rows_titles: Vec<String>) -> Self {
        let me = CsvOutut {
            line_seperator,
            printer: CsvPrinter {},
        };
        let row = rows_titles.iter().map(|f| Some(f.into())).collect();
        me.output_row(row).unwrap();
        me
    }
}
impl Output for CsvOutut {
    fn output_row(&self, row: Vec<Option<JsonValue>>) -> Result {
        let mut string_row = Vec::new();
        for value in row {
            let str = if let Some(value) = value {
                let mut str = String::new();
                self.printer.print(&mut str, &value)?;
                str
            } else {
                "".into()
            };
            string_row.push(str);
        }
        print!("{}", string_row.join(", "));
        print!("{}", self.line_seperator);
        Ok(())
    }
}

struct RawOutput {
    line_seperator: String,
    printer: RawTextPrinter,
}
impl RawOutput {
    fn new(line_seperator: String, rows_titles: Vec<String>) -> Self {
        let me = RawOutput {
            line_seperator,
            printer: RawTextPrinter {},
        };
        let row = rows_titles.iter().map(|f| Some(f.into())).collect();
        me.output_row(row).unwrap();
        me
    }
}
impl Output for RawOutput {
    fn output_row(&self, row: Vec<Option<JsonValue>>) -> Result {
        let mut string_row = Vec::new();
        for value in row {
            let str = if let Some(value) = value {
                let mut str = String::new();
                self.printer.print(&mut str, &value)?;
                str
            } else {
                "".into()
            };
            string_row.push(str);
        }
        print!("{}", string_row.join("\t"));
        print!("{}", self.line_seperator);
        Ok(())
    }
}

pub fn get_output(
    style: OutputStyle,
    rows_titles: Vec<String>,
    line_seperator: String,
) -> Box<dyn Output> {
    if rows_titles.is_empty() {
        match style {
            OutputStyle::ConsiseJson => Box::new(EmptyJsonOutput {
                line_seperator,
                printer: JsonPrinter::new(false, false),
            }),
            OutputStyle::Json => Box::new(EmptyJsonOutput {
                line_seperator,
                printer: JsonPrinter::new(true, false),
            }),
            OutputStyle::OneLineJson => Box::new(EmptyJsonOutput {
                line_seperator,
                printer: JsonPrinter::new(false, true),
            }),
            OutputStyle::Csv => panic!("CSV output must contain a selection"),
            OutputStyle::Text => Box::new(RawOutput::new(line_seperator, rows_titles)),
        }
    } else {
        match style {
            OutputStyle::ConsiseJson => Box::new(JsonOutput {
                line_seperator,
                printer: JsonPrinter::new(false, false),
                rows_titles,
            }),
            OutputStyle::Json => Box::new(JsonOutput {
                line_seperator,
                printer: JsonPrinter::new(true, false),
                rows_titles,
            }),
            OutputStyle::OneLineJson => Box::new(JsonOutput {
                line_seperator,
                printer: JsonPrinter::new(false, true),
                rows_titles,
            }),
            OutputStyle::Csv => Box::new(CsvOutut::new(line_seperator, rows_titles)),
            OutputStyle::Text => Box::new(RawOutput::new(line_seperator, rows_titles)),
        }
    }
}
