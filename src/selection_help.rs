use std::{collections::HashMap, str::FromStr};

use crate::json_value::JsonValue;

struct UsageExample {
    selection: String,
    input: String,
    expected_output: Option<JsonValue>,
    previous_selection: HashMap<String, Option<JsonValue>>,
}
impl UsageExample {
    fn new(selection: &str, input: &str, expected_output: &str) -> Self {
        let expected_output = JsonValue::from_str(expected_output).ok();
        let input = input.to_string();
        let selection = selection.to_string();
        UsageExample {
            selection,
            input,
            expected_output,
            previous_selection: HashMap::new(),
        }
    }

    fn with_previous_selection(self, name: &str, value: &str) -> Self {
        let mut previous_selection = self.previous_selection.clone();
        previous_selection.insert(name.into(), JsonValue::from_str(value).ok());
        UsageExample {
            selection: self.selection,
            input: self.input,
            expected_output: self.expected_output,
            previous_selection,
        }
    }
}
struct SelectionHelp {
    name: &'static str,
    description: Vec<&'static str>,
    examples: Vec<UsageExample>,
}
impl SelectionHelp {
    fn new(name: &'static str, description: Vec<&'static str>) -> Self {
        SelectionHelp {
            name,
            description,
            examples: vec![],
        }
    }

