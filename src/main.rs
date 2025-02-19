use std::io;

use config::AppConfig;
use ratatui::crossterm::execute;
use ratatui::crossterm::terminal::disable_raw_mode;
use ratatui::crossterm::terminal::enable_raw_mode;

use ratatui::{backend::CrosstermBackend, Terminal};

pub use app::App;
use cli::{Args, Parser};

pub mod app;
mod cli;
mod config;
mod export;
mod grade;
mod helpers;
mod ui;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // parse args.
    let args = Args::parse();
    // setup terminal
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, crossterm::terminal::EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // create app and run it
    let mut app = if let Ok(config) = AppConfig::read_config() {
        App::new().with_config(config).with_points(args.points)
    } else {
        App::new().with_points(args.points)
    };
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
