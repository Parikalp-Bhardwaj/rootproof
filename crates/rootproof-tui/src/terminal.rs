use std::io::{self, stdout};

use crossterm::{
    execute,
    terminal::{EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode},
};

use ratatui::{Terminal, backend::CrosstermBackend};

pub type RootProofTerminal = Terminal<CrosstermBackend<std::io::Stdout>>;

pub fn init_terminal() -> io::Result<RootProofTerminal> {
    enable_raw_mode()?;

    let mut stdout = stdout();

    execute!(stdout, EnterAlternateScreen)?;

    let backend = CrosstermBackend::new(stdout);

    Terminal::new(backend)
}

pub fn restore_terminal(terminal: &mut RootProofTerminal) -> io::Result<()> {
    disable_raw_mode()?;

    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;

    terminal.show_cursor()?;

    Ok(())
}
