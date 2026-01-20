use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
    Frame,
};

use crate::app::{App, ConnectionField};

pub fn render_connection(f: &mut Frame, app: &App, area: Rect) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .margin(2)
        .constraints([
            Constraint::Length(3),
            Constraint::Length(3),
            Constraint::Length(3),
            Constraint::Length(3),
            Constraint::Length(3),
            Constraint::Length(3),
            Constraint::Min(0),
        ])
        .split(area);

    // Title
    let title = Paragraph::new("PostgreSQL TUI Client")
        .style(Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD))
        .alignment(Alignment::Center)
        .block(Block::default().borders(Borders::NONE));
    f.render_widget(title, chunks[0]);

    // Host
    render_input_field(
        f,
        "Host",
        &app.host,
        app.connection_field == ConnectionField::Host,
        chunks[1],
    );

    // Port
    render_input_field(
        f,
        "Port",
        &app.port,
        app.connection_field == ConnectionField::Port,
        chunks[2],
    );

    // Database
    render_input_field(
        f,
        "Database",
        &app.database,
        app.connection_field == ConnectionField::Database,
        chunks[3],
    );

    // User
    render_input_field(
        f,
        "User",
        &app.user,
        app.connection_field == ConnectionField::User,
        chunks[4],
    );

    // Password
    let masked_password = "*".repeat(app.password.len());
    render_input_field(
        f,
        "Password",
        &masked_password,
        app.connection_field == ConnectionField::Password,
        chunks[5],
    );

    // Instructions
    let instructions = Paragraph::new(vec![
        Line::from("Tab/Shift+Tab: Next/Previous field | Enter: Connect | q: Quit"),
        Line::from(Span::styled(
            "Note: Connection details (except password) are saved after first login",
            Style::default().fg(Color::DarkGray),
        )),
    ])
    .alignment(Alignment::Center);
    f.render_widget(instructions, chunks[6]);
}

fn render_input_field(
    f: &mut Frame,
    label: &str,
    value: &str,
    is_selected: bool,
    area: Rect,
) {
    let style = if is_selected {
        Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)
    } else {
        Style::default().fg(Color::White)
    };

    let border_style = if is_selected {
        Style::default().fg(Color::Cyan)
    } else {
        Style::default().fg(Color::Gray)
    };

    let input = Paragraph::new(value)
        .style(style)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(border_style)
                .title(label),
        );

    f.render_widget(input, area);
}
