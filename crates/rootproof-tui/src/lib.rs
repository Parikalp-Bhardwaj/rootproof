mod app;
mod terminal;
mod ui;

use std::{io, path::PathBuf, time::Duration};

use crossterm::event::{self, Event, KeyCode, KeyEventKind, KeyModifiers};

use app::App;

use terminal::{init_terminal, restore_terminal};

pub fn run(repository: PathBuf, incident_input: Option<PathBuf>) -> io::Result<()> {
    let mut terminal = init_terminal()?;

    let result = run_app(&mut terminal, repository, incident_input);

    let restore_result = restore_terminal(&mut terminal);

    match (result, restore_result) {
        (Err(error), _) => Err(error),

        (Ok(()), Err(error)) => Err(error),

        (Ok(()), Ok(())) => Ok(()),
    }
}

fn run_app(
    terminal: &mut terminal::RootProofTerminal,
    repository: PathBuf,
    incident_input: Option<PathBuf>,
) -> io::Result<()> {
    let mut app = App::new(repository, incident_input);

    while !app.should_quit {
        terminal.draw(|frame| {
            ui::render(frame, &app);
        })?;

        if !event::poll(Duration::from_millis(100))? {
            continue;
        }

        let Event::Key(key) = event::read()? else {
            continue;
        };

        if key.kind != KeyEventKind::Press {
            continue;
        }

        match (key.code, key.modifiers) {
            (KeyCode::Char('q'), _) => {
                app.quit();
            }

            (KeyCode::Char('c'), modifiers) if modifiers.contains(KeyModifiers::CONTROL) => {
                app.quit();
            }

            (KeyCode::Tab, KeyModifiers::NONE) | (KeyCode::Right, _) => {
                app.next_panel();
            }

            (KeyCode::BackTab, _) | (KeyCode::Left, _) => {
                app.previous_panel();
            }

            _ => {}
        }
    }

    Ok(())
}
