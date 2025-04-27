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
mod tui;

mod model;
mod ui;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // init logging
    initialize_logging()?;

    // parse args.
    let args = Args::parse();
    debug!("Found args: {:?}", &args);

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
    let _res = app.run();
    info!("Terminate app with {:?}", _res);
    Ok(())
}
