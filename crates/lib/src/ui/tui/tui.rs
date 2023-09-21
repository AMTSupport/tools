/*
 * Copyright (C) 2023 James Draycott <me@racci.dev>
 *
 * This program is free software: you can redistribute it and/or modify
 * it under the terms of the GNU General Public License as published by
 * the Free Software Foundation, version 3.
 *
 * This program is distributed in the hope that it will be useful,
 * but WITHOUT ANY WARRANTY; without even the implied warranty of
 * MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
 * GNU General Public License for more details.
 *
 * You should have received a copy of the GNU General Public License
 * along with this program.  If not, see <https://www.gnu.org/licenses/>.
 */

use crate::ui::tui::event::EventHandler;
use anyhow::Result;
use crossterm::event::{DisableMouseCapture, EnableMouseCapture, KeyEvent};
use crossterm::terminal::{self, EnterAlternateScreen, LeaveAlternateScreen};
use ratatouille::prelude::CrosstermBackend;
use ratatouille::Terminal;
use std::io;
use std::io::Stdout;
use std::panic;
use tracing::debug;

/// Representation of a terminal user interface.
///
/// It is responsible for setting up the terminal,
/// initializing the interface and handling the draw events.
// #[derive(Debug)]
// pub struct Tui<'a> {
//     /// Interface to the Terminal.
//     terminal: Option<Terminal<CrosstermBackend<Stdout>>>,
//
//     pub titles: Vec<&'a str>,
//     pub index: usize,
//
//     /// Terminal event handler.
//     pub events: EventHandler<?>,
// }

pub trait Tui<'a> {
    type App;

    type Args;

    type Events: EventHandler;

    /// Constructs a new instance of [`Tui`].
    fn new(events: Self::Events, args: Self::Args) -> Result<Self>
    where
        Self: Sized;

    /// Initializes the terminal interface.
    ///
    /// It enables the raw mode and sets terminal properties.
    fn init(&mut self) -> Result<()> {
        terminal::enable_raw_mode()?;
        let mut stdout = io::stdout();
        crossterm::execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
        let backend = CrosstermBackend::new(stdout);
        match self.set_terminal(Terminal::new(backend)?) {
            None => {}
            Some(_) => debug!("dropping old terminal"),
        }

        // Define a custom panic hook to reset the terminal properties.
        // This way, you won't have your terminal messed up if an unexpected error happens.
        let panic_hook = panic::take_hook();
        panic::set_hook(Box::new(move |panic| {
            reset().expect("failed to reset the terminal");
            panic_hook(panic);
        }));

        self.get_terminal().hide_cursor()?;
        self.get_terminal().clear()?;
        Ok(())
    }

    /// Exits the terminal interface.
    ///
    /// It disables the raw mode and reverts back the terminal properties.
    fn exit(&mut self) -> Result<()> {
        reset()?;
        self.get_terminal().show_cursor()?;
        Ok(())
    }

    /// [`Draw`] the terminal interface by [`rendering`] the widgets.
    ///
    /// [`Draw`]: tui::Terminal::draw
    /// [`rendering`]: crate::ui:render
    fn draw(&mut self, app: &mut Self::App) -> Result<()>;

    fn handle_key_events(&mut self, key_event: KeyEvent, app: &mut Self::App) -> Result<()>;

    fn get_terminal(&mut self) -> &mut Terminal<CrosstermBackend<Stdout>>;

    fn set_terminal(
        &mut self,
        terminal: Terminal<CrosstermBackend<Stdout>>,
    ) -> Option<Terminal<CrosstermBackend<Stdout>>>;
}

pub trait Paged {
    fn next(&mut self) {
        self.set_page((self.get_page() + 1) % self.pages());
    }

    fn prev(&mut self) {
        self.set_page((self.get_page() - 1) % self.pages());
    }

    fn pages(&self) -> usize;

    fn get_page(&self) -> usize;

    fn set_page(&mut self, index: usize);
}

/// Resets the terminal interface.
///
/// This function is also used for the panic hook to revert
/// the terminal properties if unexpected errors occur.
fn reset() -> Result<()> {
    terminal::disable_raw_mode()?;
    crossterm::execute!(io::stdout(), LeaveAlternateScreen, DisableMouseCapture)?;
    Ok(())
}
