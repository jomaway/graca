use color_eyre::eyre::Result;
use ratatui::backend::CrosstermBackend as Backend;
use ratatui::crossterm::{
    cursor,
    event::{DisableMouseCapture, EnableMouseCapture},
    terminal::{EnterAlternateScreen, LeaveAlternateScreen},
};
// use tokio::{
//     sync::{mpsc, Mutex},
//     task::JoinHandle,
// };

// pub type Frame<'a> = ratatui::Frame<'a, Backend<std::io::Stdout>>;

pub struct Tui {
    pub terminal: ratatui::Terminal<Backend<std::io::Stdout>>,
}

impl Tui {
    pub fn new() -> Result<Self> {
        let terminal = ratatui::Terminal::new(Backend::new(std::io::stdout()))?;
        Ok(Self { terminal })
    }

    pub fn enter(&self) -> Result<()> {
        crossterm::terminal::enable_raw_mode()?;
        crossterm::execute!(
            std::io::stdout(),
            EnterAlternateScreen,
            EnableMouseCapture,
            cursor::Hide
        )?;
        Ok(())
    }

    pub fn exit(&mut self) -> Result<()> {
        crossterm::execute!(
            self.terminal.backend_mut(),
            LeaveAlternateScreen,
            DisableMouseCapture,
            cursor::Show
        )?;
        crossterm::terminal::disable_raw_mode()?;
        Ok(())
    }

    // pub fn suspend(&self) -> Result<()> {
    //     self.exit()?;
    //     #[cfg(not(windows))]
    //     signal_hook::low_level::raise(signal_hook::consts::signal::SIGTSTP)?;
    //     Ok(())
    // }

    // pub fn resume(&self) -> Result<()> {
    //     self.enter()?;
    //     Ok(())
    // }
}
