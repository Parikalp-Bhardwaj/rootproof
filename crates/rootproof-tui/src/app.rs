use std::path::PathBuf;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ActivePanel {
    Overview,
    Evidence,
    Hypotheses,
    Reproduction,
    Fix,
}

impl ActivePanel {
    pub fn next(self) -> Self {
        match self {
            Self::Overview => Self::Evidence,

            Self::Evidence => Self::Hypotheses,

            Self::Hypotheses => Self::Reproduction,

            Self::Reproduction => Self::Fix,

            Self::Fix => Self::Overview,
        }
    }

    pub fn previous(self) -> Self {
        match self {
            Self::Overview => Self::Fix,

            Self::Evidence => Self::Overview,

            Self::Hypotheses => Self::Evidence,

            Self::Reproduction => Self::Hypotheses,

            Self::Fix => Self::Reproduction,
        }
    }

    pub fn title(self) -> &'static str {
        match self {
            Self::Overview => "Overview",

            Self::Evidence => "Evidence",

            Self::Hypotheses => "Hypotheses",

            Self::Reproduction => "Reproduction",

            Self::Fix => "Fix Validation",
        }
    }
}

#[derive(Debug)]
pub struct App {
    pub should_quit: bool,
    pub active_panel: ActivePanel,

    pub repository: PathBuf,
    pub incident_input: Option<PathBuf>,

    pub status: String,
}

impl App {
    pub fn new(repository: PathBuf, incident_input: Option<PathBuf>) -> Self {
        Self {
            should_quit: false,

            active_panel: ActivePanel::Overview,

            repository,

            incident_input,

            status: "Ready".to_owned(),
        }
    }

    pub fn quit(&mut self) {
        self.should_quit = true;
    }

    pub fn next_panel(&mut self) {
        self.active_panel = self.active_panel.next();
    }

    pub fn previous_panel(&mut self) {
        self.active_panel = self.active_panel.previous();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn panels_cycle_forward() {
        let mut app = App::new(PathBuf::from("."), None);

        assert_eq!(app.active_panel, ActivePanel::Overview);

        app.next_panel();

        assert_eq!(app.active_panel, ActivePanel::Evidence);
    }

    #[test]
    fn panels_cycle_backward() {
        let mut app = App::new(PathBuf::from("."), None);

        app.previous_panel();

        assert_eq!(app.active_panel, ActivePanel::Fix);
    }

    #[test]
    fn quit_sets_exit_flag() {
        let mut app = App::new(PathBuf::from("."), None);

        app.quit();

        assert!(app.should_quit);
    }
}
