use std::str::FromStr;

use crate::json_value::JsonValue;

struct UsageExample {
    selection: String,
    input: Option<JsonValue>,
    expected_output: Option<JsonValue>,
}
impl UsageExample {
    fn new(selection: &str, input: &str, expected_output: &str) -> Self {
        let expected_output = JsonValue::from_str(expected_output).ok();
        let input = JsonValue::from_str(input).ok();
        let selection = selection.to_string();
        UsageExample {
            selection,
            input,
            expected_output,
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
                "* Use `. to get the input as is.",
                "* Use `.<name>` to access a key within an object. If the name has spaces, use the `get` function instead.",
                "* Use `#<index> to access an element in an array.",
                "* Use `^` to access the \"parent\" input. That is,, while in a functional function - like `filter` or `map`, use `^` to access the original input and `^^` to access the input of that input and so on.",
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
                "Invoke a function. Has a format of `(<function-name>` <arg0> <arg1> ..)` where `<argN>` are other selection.",
                "Alternative format is `(.<function-name>` <arg1>...)` - in that case, the first argument will be the input (i.e. `.`).",
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
                "Use a variable (either one the was predefined by the `set` command line argument or one that was defined by the `set` function.",
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
                "Use a macro (either one the was predefined by the `set` command line argument or one that was defined by the `define` function.",
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
            )
    ]
}

pub fn print_selection_help() {
    let types = build_help();
    println!("There are {} types of selectoion:", types.len());
    for t in types {
        println!();
        println!("{}", t.name);
        for d in t.description {
            println!("  {}", d);
        }
        println!("  For example:");
        for e in t.examples {
            println!("      * For selection: `{}:`", e.selection);
            if let Some(o) = e.expected_output {
                println!("        will produce: `{}`", o);
            } else {
                println!("        will produce nothing");
            }
            if let Some(i) = e.input {
                println!("        for input: `{}.`", i);
            } else {
                println!("        regardless of the input.");
            }
        }
    }
}
#[cfg(test)]
mod tests {
    use crate::{
        processor::Context,
        reader::from_string,
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
                let context = match example.input {
                    Some(i) => {
                        println!("\tAnd input: {}", &i);
                        Context::new_with_input(i)
                    }
                    None => {
                        println!("\tAnd no input");
                        Context::new_empty()
                    }
                };

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
