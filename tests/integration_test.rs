use clap::Parser;
use jawk::{Cli, Master};
use std::cell::RefCell;
use std::io::Read;
use std::ops::Deref;
use std::rc::Rc;
use std::{fs, fs::File, path::Path};

#[test]
fn test_examples() {
    tests_in_dir("./tests/integration/examples/");
}

fn tests_in_dir<P: AsRef<Path>>(path: P) {
    let paths = fs::read_dir(path).unwrap();
    for path in paths {
        let path = path.unwrap().path();
        if path.is_dir() {
            tests_in_dir(&path);
        } else if path.file_name().unwrap().to_str() == Some("args.txt") {
            test_example(path.parent().unwrap());
        }
    }
}

fn test_example(dir: &Path) {
    println!("Looking into {}", dir.display());
    let description = dir.join("description.txt");
    if description.exists() {
        let description = fs::read_to_string(description).unwrap();
        println!("Description: {}", description);
    }
    let args = dir.join("args.txt");
    let args = fs::read_to_string(args).unwrap();
    let args: Vec<_> = args.lines().collect();
    let cli = Cli::parse_from(args);
    let stdout = Rc::new(RefCell::new(Vec::new()));
    let stderr = Rc::new(RefCell::new(Vec::new()));
    let input_file = if cfg!(windows) {
        let file = dir.join("input.txt");
        if file.exists() {
            let mut input_file = File::open(file).unwrap();
            let mut input_text = String::new();
            input_file.read_to_string(&mut input_text).unwrap();
            input_text = input_text.replace('\r', "");

            let target_dir = dir.join("target");
            fs::create_dir_all(&target_dir).ok();
            let tmp = target_dir.join("input.txt");
            fs::write(&tmp, input_text).unwrap();
            tmp
        } else {
            dir.join("input.txt")
        }
    } else {
        dir.join("input.txt")
    };
    let input_file = Rc::new(input_file.clone());
    let stdin = Box::new(move || File::open(input_file.deref().clone()).unwrap());
    let master = Master::new(cli, stdout.clone(), stderr.clone(), stdin);
    let result = master.go();

    let results_file = dir.join("results.txt");
    if results_file.exists() {
        let mut results_file = File::open(results_file).unwrap();
        let mut expected_result = String::new();
        results_file.read_to_string(&mut expected_result).unwrap();
        if result.is_err() {
            let result = format!("{:?}", result).to_string();
            assert_eq!(result, expected_result);
        } else {
            panic!("Expecting error: {}, got nothing", expected_result);
        }
    } else if result.is_err() {
        panic!("Expecting to pass, failed with: {:?}", result);
    }

    let error_file = dir.join("error.txt");
    let error_text = String::from_utf8(stderr.borrow().clone()).unwrap();
    if error_file.exists() {
        let mut error_file = File::open(error_file).unwrap();
        let mut expected_error = String::new();
        error_file.read_to_string(&mut expected_error).unwrap();
        if cfg!(windows) {
            expected_error = expected_error.replace('\r', "");
        }
        assert_eq!(expected_error, error_text);
    } else if !error_text.is_empty() {
        panic!("Expecting no error, got {}", error_text);
    }

    let output_file = if cfg!(windows) {
        let windows_file = dir.join("output.windows.txt");
        if windows_file.exists() {
            windows_file
        } else {
            dir.join("output.txt")
        }
    } else {
        dir.join("output.txt")
    };
    let output_text = String::from_utf8(stdout.borrow().clone()).unwrap();
    if output_file.exists() {
        let mut output_file = File::open(output_file).unwrap();
        let mut expected_output = String::new();
        output_file.read_to_string(&mut expected_output).unwrap();
        if cfg!(windows) {
            expected_output = expected_output.replace('\r', "");
        }
        assert_eq!(expected_output, output_text)
    } else if !output_text.is_empty() {
        panic!("Expecting no output, got {}", output_text);
    }
}
