use crate::json_parser::JsonParser;
use crate::json_parser::JsonParserError;
use crate::json_value::JsonValue;
use crate::json_value::NumberValue;
use crate::reader::from_string;
use crate::reader::Location;
use crate::reader::Reader;
use lazy_static::lazy_static;
use std::collections::HashMap;
use std::io::Error as IoError;
use std::io::Read;
use std::num::ParseIntError;
use std::str::FromStr;
use std::string::FromUtf8Error;
use std::sync::Arc;
use std::vec;
use thiserror::Error;

pub trait Get: Sync + Send {
    fn get(&self, value: &Option<JsonValue>) -> Option<JsonValue>;
}

#[derive(Clone)]
pub struct Selection {
    getter: Arc<Box<dyn Get>>,
    name: String,
}
type Factory = fn(args: Vec<Box<dyn Get>>) -> Box<dyn Get>;

struct Example {
    input: &'static str,
    arguments: Vec<&'static str>,
}
struct FunctionDefinitions {
    names: Vec<&'static str>,
    min_args_count: usize,
    max_args_count: usize,
    build_extractor: Factory,
    description: Vec<&'static str>,
    examples: Vec<Example>,
}

struct FunctionsGroup {
    name: &'static str,
    functions: Vec<FunctionDefinitions>,
}

