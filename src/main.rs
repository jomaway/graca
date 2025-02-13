use std::io;

use ratatui::crossterm::execute;
use ratatui::crossterm::terminal::disable_raw_mode;
use ratatui::crossterm::terminal::enable_raw_mode;

use ratatui::{backend::CrosstermBackend, Terminal};

pub use app::App;
use cli::{Args, Parser};

pub mod app;
mod cli;
mod grade;
mod helpers;
mod ui;
mod export;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();
    // setup terminal
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, crossterm::terminal::EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // create app and run it
    let mut app = App::new();
    app.set_points(args.points);
    let _res = app.run(&mut terminal);

    // restore terminal
    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        crossterm::terminal::LeaveAlternateScreen
    )?;
    terminal.show_cursor()?;

    Ok(())
}
