use logging::initialize_logging;

pub use app::App;
use cli::{Args, Parser};
use tracing::{debug, info};

mod action;
mod app;
mod cli;
mod config;
mod export;
mod logging;
mod tui;

mod model;
mod ui;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // init logging
    initialize_logging()?;
    info!(
        "Starting {}::{}",
        env!("CARGO_PKG_NAME").to_uppercase(),
        env!("CARGO_PKG_VERSION")
    );

    // parse args.
    let args = Args::parse();
    debug!("ARGS: {:?}", &args);

    let mut app = App::new()
        .with_points(args.points)
        .with_course(args.course)
        .init();

    debug!("Debug mode active.");
    let _res = app.run();
    info!("Terminate app with {:?}", _res);
    Ok(())
}
