use std::io;

use config::AppConfig;
use logging::initialize_logging;
use ratatui::crossterm::execute;
use ratatui::crossterm::terminal::disable_raw_mode;
use ratatui::crossterm::terminal::enable_raw_mode;

use ratatui::{backend::CrosstermBackend, Terminal};

pub use app::App;
use cli::{Args, Parser};
use tracing::{debug, info};

mod action;
mod app;
mod cli;
mod command;
mod config;
mod export;
mod grade;
mod logging;

mod model;
mod ui;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // init logging
    initialize_logging()?;
    // parse args.
    let args = Args::parse();
    debug!("Found args: {:?}", &args);
    // setup terminal
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, crossterm::terminal::EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // create app and run it
    info!("Starting app ...");
    let mut app = if let Ok(config) = AppConfig::read_config() {
        App::new()
            .with_config(config)
            .with_points(args.points)
            .init()
    } else {
        App::new().with_points(args.points).init()
    };
    debug!("Debug mode active.");
    let _res = app.run(&mut terminal);
    info!("Terminate app with {:?}", _res);
    // restore terminal
    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        crossterm::terminal::LeaveAlternateScreen
    )?;
    terminal.show_cursor()?;
    Ok(())
}
