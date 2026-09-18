use ratatui::{
    Frame,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph, Tabs, Wrap},
};

use crate::app::{ActivePanel, App};

pub fn render(frame: &mut Frame, app: &App) {
    let layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),
            Constraint::Length(3),
            Constraint::Min(8),
            Constraint::Length(3),
        ])
        .split(frame.area());

    render_header(frame, layout[0]);

    render_tabs(frame, layout[1], app);

    render_content(frame, layout[2], app);

    render_footer(frame, layout[3], app);
}

fn render_header(frame: &mut Frame, area: Rect) {
    let title = Paragraph::new(Line::from(vec![
        Span::styled("RootProof", Style::default().add_modifier(Modifier::BOLD)),
        Span::raw("  ·  Prove the root cause. Reproduce the failure. Validate the fix."),
    ]))
    .block(Block::default().borders(Borders::ALL));

    frame.render_widget(title, area);
}

fn render_tabs(frame: &mut Frame, area: Rect, app: &App) {
    let titles = ["Overview", "Evidence", "Hypotheses", "Reproduction", "Fix"];

    let selected = match app.active_panel {
        ActivePanel::Overview => 0,
        ActivePanel::Evidence => 1,
        ActivePanel::Hypotheses => 2,
        ActivePanel::Reproduction => 3,
        ActivePanel::Fix => 4,
    };

    let tabs = Tabs::new(titles)
        .select(selected)
        .highlight_style(Style::default().add_modifier(Modifier::BOLD))
        .divider(" │ ")
        .block(Block::default().borders(Borders::ALL));

    frame.render_widget(tabs, area);
}

fn render_content(frame: &mut Frame, area: Rect, app: &App) {
    match app.active_panel {
        ActivePanel::Overview => {
            render_overview(frame, area, app);
        }

        ActivePanel::Evidence => {
            render_placeholder(
                frame,
                area,
                "Evidence",
                "Evidence collected during investigation will appear here.",
            );
        }

        ActivePanel::Hypotheses => {
            render_placeholder(
                frame,
                area,
                "Hypotheses",
                "Ranked root-cause hypotheses will appear here.",
            );
        }

        ActivePanel::Reproduction => {
            render_placeholder(
                frame,
                area,
                "Reproduction",
                "Generated reproduction tests and matching results will appear here.",
            );
        }

        ActivePanel::Fix => {
            render_placeholder(
                frame,
                area,
                "Fix Validation",
                "Candidate fixes and isolated validation results will appear here.",
            );
        }
    }
}

fn render_overview(frame: &mut Frame, area: Rect, app: &App) {
    let input = app
        .incident_input
        .as_ref()
        .map(|path| path.display().to_string())
        .unwrap_or_else(|| "Not provided".to_owned());

    let content = vec![
        Line::from(vec![
            Span::styled(
                "Repository: ",
                Style::default().add_modifier(Modifier::BOLD),
            ),
            Span::raw(app.repository.display().to_string()),
        ]),
        Line::from(""),
        Line::from(vec![
            Span::styled("Incident: ", Style::default().add_modifier(Modifier::BOLD)),
            Span::raw(input),
        ]),
        Line::from(""),
        Line::from(vec![
            Span::styled("Status: ", Style::default().add_modifier(Modifier::BOLD)),
            Span::raw(&app.status),
        ]),
        Line::from(""),
        Line::from(
            "Stage 11A provides the terminal UI shell. Investigation results will be connected to these panels next.",
        ),
    ];

    let paragraph = Paragraph::new(content).wrap(Wrap { trim: false }).block(
        Block::default()
            .title(" Investigation ")
            .borders(Borders::ALL),
    );

    frame.render_widget(paragraph, area);
}

fn render_placeholder(frame: &mut Frame, area: Rect, title: &str, message: &str) {
    let paragraph = Paragraph::new(message).wrap(Wrap { trim: false }).block(
        Block::default()
            .title(format!(" {title} "))
            .borders(Borders::ALL),
    );

    frame.render_widget(paragraph, area);
}

fn render_footer(frame: &mut Frame, area: Rect, app: &App) {
    let footer = Paragraph::new(Line::from(vec![
        Span::styled("q", Style::default().add_modifier(Modifier::BOLD)),
        Span::raw(" quit   "),
        Span::styled("Tab / →", Style::default().add_modifier(Modifier::BOLD)),
        Span::raw(" next   "),
        Span::styled(
            "Shift+Tab / ←",
            Style::default().add_modifier(Modifier::BOLD),
        ),
        Span::raw(" previous   "),
        Span::raw(format!("Panel: {}", app.active_panel.title())),
    ]))
    .block(Block::default().borders(Borders::ALL));

    frame.render_widget(footer, area);
}
