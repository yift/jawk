use clap::builder::{PossibleValue, PossibleValuesParser};

#[cfg(feature = "create-docs")]
use crate::build_docs::build_docs;
use crate::functions_definitions::{create_possible_fn_help_types, get_fn_help_name};

pub trait AdditionalHelpFactory {
    fn get() -> Vec<String>;
}

pub fn create_possible_values() -> PossibleValuesParser {
    let mut values = create_possible_fn_help_types();
    values.insert(
        0,
        PossibleValue::new("selection").help("Additional help about the selection"),
    );
    #[cfg(feature = "create-docs")]
    {
        values.push(PossibleValue::new("book").help("Create a book").hide(true));
    }
    values.into()
}

pub fn display_additional_help(help_type: &str) {
    #[cfg(feature = "create-docs")]
    {
        if help_type == "book" {
            build_docs().unwrap();
            return;
        }
    }
    let help_type = help_type.to_lowercase();

    let help = if help_type == "selection" {
        help_type
    } else {
        get_fn_help_name(&help_type)
    };
    let root = option_env!("JAWK_BOOK_ROOT").unwrap_or("https://jawk.ykaplan.me/");
    let url = format!("{root}{help}.html");
    if open::that(&url).is_err() {
        println!("See additional help in {url}");
    }
}
