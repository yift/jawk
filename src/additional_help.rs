use std::io::{stdout, Error as IoError, IsTerminal, Stdout, Write};
use thiserror::Error;

use clap::builder::{PossibleValue, PossibleValuesParser};

#[cfg(all(not(target_os = "windows"), feature = "termimad-help"))]
use termimad::{crossterm::style::Color, Alignment, Error as TermimadError, MadSkin};

#[cfg(feature = "create-docs")]
use crate::build_docs::build_docs;
use crate::{
    functions_definitions::{create_possible_fn_help_types, get_fn_help},
    selection_help::get_selection_help,
};

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
#[cfg(all(not(target_os = "windows"), feature = "termimad-help"))]
fn make_skin() -> MadSkin {
    let mut skin = MadSkin::default();
    skin.table.align = Alignment::Center;
    skin.set_headers_fg(Color::DarkBlue);
    skin.bold.set_fg(Color::Red);
    skin.italic.set_fg(Color::DarkGreen);
    skin.code_block.align = Alignment::Left;
    skin.code_block.set_bg(Color::Blue);
    skin.code_block.set_fg(Color::Yellow);
    skin.inline_code.set_bg(Color::Reset);
    skin.inline_code.set_fg(Color::DarkMagenta);
    skin
}

#[cfg(all(not(target_os = "windows"), feature = "termimad-help"))]
fn display_help(w: &mut Stdout, help: &str) -> Result<(), HelpError> {
    let skin = make_skin();
    let help = skin.term_text(help);
    writeln!(w, "{help}")?;
    Ok(())
}
#[cfg(any(target_os = "windows", not(feature = "termimad-help")))]
fn display_help(w: &mut Stdout, help: &String) -> Result<(), HelpError> {
    printout_help(w, help)?;
    Ok(())
}
fn printout_help(w: &mut Stdout, help: &String) -> Result<(), HelpError> {
    writeln!(w, "{help}")?;
    Ok(())
}
pub fn display_additional_help(help_type: &str) -> Result<(), HelpError> {
    #[cfg(feature = "create-docs")]
    {
        if help_type == "book" {
            build_docs()?;
            return Ok(());
        }
    }
    let help_type = help_type.to_lowercase();
    let help = if help_type == "selection" {
        get_selection_help()
    } else {
        get_fn_help(&help_type)
    };
    let mut w = stdout();
    let help = help.join("\n");
    if w.is_terminal() {
        display_help(&mut w, &help)?;
    } else {
        printout_help(&mut w, &help)?;
    }

    Ok(())
}
#[derive(Debug, Error)]
pub enum HelpError {
    #[error("{0}")]
    Io(#[from] IoError),
    #[cfg(all(not(target_os = "windows"), feature = "termimad-help"))]
    #[error("{0}")]
    Termimad(#[from] TermimadError),
}
