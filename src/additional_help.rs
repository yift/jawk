use std::io::{stdout, Error as IoError, IsTerminal, Stdout, Write};
use thiserror::Error;

use clap::builder::{PossibleValue, PossibleValuesParser};
#[cfg(all(not(target_os = "windows"), feature = "termimad-help"))]
use termimad::{
    crossterm::{
        cursor::{Hide, Show},
        event::{self, Event, KeyCode, KeyEvent},
        queue,
        style::Color,
        terminal::{self, Clear, ClearType, EnterAlternateScreen, LeaveAlternateScreen},
    },
    Alignment, Area, Error as TermimadError, MadSkin, MadView,
};

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
    values.into()
}
#[cfg(all(not(target_os = "windows"), feature = "termimad-help"))]
fn view_area() -> Area {
    let mut area = Area::full_screen();
    area.pad_for_max_width(200);
    area
}
#[cfg(all(not(target_os = "windows"), feature = "termimad-help"))]
fn make_skin() -> MadSkin {
    let mut skin = MadSkin::default();
    skin.table.align = Alignment::Center;
    skin.set_headers_fg(Color::DarkBlue);
    skin.bold.set_fg(Color::Red);
    skin.italic.set_fg(Color::DarkGreen);
    skin.scrollbar.thumb.set_fg(Color::Green);
    skin.code_block.align = Alignment::Left;
    skin.code_block.set_bg(Color::Blue);
    skin.code_block.set_fg(Color::Yellow);
    skin.inline_code.set_bg(Color::Reset);
    skin.inline_code.set_fg(Color::DarkMagenta);
    skin
}

#[cfg(all(not(target_os = "windows"), feature = "termimad-help"))]
fn display_help(w: &mut Stdout, help: &String) -> Result<(), HelpError> {
    queue!(w, EnterAlternateScreen)?;
    terminal::enable_raw_mode()?;
    queue!(w, Hide)?;
    let skin = make_skin();
    let mut view = MadView::from(help.to_owned(), view_area(), skin);
    loop {
        view.write_on(w)?;
        w.flush()?;
        match event::read() {
            Ok(Event::Key(KeyEvent { code, .. })) => match code {
                KeyCode::Up => view.try_scroll_lines(-1),
                KeyCode::Down => view.try_scroll_lines(1),
                KeyCode::PageUp => view.try_scroll_pages(-1),
                KeyCode::PageDown => view.try_scroll_pages(1),
                _ => break,
            },
            Ok(Event::Resize(..)) => {
                queue!(w, Clear(ClearType::All))?;
                view.resize(&view_area());
            }
            _ => {}
        }
    }
    terminal::disable_raw_mode()?;
    queue!(w, Show)?;
    queue!(w, LeaveAlternateScreen)?;
    Ok(())
}
#[cfg(any(target_os = "windows", not(feature = "termimad-help")))]
fn display_help(w: &mut Stdout, help: &String) -> Result<(), HelpError> {
    printout_help(w, help)?;
    Ok(())
}
fn printout_help(w: &mut Stdout, help: &String) -> Result<(), HelpError> {
    writeln!(w, "{}", help)?;
    Ok(())
}
pub fn display_additional_help(help_type: &str) -> Result<(), HelpError> {
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