    fn with_example(mut self, example: UsageExample) -> Self {
        self.examples.push(example);
        self
    }
}
fn build_help() -> Vec<SelectionHelp> {
    vec![
        SelectionHelp::new(
            "Extraction",
            vec![
                "Used to extract data from the input.",
                "* Use `.` to get the input as is.",
                "* Use `.<name>` to access a key within an object. If the name has spaces, use the `get` function instead.",
                "* Use `#<index>` to access an element in an array.",
                "* Use `^` to access the \"parent\" input. That is, while in a functional function - like `filter` or `map` - use `^` to access the original input and `^^` to access the input of that input and so on.",
                "* One can use a combination of all of the aboove, i.e. `^.key-1.key-2#3.key-4`."
            ]
        )
            .with_example(UsageExample::new(".", r#"{"key": 12}"#, r#"{"key": 12}"#))
            .with_example(UsageExample::new(".key-1.key-2", r#"{"key-1": {"key-2": 300}}"#, "300"))
            .with_example(UsageExample::new(".key-1.key-2", r#"{"key-3": {"key-2": 300}}"#, ""))
            .with_example(
                UsageExample::new(".key-1#3", r#"{"key-1": ["a", "b", 3, "word", 6]}"#, r#""word""#)
            )
            .with_example(
                UsageExample::new(
                    "(.map (.map (.+ (size ^.) (size ^^.))))",
                    "[[1, 2, 3], [4, 5], [6]]",
                    "[[7, 8, 9], [9, 10], [10]]"
                )
            ),
        SelectionHelp::new(
            "Literal value",
            vec!["Used to select constant value. Use a simple JSON format. The input is ignored."]
        )
            .with_example(UsageExample::new("4001.1", "", "4001.1"))
            .with_example(UsageExample::new("null", "", "null"))
            .with_example(UsageExample::new(r#""test""#, "", r#""test""#))
            .with_example(UsageExample::new("[1, 4, {}, 100]", "", "[1, 4, {}, 100]")),
        SelectionHelp::new(
            "Function",
            vec![
                "Invoke a function. Has a format of `(<function-name> <arg0> <arg1> ..)` where `<argN>` are other selection.",
                "Alternative format is `(.<function-name> <arg1>...)` - in that case, the first argument will be the input (i.e. `.`).",
                "The argument can be seperated by comma or whitespace.",
                "See list of available functions in functions additional help."
            ]
        )
            .with_example(UsageExample::new("(len .)", "[1, 4, {}, 100]", "4"))
            .with_example(UsageExample::new("(.len)", "[1, 4, {}, 100]", "4"))
            .with_example(UsageExample::new("(len .list)", r#"{"list": [1, 4, {}, 100]}"#, "4"))
            .with_example(
                UsageExample::new(
                    "(map (range 10) (+ . 5))",
                    "",
                    "[5, 6, 7, 8, 9, 10, 11, 12, 13, 14]"
                )
            ),
        SelectionHelp::new(
            "Variables",
            vec![
                "Use a variable (either one the was predefined by the `set` command line argument or one that was defined by the `set` function).",
                "The format to use varaibles is `:<variable-name>`. Note that the variable is defiend once."
            ]
        )
            .with_example(UsageExample::new(":nothing", "", ""))
            .with_example(
                UsageExample::new(
                    r#"(set "length" (.len) (.map (.+ :length)))"#,
                    "[1, 2, 3]",
                    "[4, 5, 6]"
                )
            ),
        SelectionHelp::new(
            "Macros",
            vec![
                "Use a macro (either one the was predefined by the `set` command line argument or one that was defined by the `define` function).",
                "The format to use a macro is `@<variable-name>`. Note that the macro is evelated on each call."
            ]
        )
            .with_example(UsageExample::new(":nothing", "", ""))
            .with_example(
                UsageExample::new(
                    r#"(define "even" (= 0 (.% 2)) (.filter @even))"#,
                    "[1, 2, 3, 4, 5, 6, 7]",
                    "[2, 4, 6]"
                )
            ),
        SelectionHelp::new(
            "Input context",
            vec![
                "Use input context to get the context of the input. The available input types are:",
                "* `&index` - To get the index of the current value within the current run.",
                "* `&index-in-file` - To get the index of the current value within the curent file.",
                "* `&started-at-line-number` - To get the line number within the input file in which the input started.",
                "* `&started-at-char-number` - To get the char number within the line within the input file in which the input started.",
                "* `&ended-at-line-number` - To get the line number within the input file in which the input ended.",
                "* `&ended-at-char-number` - To get the char number within the line within the input file in which the input ended.",
                "* `&file-name` - To get the name of the input file from which the input was parsed (will be empty for stdin input)."
            ]
        ).with_example(
            UsageExample::new(
                r"(- &ended-at-char-number &started-at-char-number)",
                r#""test""#,
                "6"
            )
        ),
        SelectionHelp::new(
            "Previous selected values",
            vec![
                "Reuse previoulsy selected value. Use this to reuse a value that had been selected previously. This is not available during filtering, and one can only refere to values that had been selected before.",
                "The format is `/<selection-name>/` where the *selection-name* is the name of the selection."
            ]
        )
            .with_example(
                UsageExample::new("/name/", "", "\"John\"").with_previous_selection(
                    "name",
                    "\"John\""
                )
            )
            .with_example(UsageExample::new("/name/", "", "").with_previous_selection("name", ""))
            .with_example(
                UsageExample::new("/name/", "", "").with_previous_selection("Last Name", "\"Doe\"")
            )
    ]
}

pub fn get_selection_help() -> Vec<String> {
    let types = build_help();
    let mut help = Vec::new();
    help.push("# Selection".into());
    help.push(format!("There are {} types of selectoion:", types.len()));
    for t in types {
        help.push(String::new());
        help.push(format!("## {}", t.name));
        for d in t.description {
            help.push(format!("  {d}"));
        }
        help.push("### Examples".into());
        for e in t.examples {
            let previous_selection = if !e.previous_selection.is_empty() {
                e.previous_selection
                    .iter()
                    .map(|(key, value)| {
                        if let Some(value) = value {
                            format!(" and previously selected *{key}* as `{value}`")
                        } else {
                            format!(" and previously selected *{key}* as nothing")
                        }
                    })
                    .collect()
            } else {
                String::new()
            };
            let input = if let Ok(i) = JsonValue::from_str(e.input.as_str()) {
                format!("for input: `{i}`")
            } else {
                "regardless of the input".into()
            };
            let output = if let Some(o) = e.expected_output {
                format!("will produce: `{o}`")
            } else {
                "will produce nothing".into()
            };
            help.push(format!(
                "* For selection: `{}` {}{} {}.",
                e.selection, input, previous_selection, output
            ));
        }
        help.push("---".into());
    }
    help
}
#[cfg(test)]
mod tests {
    use std::rc::Rc;

    use crate::{
        json_parser::JsonParser,
        processor::Context,
        reader::from_string,
        regex_cache::RegexCache,
        selection::{self, read_getter},
    };

    use super::*;

    #[test]
    fn test_examples() -> selection::Result<()> {
        let types = build_help();
        for t in types {
            println!("Running examples for: {}", t.name);
            for example in t.examples {
                println!("\tFor selection: {}", &example.selection);
                let mut reader = from_string(&example.selection);
                let getter = read_getter(&mut reader).unwrap();
                let mut context_reader = from_string(&example.input);
                let started = context_reader.where_am_i();
                let input = context_reader.next_json_value()?;
                let ended = context_reader.where_am_i();
                let mut context = match input {
                    Some(i) => {
                        println!("\tAnd input: {}", &i);
                        Context::new_with_input(i, started, ended, 0, 0, &RegexCache::new(0))
                    }
                    None => {
                        println!("\tAnd no input");
                        Context::new_empty()
                    }
                };
                for (key, value) in example.previous_selection {
                    context = context.with_result(&Rc::new(key), value);
                }

                let output = getter.get(&context);
                match &output {
                    Some(i) => println!("\tGot output: {}", &i),
                    None => println!("\tGot no output"),
                }
                assert_eq!(output, example.expected_output);
            }
        }
        Ok(())
    }
}
