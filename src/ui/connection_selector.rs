use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, Paragraph},
    Frame,
};

use crate::app::App;

pub fn render_connection_selector(f: &mut Frame, app: &App, area: Rect) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .margin(2)
        .constraints([
            Constraint::Length(3),
            Constraint::Min(5),
            Constraint::Length(5),
        ])
        .split(area);

    // Title
    let title = Paragraph::new("PostgreSQL Connection Manager")
        .style(Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD))
        .alignment(Alignment::Center)
        .block(Block::default().borders(Borders::NONE));
    f.render_widget(title, chunks[0]);

    // Connection list
    if app.config.connections.is_empty() {
        let empty = Paragraph::new("No saved connections.\nPress 'n' to create a new connection.")
            .style(Style::default().fg(Color::DarkGray))
            .alignment(Alignment::Center)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .title("Saved Connections")
                    .border_style(Style::default().fg(Color::Cyan)),
            );
        f.render_widget(empty, chunks[1]);
    } else {
        let items: Vec<ListItem> = app
            .config
            .connections
            .iter()
            .enumerate()
            .map(|(i, profile)| {
                let content = format!(
                    "{} - {}:{}/{}",
                    profile.name, profile.host, profile.port, profile.database
                );
                
                let style = if i == app.selected_profile {
                    Style::default()
                        .fg(Color::Yellow)
                        .add_modifier(Modifier::BOLD)
                } else {
                    Style::default().fg(Color::White)
                };

                ListItem::new(content).style(style)
            })
            .collect();

        let list = List::new(items).block(
            Block::default()
                .borders(Borders::ALL)
                .title("Saved Connections")
                .border_style(Style::default().fg(Color::Cyan)),
        );

        f.render_widget(list, chunks[1]);
    }

    // Instructions
    let instructions = Paragraph::new(vec![
        Line::from("↑/↓: Navigate | Enter: Connect | n: New Connection"),
        Line::from("e: Edit Selected | d: Delete Selected | q: Quit"),
    ])
    .style(Style::default().fg(Color::DarkGray))
    .alignment(Alignment::Center);
    f.render_widget(instructions, chunks[2]);
}