pub type Result<T> = std::result::Result<T, SelectionParseError>;
#[derive(Debug, Error)]
pub enum SelectionParseError {
    #[error("{0}")]
    IoError(#[from] IoError),
    #[error("{0}")]
    JsonError(#[from] JsonParserError),
    #[error("{0}")]
    StringUtfError(#[from] FromUtf8Error),
    #[error("{0}")]
    NumberParseError(#[from] ParseIntError),
    #[error("FunctionDefinitions '{0}' is unknwon")]
    UnknownFunction(String),
    #[error("{0}: Mising argument for {1}, got only {2} needed {3}")]
    MissingArgument(Location, String, usize, usize),
    #[error("{0}, Missing key name")]
    MissingKey(Location),
    #[error("{0}: Too many arguments for {1}, got only {2} needed {3}")]
    TooManyArgument(Location, String, usize, usize),
    #[error("{0}: Expecting equals, got {1}")]
    ExpectingEquals(Location, char),
    #[error("Unexpected end of string")]
    UnexpectedEof,
}

impl FromStr for Selection {
    type Err = SelectionParseError;
    fn from_str(s: &str) -> Result<Self> {
        let source = s.to_string();
        let mut reader = from_string(&source);
        reader.eat_whitespace()?;
        let extractors = read_getter(&mut reader)?;
        reader.eat_whitespace()?;
        let name = match reader.peek()? {
            Some(b'=') => {
                reader.next()?;
                reader.eat_whitespace()?;
                let mut buf = vec![];
                while let Some(ch) = reader.peek()? {
                    buf.push(ch);
                    reader.next()?;
                }
                String::from_utf8(buf)?
            }
            Some(ch) => {
                return Err(SelectionParseError::ExpectingEquals(
                    reader.where_am_i(),
                    ch as char,
                ));
            }
            None => source.clone(),
        };
        let getter = Arc::new(extractors);
        Ok(Selection { name, getter })
    }
}

enum SingleExtract {
    ByKey(String),
    ByIndex(usize),
}
enum Extract {
    Root,
    Element(Vec<SingleExtract>),
}
enum BasicGetters {
    Const(JsonValue),
}
impl Get for BasicGetters {
    fn get(&self, _: &Option<JsonValue>) -> Option<JsonValue> {
        match self {
            BasicGetters::Const(val) => {
                let val = val.clone();
                Some(val)
            }
        }
    }
}
impl Get for Extract {
    fn get(&self, value: &Option<JsonValue>) -> Option<JsonValue> {
        match self {
            Extract::Root => value.clone(),
            Extract::Element(es) => {
                let mut val = value.clone();
                for e in es {
                    match val {
                        None => break,
                        Some(_) => val = e.get(&val),
                    }
                }
                val
            }
        }
    }
}
impl Get for SingleExtract {
    fn get(&self, value: &Option<JsonValue>) -> Option<JsonValue> {
        match value {
            Some(JsonValue::Array(list)) => match self {
                SingleExtract::ByIndex(index) => list.get(*index).cloned(),
                _ => None,
            },
            Some(JsonValue::Object(map)) => match self {
                SingleExtract::ByKey(key) => map.get(key).cloned(),
                _ => None,
            },
            _ => None,
        }
    }
}
fn read_getter<R: Read>(reader: &mut Reader<R>) -> Result<Box<dyn Get>> {
    reader.eat_whitespace()?;
    match reader.peek()? {
        None => Err(SelectionParseError::UnexpectedEof),
        Some(b'.') | Some(b'#') => {
            let ext = Extract::parse(reader)?;
            Ok(Box::new(ext))
        }
        Some(b'(') => parse_function(reader),
        _ => match reader.next_json_value()? {
            None => Err(SelectionParseError::UnexpectedEof),
            Some(val) => Ok(Box::new(BasicGetters::Const(val))),
        },
    }
}
impl Extract {
    fn parse<R: Read>(reader: &mut Reader<R>) -> Result<Self> {
        let mut ext = vec![];
        loop {
            match reader.peek()? {
                Some(b'.') => {
                    let key = Self::read_extract_key(reader)?;
                    if key.is_empty() {
                        if ext.is_empty() {
                            return Ok(Extract::Root);
                        } else {
                            return Err(SelectionParseError::MissingKey(reader.where_am_i()));
                        }
                    }
                    let es = SingleExtract::ByKey(key);
                    ext.push(es);
                }
                Some(b'#') => match Self::read_extract_index(reader)? {
                    None => {
                        if ext.is_empty() {
                            return Ok(Extract::Root);
                        } else {
                            return Err(SelectionParseError::MissingKey(reader.where_am_i()));
                        }
                    }
                    Some(index) => {
                        let es = SingleExtract::ByIndex(index);
                        ext.push(es);
                    }
                },
                _ => return Ok(Extract::Element(ext)),
            }
        }
    }
    fn read_extract_key<R: Read>(reader: &mut Reader<R>) -> Result<String> {
        let mut buf = Vec::new();
        loop {
            match reader.next()? {
                None => break,
                Some(ch) => {
                    if ch.is_ascii_whitespace()
                        || ch == b'.'
                        || ch == b','
                        || ch == b'='
                        || ch == b'('
                        || ch == b')'
                        || ch.is_ascii_control()
                        || ch == b'\"'
                        || ch == b']'
                        || ch == b'['
                        || ch == b'{'
                        || ch == b'}'
                        || ch == b'#'
                    {
                        break;
                    } else {
                        buf.push(ch)
                    }
                }
            }
        }
        let str = String::from_utf8(buf)?;
        Ok(str)
    }
    fn read_extract_index<R: Read>(reader: &mut Reader<R>) -> Result<Option<usize>> {
        let mut digits = Vec::new();
        reader.read_digits(&mut digits)?;
        let str = String::from_utf8(digits)?;
        if str.is_empty() {
            return Ok(None);
        }
        let number = str.parse::<usize>()?;
        Ok(Some(number))
    }
}

fn parse_function<R: Read>(reader: &mut Reader<R>) -> Result<Box<dyn Get>> {
    let name = read_function_name(reader)?;
    let function = find_function(&name)?;
    let mut args = Vec::new();
    loop {
        reader.eat_whitespace()?;
        match reader.peek()? {
            Some(b',') => {
                reader.next()?;
            }
            Some(b')') => break,
            None => {
                return Err(SelectionParseError::UnexpectedEof);
            }
            _ => {
                let arg = read_getter(reader)?;
                args.push(arg);
            }
        }
    }
    reader.next()?;
    if args.len() < function.min_args_count {
        return Err(SelectionParseError::MissingArgument(
            reader.where_am_i(),
            name.to_string(),
            function.min_args_count,
            args.len(),
        ));
    }
    if args.len() > function.max_args_count {
        return Err(SelectionParseError::TooManyArgument(
            reader.where_am_i(),
            name.to_string(),
            args.len(),
            function.min_args_count,
        ));
    }
    Ok((function.build_extractor)(args))
}

fn read_function_name<R: Read>(reader: &mut Reader<R>) -> Result<String> {
    reader.eat_whitespace()?;
    let mut buf = Vec::new();
    loop {
        match reader.next()? {
            None => break,
            Some(ch) => {
                if ch.is_ascii_whitespace()
                    || ch == b','
                    || ch == b'('
                    || ch == b')'
                    || ch.is_ascii_control()
                {
                    break;
                } else {
                    buf.push(ch)
                }
            }
        }
    }
    let str = String::from_utf8(buf)?;
    Ok(str)
}

impl Selection {
    pub fn name(&self) -> &String {
        &self.name
    }
}
impl Get for Selection {
    fn get(&self, value: &Option<JsonValue>) -> Option<JsonValue> {
        self.getter.get(value)
    }
}

struct SingleArgument {
    args: Vec<Box<dyn Get>>,
}
impl SingleArgument {
    fn new(args: Vec<Box<dyn Get>>) -> Self {
        SingleArgument { args }
    }

    fn apply(&self, value: &Option<JsonValue>) -> Option<JsonValue> {
        if let Some(arg) = self.args.get(0) {
            arg.get(value)
        } else {
            None
        }
    }
}

struct MultiArguments {
    args: Vec<Box<dyn Get>>,
}
impl MultiArguments {
    fn new(args: Vec<Box<dyn Get>>) -> Self {
        MultiArguments { args }
    }

    fn apply(&self, value: &Option<JsonValue>, index: usize) -> Option<JsonValue> {
        if let Some(arg) = self.args.get(index) {
            arg.get(value)
        } else {
            None
        }
    }
}

fn getter(args: Vec<Box<dyn Get>>) -> Box<dyn Get> {
    struct Impl(SingleArgument);
    impl Get for Impl {
        fn get(&self, value: &Option<JsonValue>) -> Option<JsonValue> {
            if let Some(JsonValue::Object(map)) = value {
                if let Some(JsonValue::String(key)) = self.0.apply(value) {
                    map.get(&key).cloned()
                } else {
                    None
                }
            } else if let Some(JsonValue::Array(array)) = value {
                if let Some(JsonValue::Number(NumberValue::Positive(index))) = self.0.apply(value) {
                    array.get(index as usize).cloned()
                } else {
                    None
                }
            } else {
                None
            }
        }
    }
    Box::new(Impl(SingleArgument::new(args)))
}

fn combine_extractors(args: Vec<Box<dyn Get>>) -> Box<dyn Get> {
    struct Impl(Vec<Box<dyn Get>>);
    impl Get for Impl {
        fn get(&self, value: &Option<JsonValue>) -> Option<JsonValue> {
            let mut value = value.clone();
            for e in &self.0 {
                value = e.get(&value);
            }
            value
        }
    }
    Box::new(Impl(args))
}

lazy_static! {
    static ref BASIC_FUNCTIONS: FunctionsGroup = FunctionsGroup{ name: "Basic functions", functions: vec![
        FunctionDefinitions {
            names: vec![".", "get", "#", "[]"],
            min_args_count: 1,
            max_args_count: 1,
            build_extractor: getter,
            description: vec!["Get an item from an array by index or from a map by key."],
            examples: vec![
                Example {
                    input: "[\"a\", \"b\", \"c\"]",
                    arguments: vec!["1"]
                },
                Example {
                    input: "{\"key-1\": 12, \"key-2\": 32}",
                    arguments: vec!["\"key-1\""]
                },
            ]
        },
        FunctionDefinitions {
            names: vec!["x", "X", "|"],
            min_args_count: 2,
            max_args_count: usize::MAX,
            build_extractor: combine_extractors,
            description: vec!["Pipe the output of one function to the next function."],
            examples: vec![Example {
                input: "{\"key\": [20, 40, 60, {\"key-2\": 100}]}",
                arguments: vec!["(get \"key\")", "(get 3)", "(get \"key-2\")"]
            },]
        },
        FunctionDefinitions {
            names: vec!["size", "count", "length", "len"],
            min_args_count: 0,
            max_args_count: 0,
            build_extractor: |_| {
                struct Impl;
                impl Get for Impl {
                    fn get(&self, value: &Option<JsonValue>) -> Option<JsonValue> {
                        if let Some(JsonValue::Array(array)) = value {
                            Some(array.len().into())
                        } else if let Some(JsonValue::Object(map)) = value {
                            Some(map.len().into())
                        } else if let Some(JsonValue::String(str)) = value {
                            Some(str.len().into())
                        } else {
                            None
                        }
                    }
                }
                Box::new(Impl)
            },
            description: vec![
                "Get the number of element in an array,",
                "the number of keys in an object or the number of characters",
                "in a string."
            ],
            examples: vec![
                Example {
                    input: "[1, 2, 3, 4]",
                    arguments: vec![]
                },
                Example {
                    input: "{\"key-1\": 1, \"key-2\": false}",
                    arguments: vec![]
                },
                Example {
                    input: "\"123456\"",
                    arguments: vec![]
                },
            ]
        },
        FunctionDefinitions {
            names: vec!["default", "defaults", "or_else"],
            min_args_count: 1,
            max_args_count: usize::MAX,
            build_extractor: |args| {
                struct Impl(Vec<Box<dyn Get>>);
                impl Get for Impl {
                    fn get(&self, value: &Option<JsonValue>) -> Option<JsonValue> {
                        for e in &self.0 {
                            let val = e.get(value);
                            if val.is_some() {
                                return val;
                            }
                        }
                        None
                    }
                }
                Box::new(Impl(args))
            },
            description: vec!["Get the first non empty value."],
            examples: vec![
                Example {
                    input: "{\"key-1\": 1, \"key-2\": false}",
                    arguments: vec!["(get 1)", "(get \"key-1\")", "22"]
                },
                Example {
                    input: "{\"key-1\": 1, \"key-2\": false}",
                    arguments: vec!["(get 1)", "(get \"key-3\")", "22"]
                },
            ]
        },
        FunctionDefinitions {
            names: vec!["unit", "U", "root"],
            min_args_count: 0,
            max_args_count: 0,
            build_extractor: |_| {
                struct Impl;
                impl Get for Impl {
                    fn get(&self, value: &Option<JsonValue>) -> Option<JsonValue> {
                        value.clone()
                    }
                }
                Box::new(Impl)
            },
            description: vec!["Return the input as is."],
            examples: vec![
                Example {
                    input: "{\"key-1\": 1, \"key-2\": false}",
                    arguments: vec![""]
                },
            ]
        },
        FunctionDefinitions {
            names: vec!["?", "if"],
            min_args_count: 3,
            max_args_count: 3,
            build_extractor: |args| {
                struct Impl(MultiArguments);
                impl Get for Impl {
                    fn get(&self, value: &Option<JsonValue>) -> Option<JsonValue> {
                        match self.0.apply(value, 0) {
                            Some(JsonValue::Boolean(true)) => self.0.apply(value, 1),
                            Some(JsonValue::Boolean(false)) => self.0.apply(value, 2),
                            _ => None,
                        }
                    }
                }
                Box::new(Impl(MultiArguments::new(args)))
            },
            description: vec![
                "Return the second argument if the first argument is true. Return the third argument",
                "if the first is false. Return nothing if the first argument is not Boolean"
            ],
            examples: vec![
                Example {
                    input: "null",
                    arguments: vec!["true", "12", "22"]
                },
                Example {
                    input: "null",
                    arguments: vec!["false", "12", "22"]
                },
                Example {
                    input: "[1, 2, 3]",
                    arguments: vec!["(array?)", "(. 1)", "(. 2)"]
                },
                Example {
                    input: "[1, 2, 3]",
                    arguments: vec!["(null?)", "(. 1)", "(. 2)"]
                },
            ]
        },
    ]};
    static ref TYPES_FUNCTIONS : FunctionsGroup = FunctionsGroup{ name: "Type functions", functions: vec![
        FunctionDefinitions {
            names: vec!["array?", "list?"],
            min_args_count: 0,
            max_args_count: 0,
            build_extractor: |_| {
                struct Impl;
                impl Get for Impl {
                    fn get(&self, value: &Option<JsonValue>) -> Option<JsonValue> {
                        match value {
                            Some(JsonValue::Array(_)) => Some(true.into()),
                            _ => Some(false.into()),
                        }
                    }
                }
                Box::new(Impl)
            },
            description: vec!["return true if the value is an array."],
            examples: vec![
                Example {
                    input: "[1, 2, 3, 4]",
                    arguments: vec![]
                },
                Example {
                    input: "312",
                    arguments: vec![]
                },
            ]
        },
        FunctionDefinitions {
            names: vec!["object?", "map?", "hash?"],
            min_args_count: 0,
            max_args_count: 0,
            build_extractor: |_| {
                struct Impl;
                impl Get for Impl {
                    fn get(&self, value: &Option<JsonValue>) -> Option<JsonValue> {
                        match value {
                            Some(JsonValue::Object(_)) => Some(true.into()),
                            _ => Some(false.into()),
                        }
                    }
                }
                Box::new(Impl)
            },
            description: vec!["return true if the value is an object."],
            examples: vec![
                Example {
                    input: "[1, 2, 3, 4]",
                    arguments: vec![]
                },
                Example {
                    input: "{\"key\": 12}",
                    arguments: vec![]
                },
            ]
        },
        FunctionDefinitions {
            names: vec!["null?", "nil?"],
            min_args_count: 0,
            max_args_count: 0,
            build_extractor: |_| {
                struct Impl;
                impl Get for Impl {
                    fn get(&self, value: &Option<JsonValue>) -> Option<JsonValue> {
                        match value {
                            Some(JsonValue::Null) => Some(true.into()),
                            _ => Some(false.into()),
                        }
                    }
                }
                Box::new(Impl)
            },
            description: vec!["return true if the value is a null."],
            examples: vec![
                Example {
                    input: "null",
                    arguments: vec![]
                },
                Example {
                    input: "{\"key\": 12}",
                    arguments: vec![]
                },
            ]
        },
        FunctionDefinitions {
            names: vec!["bool?", "boolean?"],
            min_args_count: 0,
            max_args_count: 0,
            build_extractor: |_| {
                struct Impl;
                impl Get for Impl {
                    fn get(&self, value: &Option<JsonValue>) -> Option<JsonValue> {
                        match value {
                            Some(JsonValue::Boolean(_)) => Some(true.into()),
                            _ => Some(false.into()),
                        }
                    }
                }
                Box::new(Impl)
            },
            description: vec!["return true if the value is a boolean."],
            examples: vec![
                Example {
                    input: "false",
                    arguments: vec![]
                },
                Example {
                    input: "1.32",
                    arguments: vec![]
                },
            ]
        },
        FunctionDefinitions {
            names: vec!["number?"],
            min_args_count: 0,
            max_args_count: 0,
            build_extractor: |_| {
                struct Impl;
                impl Get for Impl {
                    fn get(&self, value: &Option<JsonValue>) -> Option<JsonValue> {
                        match value {
                            Some(JsonValue::Number(_)) => Some(true.into()),
                            _ => Some(false.into()),
                        }
                    }
                }
                Box::new(Impl)
            },
            description: vec!["return true if the value is a number."],
            examples: vec![
                Example {
                    input: "null",
                    arguments: vec![]
                },
                Example {
                    input: "1.32",
                    arguments: vec![]
                },
            ]
        },
        FunctionDefinitions {
            names: vec!["string?"],
            min_args_count: 0,
            max_args_count: 0,
            build_extractor: |_| {
                struct Impl;
                impl Get for Impl {
                    fn get(&self, value: &Option<JsonValue>) -> Option<JsonValue> {
                        match value {
                            Some(JsonValue::String(_)) => Some(true.into()),
                            _ => Some(false.into()),
                        }
                    }
                }
                Box::new(Impl)
            },
            description: vec!["return true if the value is a string."],
            examples: vec![
                Example {
                    input: "\"one\"",
                    arguments: vec![]
                },
                Example {
                    input: "1.32",
                    arguments: vec![]
                },
            ]
        },
    ]};
    static ref NUMBER_FUNCTIONS : FunctionsGroup = FunctionsGroup{ name: "Number functions", functions: vec![
        FunctionDefinitions {
            names: vec!["+", "add", "plus"],
            min_args_count: 2,
            max_args_count: usize::MAX,
            build_extractor: |args| {
                struct Impl(MultiArguments);
                impl Get for Impl {
                    fn get(&self, value: &Option<JsonValue>) -> Option<JsonValue> {
                        let mut sum = 0.0;
                        for s in &self.0.args {
                            if let Some(JsonValue::Number(num)) = s.get(value) {
                                let num: f64 = num.into();
                                sum += num;
                            } else {
                                return None;
                            }
                        }
                        Some(sum.into())
                    }
                }
                Box::new(Impl(MultiArguments::new(args)))
            },
            description: vec![
                "If all the arguments are number, add them.",
            ],
            examples: vec![
                Example {
                    input: "null",
                    arguments: vec!["1", "3"],
                },
                Example {
                    input: "null",
                    arguments: vec!["1", "10", "-4.1", "0.1"],
                },
            ],
        },
        FunctionDefinitions {
            names: vec!["-", "take_away", "minus", "substruct"],
            min_args_count: 2,
            max_args_count: 2,
            build_extractor: |args| {
                struct Impl(MultiArguments);
                impl Get for Impl {
                    fn get(&self, value: &Option<JsonValue>) -> Option<JsonValue> {
                        if let (Some(JsonValue::Number(num1)), Some(JsonValue::Number(num2))) = (self.0.apply(value, 0), self.0.apply(value, 1)) {
                            let num1: f64 = num1.into();
                            let num2: f64 = num2.into();
                            Some((num1 - num2).into())
                        } else {
                            None
                        }
                    }
                }
                Box::new(Impl(MultiArguments::new(args)))
            },
            description: vec![
                "Substract the second argument from the first one if both are number.",
            ],
            examples: vec![
                Example {
                    input: "null",
                    arguments: vec!["100", "3"],
                },
                Example {
                    input: "null",
                    arguments: vec!["10", "3.2"],
                },
            ],
        },
        FunctionDefinitions {
            names: vec!["*", "times", "multiple"],
            min_args_count: 2,
            max_args_count: usize::MAX,
            build_extractor: |args| {
                struct Impl(MultiArguments);
                impl Get for Impl {
                    fn get(&self, value: &Option<JsonValue>) -> Option<JsonValue> {
                        let mut sum = 0.0;
                        for s in &self.0.args {
                            if let Some(JsonValue::Number(num)) = s.get(value) {
                                let num: f64 = num.into();
                                sum *= num;
                            } else {
                                return None;
                            }
                        }
                        Some(sum.into())
                    }
                }
                Box::new(Impl(MultiArguments::new(args)))
            },
            description: vec![
                "If all the arguments are number, multiply them.",
            ],
            examples: vec![
                Example {
                    input: "null",
                    arguments: vec!["2", "3"],
                },
                Example {
                    input: "null",
                    arguments: vec!["2", "15", "0.1"],
                },
            ],
        },
        FunctionDefinitions {
            names: vec!["/", "divide"],
            min_args_count: 2,
            max_args_count: 2,
            build_extractor: |args| {
                struct Impl(MultiArguments);
                impl Get for Impl {
                    fn get(&self, value: &Option<JsonValue>) -> Option<JsonValue> {
                        if let (Some(JsonValue::Number(num1)), Some(JsonValue::Number(num2))) = (self.0.apply(value, 0), self.0.apply(value, 1)) {
                            let num1: f64 = num1.into();
                            let num2: f64 = num2.into();
                            if num2 != 0.0 {
                                Some((num1/ num2).into())
                            } else {
                                None
                            }
                        } else {
                            None
                        }
                    }
                }
                Box::new(Impl(MultiArguments::new(args)))
            },
            description: vec![
                "Divide the firs argument by the second argument. If the second argument is 0 will return nothing",
            ],
            examples: vec![
                Example {
                    input: "null",
                    arguments: vec!["100", "25"],
                },
                Example {
                    input: "null",
                    arguments: vec!["7", "2"],
                },
            ],
        },
        FunctionDefinitions {
            names: vec!["%", "mod", "modulu"],
            min_args_count: 2,
            max_args_count: 2,
            build_extractor: |args| {
                struct Impl(MultiArguments);
                impl Get for Impl {
                    fn get(&self, value: &Option<JsonValue>) -> Option<JsonValue> {
                        if let (Some(JsonValue::Number(num1)), Some(JsonValue::Number(num2))) = (self.0.apply(value, 0), self.0.apply(value, 1)) {
                            let num1: f64 = num1.into();
                            let num2: f64 = num2.into();
                            if num2 != 0.0 {
                                Some((num1 % num2).into())
                            } else {
                                None
                            }
                        } else {
                            None
                        }
                    }
                }
                Box::new(Impl(MultiArguments::new(args)))
            },
            description: vec![
                "Find the modulu of the division of the firs argument by the second argument. If the second argument is 0 will return nothing",
            ],
            examples: vec![
                Example {
                    input: "null",
                    arguments: vec!["5", "3"],
                },
                Example {
                    input: "null",
                    arguments: vec!["7", "2"],
                },
            ],
        },

        ]};
    static ref OBJECT_FUNCTIONS : FunctionsGroup =    FunctionsGroup { name: "Object functions", functions:  vec![
        FunctionDefinitions {
            names: vec!["keys"],
            min_args_count: 0,
            max_args_count: 0,
            build_extractor: |_| {
                struct Impl;
                impl Get for Impl {
                    fn get(&self, value: &Option<JsonValue>) -> Option<JsonValue> {
                        if let Some(JsonValue::Object(map)) = value {
                            Some(JsonValue::Array(
                                map.keys().cloned().map(JsonValue::String).collect(),
                            ))
                        } else {
                            None
                        }
                    }
                }
                Box::new(Impl)
            },
            description: vec!["Get the list of keys from an object."],
            examples: vec![Example {
                input: "{\"key-1\": 1, \"key-2\": false}",
                arguments: vec![]
            },]
        },
        FunctionDefinitions {
            names: vec!["values", "vals"],
            min_args_count: 0,
            max_args_count: 0,
            build_extractor: |_| {
                struct Impl;
                impl Get for Impl {
                    fn get(&self, value: &Option<JsonValue>) -> Option<JsonValue> {
                        if let Some(JsonValue::Object(map)) = value {
                            Some(JsonValue::Array(map.values().cloned().collect()))
                        } else {
                            None
                        }
                    }
                }
                Box::new(Impl)
            },
            description: vec!["Get the list of values from an object."],
            examples: vec![Example {
                input: "{\"key-1\": 1, \"key-2\": false}",
                arguments: vec![]
            },]
        },
    ]};
    static ref LIST_FUNCTIONS : FunctionsGroup =    FunctionsGroup { name: "List functions", functions:  vec![
        FunctionDefinitions {
            names: vec!["filter"],
            min_args_count: 2,
            max_args_count: 2,
            build_extractor: |args| {
                struct Impl(MultiArguments);
                impl Get for Impl {
                    fn get(&self, value: &Option<JsonValue>) -> Option<JsonValue> {
                        match self.0.apply(value, 0) {
                            Some(JsonValue::Array(list)) => {
                                let list: Vec<_> = list.into_iter()

                                .filter(|v| {
                                    let v = Some(v.clone());
                                    matches!(self.0.apply(&v, 1), Some(JsonValue::Boolean(true)))
                                }
                                ).collect();
                                Some(list.into())
                            },
                            _ => None,
                        }
                    }
                }
                Box::new(Impl(MultiArguments::new(args)))
            },
            description: vec![
                "Filter a list.",
                "If the first argument is a list, return all the values for which the second argument is a list."
            ],
            examples: vec![
                Example {
                    input: "null",
                    arguments: vec!["[1, 2, 3, 4]", "true"]
                },
                Example {
                    input: "null",
                    arguments: vec!["[1, 2, 3, 4]", "null"]
                },
                Example {
                    input: "null",
                    arguments: vec!["[1, 2, 3, 4, \"one\", null]", "(string?)"]
                },
                Example {
                    input: "[1, 2, null, \"a\", 4]",
                    arguments: vec!["(unit)", "(number?)"]
                },
            ]
        },
        FunctionDefinitions {
            names: vec!["sort", "order"],
            min_args_count: 1,
            max_args_count: 1,
            build_extractor: |args| {
                struct Impl(SingleArgument);
                impl Get for Impl {
                    fn get(&self, value: &Option<JsonValue>) -> Option<JsonValue> {
                        match self.0.apply(value) {
                            Some(JsonValue::Array(list)) => {
                                let mut list: Vec<JsonValue> = list.clone();
                                list.sort();

                                Some(list.into())
                            },
                            _ => None,
                        }
                    }
                }
                Box::new(Impl(SingleArgument::new(args)))
            },
            description: vec![
                "Sort a list.",
                "If the first argument is a list, return list sorted."
            ],
            examples: vec![
                Example {
                    input: "null",
                    arguments: vec!["[1, -2, 3.01, 3.05, -544, 100]"]
                },
                Example {
                    input: "null",
                    arguments: vec!["[null, true, false, {}, [1, 2, 3], \"abc\", \"cde\", {\"key\": 12}]"]
                },
            ]
        },
        FunctionDefinitions {
            names: vec!["sort_by", "order_by"],
            min_args_count: 2,
            max_args_count: 2,
            build_extractor: |args| {
                struct Impl(MultiArguments);
                impl Get for Impl {
                    fn get(&self, value: &Option<JsonValue>) -> Option<JsonValue> {
                        match self.0.apply(value, 0) {
                            Some(JsonValue::Array(list)) => {
                                let mut list: Vec<JsonValue> = list.clone();
                                list.sort_by(|v1, v2| {
                                    let v1 = Some(v1.clone());
                                    let v1 = self.0.apply(&v1,1 );
                                    let v2 = Some(v2.clone());
                                    let v2 = self.0.apply(&v2,1 );
                                    v1.cmp(&v2)
                                });

                                Some(list.into())
                            },
                            _ => None,
                        }
                    }
                }
                Box::new(Impl(MultiArguments::new(args)))
            },
            description: vec![
                "Filter a list.",
                "If the first argument is a list, return list sorted by the second argument."
            ],
            examples: vec![
                Example {
                    input: "null",
                    arguments: vec!["[\"12345\", \"5\", \"23\", \"abc\", \"-1-2\", \"\"]", "(len)"]
                },
            ]
        },
    ]};
    static ref ALL_FUNCTIONS: Vec<&'static FunctionsGroup> = vec![
        &BASIC_FUNCTIONS,
        &TYPES_FUNCTIONS,
        &LIST_FUNCTIONS,
        &OBJECT_FUNCTIONS,
        &NUMBER_FUNCTIONS,
    ];
    static ref NAME_TO_FUNCTION: HashMap<&'static str, &'static FunctionDefinitions> = ALL_FUNCTIONS
        .iter()
        .flat_map(|l| l.functions.iter())
        .flat_map(|f| f.names.iter().map(move |n| (*n, f)))
        .collect();
}

fn find_function(name: &str) -> Result<&'static FunctionDefinitions> {
    NAME_TO_FUNCTION
        .get(name)
        .ok_or(SelectionParseError::UnknownFunction(name.to_string()))
        .map(|f| *f)
}

pub fn print_help() {
    for group in ALL_FUNCTIONS.iter() {
        println!("--- {} ---", group.name);
        for func in group.functions.iter() {
            let name = func.names.first().unwrap();
            println!("  {} function:", name);
            for alias in &func.names {
                println!("    * Can be called as '{}'", alias);
            }
            for description in &func.description {
                println!("    {}", description);
            }
            println!("    For example:");
            for example in &func.examples {
                let input = example.input.to_string();
                let mut reader = from_string(&input);
                let json = reader.next_json_value().unwrap().unwrap();
                println!("      * for input: \"{}\"", json);
                let args = example.arguments.join(", ");
                let run = format!("({} {})", name, args);
                let selection = Selection::from_str(&run).unwrap();
                let json = Some(json);
                let result = selection.getter.get(&json).unwrap();
                println!("        running: \"{}\"", run);
                println!("        will give: \"{}\"", result);
            }
        }
    }
}
