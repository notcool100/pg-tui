use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Style},
    text::Span,
    widgets::{Block, Borders, Paragraph},
    Frame,
};

use crate::app::{App, AppMode};

mod connection_selector;
mod connection;
mod browser;
mod query;

pub fn render(f: &mut Frame, app: &mut App) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Min(0), Constraint::Length(3)])
        .split(f.area());

    // Main content area
    match app.mode {
        AppMode::ConnectionSelector => connection_selector::render_connection_selector(f, app, chunks[0]),
        AppMode::ConnectionEdit => connection::render_connection(f, app, chunks[0]),
        AppMode::Browser => {
            let main_chunks = Layout::default()
                .direction(Direction::Horizontal)
                .constraints([Constraint::Percentage(30), Constraint::Percentage(70)])
                .split(chunks[0]);
            
            browser::render_browser(f, app, main_chunks[0]);
            browser::render_details(f, app, main_chunks[1]);
        }
        AppMode::Query => query::render_query(f, app, chunks[0]),
    }

    // Status bar
    render_status_bar(f, app, chunks[1]);
}

fn render_status_bar(f: &mut Frame, app: &App, area: Rect) {
    let mode_text = match app.mode {
        AppMode::ConnectionSelector => "CONNECTION MANAGER",
        AppMode::ConnectionEdit => "EDIT CONNECTION",
        AppMode::Browser => "BROWSER",
        AppMode::Query => "QUERY",
    };

    let status_text = if let Some(err) = &app.error_message {
        format!(" {} | ERROR: {} ", mode_text, err)
    } else {
        match app.mode {
            AppMode::ConnectionSelector => {
                if app.config.connections.is_empty() {
                    format!(" {} | n:new connection | q:quit ", mode_text)
                } else {
                    format!(" {} | ↑↓:navigate | Enter:select | n:new | d:delete | q:quit ", mode_text)
                }
            }
            AppMode::ConnectionEdit => format!(" {} | Tab:next field | Enter:connect | Esc:back | q:quit ", mode_text),
            AppMode::Browser => format!(" {} | ↑↓:navigate | Enter:expand | Tab:query mode | r:refresh | q:quit ", mode_text),
            AppMode::Query => format!(" {} | Ctrl+Enter/F5:execute | Tab:browser mode | q:quit ", mode_text),
        }
    };

    let status_style = if app.error_message.is_some() {
        Style::default().fg(Color::Red).bg(Color::Black)
    } else {
        Style::default().fg(Color::Cyan).bg(Color::Black)
    };

    let status = Paragraph::new(status_text)
        .style(status_style)
        .block(Block::default().borders(Borders::ALL));

    f.render_widget(status, area);
}
