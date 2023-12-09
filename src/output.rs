use std::{fmt::Result, sync::Arc};

use indexmap::IndexMap;

use crate::{
    json_value::JsonValue,
    printer::{CsvPrinter, JsonPrinter, Print, RawTextPrinter},
    OutputStyle,
};

pub trait Output: Sync + Send {
    fn start(&mut self) -> Result {
        Ok(())
    }
    fn output_row(&mut self, value: &JsonValue, row: Vec<Option<JsonValue>>) -> Result;
    fn done(&mut self) -> Result {
        Ok(())
    }
    fn without_titles(&self) -> Option<Box<dyn Output>>;
}

struct JsonOutput {
    line_seperator: String,
    rows_titles: Arc<Vec<String>>,
    printer: Arc<JsonPrinter>,
}

pub fn get_value_or_values(
    value: &JsonValue,
    row: Vec<Option<JsonValue>>,
    rows_titles: &Arc<Vec<String>>,
) -> JsonValue {
    if rows_titles.is_empty() {
        value.clone()
    } else {
        let mut data = IndexMap::new();
        rows_titles.iter().zip(row).for_each(|(title, value)| {
            if let Some(value) = value {
                data.insert(title.clone(), value);
            }
        });
        data.into()
    }
}

impl Output for JsonOutput {
    fn output_row(&mut self, value: &JsonValue, row: Vec<Option<JsonValue>>) -> Result {
        let data = get_value_or_values(value, row, &self.rows_titles);
        let mut str = String::new();
        self.printer.print(&mut str, &data)?;
        print!("{}", str);
        print!("{}", self.line_seperator);
        Ok(())
    }

    fn without_titles(&self) -> Option<Box<dyn Output>> {
        let printer = JsonOutput {
            line_seperator: self.line_seperator.clone(),
            printer: self.printer.clone(),
            rows_titles: Arc::new(vec![]),
        };
        Some(Box::new(printer))
    }
}

struct CsvOutut {
    line_seperator: String,
    printer: CsvPrinter,
    rows_titles: Arc<Vec<String>>,
}
impl CsvOutut {
    fn new(line_seperator: String, rows_titles: Arc<Vec<String>>) -> Self {
        CsvOutut {
            line_seperator,
            printer: CsvPrinter {},
            rows_titles: rows_titles.clone(),
        }
    }
}
impl Output for CsvOutut {
    fn output_row(&mut self, _: &JsonValue, row: Vec<Option<JsonValue>>) -> Result {
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
    fn start(&mut self) -> Result {
        let row = self.rows_titles.iter().map(|f| Some(f.into())).collect();
        self.output_row(&JsonValue::Null, row)
    }
    fn without_titles(&self) -> Option<Box<dyn Output>> {
        panic!("CSV output must have headers")
    }
}

struct RawOutput {
    line_seperator: String,
    printer: RawTextPrinter,
    rows_titles: Arc<Vec<String>>,
}
impl RawOutput {
    fn new(line_seperator: String, rows_titles: Arc<Vec<String>>) -> Self {
        RawOutput {
            line_seperator,
            printer: RawTextPrinter {},
            rows_titles: rows_titles.clone(),
        }
    }
}
impl Output for RawOutput {
    fn output_row(&mut self, value: &JsonValue, row: Vec<Option<JsonValue>>) -> Result {
        if !row.is_empty() {
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
        } else {
            let mut str = String::new();
            self.printer.print(&mut str, value)?;
            print!("{}", str);
            print!("{}", self.line_seperator);
        }
        Ok(())
    }
    fn start(&mut self) -> Result {
        let row = self.rows_titles.iter().map(|f| Some(f.into())).collect();
        self.output_row(&JsonValue::Null, row)
    }
    fn without_titles(&self) -> Option<Box<dyn Output>> {
        let output = RawOutput {
            line_seperator: self.line_seperator.clone(),
            printer: RawTextPrinter {},
            rows_titles: Arc::new(vec![]),
        };
        Some(Box::new(output))
    }
}

pub fn get_output(
    style: OutputStyle,
    rows_titles: Arc<Vec<String>>,
    line_seperator: String,
) -> Box<dyn Output> {
    match style {
        OutputStyle::ConsiseJson => Box::new(JsonOutput {
            line_seperator,
            printer: Arc::new(JsonPrinter::new(false, false)),
            rows_titles,
        }),
        OutputStyle::Json => Box::new(JsonOutput {
            line_seperator,
            printer: Arc::new(JsonPrinter::new(true, false)),
            rows_titles,
        }),
        OutputStyle::OneLineJson => Box::new(JsonOutput {
            line_seperator,
            printer: Arc::new(JsonPrinter::new(false, true)),
            rows_titles,
        }),
        OutputStyle::Csv => {
            if rows_titles.is_empty() {
                panic!("CSV output must contain a selection")
            }
            Box::new(CsvOutut::new(line_seperator, rows_titles))
        }
        OutputStyle::Text => Box::new(RawOutput::new(line_seperator, rows_titles)),
    }
}
