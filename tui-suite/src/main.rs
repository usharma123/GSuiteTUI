use std::io;
use std::panic;

use anyhow::Result;
use crossterm::{
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{backend::CrosstermBackend, Terminal};

use tui_suite::engine::run_app;

#[tokio::main]
async fn main() -> Result<()> {
    // Set up panic hook to restore terminal on panic
    let default_panic = panic::take_hook();
    panic::set_hook(Box::new(move |info| {
        let _ = restore_terminal();
        default_panic(info);
    }));

    // Set up terminal
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // Run the application
    let res = run_app(&mut terminal).await;

    // Restore terminal
    restore_terminal()?;

    res
}

fn restore_terminal() -> Result<()> {
    disable_raw_mode()?;
    execute!(io::stdout(), LeaveAlternateScreen)?;
    Ok(())
}
