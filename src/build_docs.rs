use crate::{
    functions_definitions::{get_fn_help, get_groups, FunctionsGroup},
    selection_help::get_selection_help,
    Cli,
};
use mdbook::MDBook;
use regex::Regex;
use std::{
    fs,
    path::{Path, PathBuf},
};

use std::io::Result;

fn copy_dir(source: &PathBuf, target: &PathBuf) -> Result<()> {
    fs::create_dir_all(target)?;
    for entry in fs::read_dir(source)? {
        let entry = entry?;
        let ty = entry.file_type()?;
        if ty.is_dir() {
            copy_dir(&entry.path(), &target.join(entry.file_name()))?;
        } else {
            fs::copy(entry.path(), target.join(entry.file_name()))?;
        }
    }
    Ok(())
}

fn copy_source() -> Result<PathBuf> {
    let source = PathBuf::from("book");
    let target = PathBuf::from("target/docs/book");
    copy_dir(&source, &target)?;
    Ok(target.join("src"))
}

fn build_group_docs(
    group: &FunctionsGroup,
    target: &PathBuf,
    indentation: &String,
) -> Result<String> {
    let mut summary = String::new();
    let group_name = group.group_name;
    let group_file = target.join(format!("{group_name}.md"));
    let code = get_fn_help(group_name).join("\n");
    fs::write(group_file, code)?;
    if group.is_root() {
        summary += format!("\n{indentation}- [Functions]({group_name}.md)").as_str();
    } else {
        summary += format!("\n{indentation}- [{group_name} functions]({group_name}.md)").as_str();
    }

    for function in group.functions() {
        let function_name = function.name();
        println!("Creating {function_name}");
        let function_file_name = function.file_name();
        let function_file = target.join(format!("{function_file_name}.md"));
        let code = get_fn_help(&function_name).join("\n");
        let code = add_links(&code);
        fs::write(function_file, code)?;
        summary +=
            format!("\n {indentation} - [{function_name}]({function_file_name}.md)").as_str();
    }
    for g in group.subgroups() {
        summary += build_group_docs(g, target, &format!("{indentation}  "))?.as_str();
    }

    Ok(summary)
}

pub fn build_docs() -> Result<()> {
    let target = copy_source()?;
    let selection = target.join("selection.md");
    let code = get_selection_help().join("\n");
    fs::write(selection, code)?;

    let summary = build_group_docs(get_groups(), &target, &"".to_string())?;
    let summary_file = target.join("SUMMARY.md");
    let old_summary = fs::read_to_string(&summary_file)?;
    let new_summary = old_summary.replace("<function_groups>", summary.as_str());

    // Add help output
    let help_output = clap_markdown::help_markdown::<Cli>();
    let help_file = target.join("help.md");
    let help = fs::read_to_string(&help_file)?;
    let new_help = help.replace("<help>", help_output.as_str());
    fs::write(help_file, new_help)?;

    // 3. Copy Examples
    let examples = create_examples(&target)?;
    let new_summary = new_summary.replace("<examples>", examples.as_str());
    fs::write(summary_file, new_summary)?;

    let book = MDBook::load(target.parent().unwrap()).unwrap();
    book.build().unwrap();

    Ok(())
}

fn add_links(code: &str) -> String {
    let replacer = Regex::new(r"\[(?<link>https://[a-z0-9./_#]+)\]").unwrap();
    replacer.replace_all(code, "[$link]($link)").to_string()
}

fn create_examples(target: &PathBuf) -> Result<String> {
    let source = PathBuf::from("tests").join("integration").join("examples");

    create_example_dir(&source, target)
}

fn create_example_dir(source: &PathBuf, target: &PathBuf) -> Result<String> {
    if let Some(summary) = create_single_example(source, target)? {
        Ok(summary)
    } else {
        let paths = fs::read_dir(source)?;
        let mut dirs: Vec<_> = paths
            .into_iter()
            .filter_map(Result::ok)
            .map(|f| f.path())
            .filter(|t| t.is_dir())
            .collect();
        dirs.sort();
        let dirs: String = dirs
            .iter()
            .filter_map(|t| create_single_example(t, target).ok())
            .flatten()
            .collect();
        Ok(dirs)
    }
}

fn create_single_example(source: &Path, target: &PathBuf) -> Result<Option<String>> {
    let title = source.join("title.txt");
    if title.exists() {
        fs::create_dir_all(target)?;
        let title = fs::read_to_string(title)?;
        let lower_case_title = title.to_ascii_lowercase();
        let md_file_name = title.replace(' ', "_");
        let example_file = target.join(format!("{md_file_name}.md"));
        let input = fs::read_to_string(source.join("input.txt"))?;
        let args: String = bash_args(&fs::read_to_string(source.join("args.txt"))?);
        let output = fs::read_to_string(source.join("output.txt"))?;

        let md = format!(
            r#"
# {title}
In this example we will see how to {lower_case_title}.

If your input looks like
```json
{input}
```
You can use `jawk` like:
```bash
{args}
```
To produce:
```
{output}
```

"#
        );

        fs::write(example_file, md)?;
        Ok(Some(format!("\n    - [{title}]({md_file_name}.md)")))
    } else {
        Ok(None)
    }
}

fn bash_args(args: &str) -> String {
    args.lines()
        .enumerate()
        .map(|(index, line)| {
            if index == 0 {
                line.to_string()
            } else {
                format!(" \\\n    {line}")
            }
        })
        .map(|line| {
            if let Some(eq) = line.find('=') {
                let arg = line[eq + 1..].to_string();
                let arg = if arg.contains(' ')
                    || arg.contains('"')
                    || arg.contains('=')
                    || arg.contains('(')
                    || arg.contains(')')
                {
                    format!("'{arg}'")
                } else {
                    arg
                };
                format!("{} {}", &line[..eq], arg)
            } else {
                line.to_string()
            }
        })
        .collect()
}
