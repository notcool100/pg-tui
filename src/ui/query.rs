use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    widgets::{Block, Borders, Paragraph, Row, Table, Wrap},
    Frame,
};

use crate::app::App;

pub fn render_query(f: &mut Frame, app: &App, area: Rect) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(10), Constraint::Min(0)])
        .split(area);

    // Query editor
    render_query_editor(f, app, chunks[0]);

    // Results
    render_query_results(f, app, chunks[1]);
}

fn render_query_editor(f: &mut Frame, app: &App, area: Rect) {
    let help_text = if app.query_input.is_empty() {
        "\n  Type your SQL query here\n  Press Ctrl+Enter or F5 to execute\n  Tab to switch to browser mode"
    } else {
        ""
    };

    let text = if app.query_input.is_empty() {
        help_text.to_string()
    } else {
        app.query_input.clone()
    };

    let editor = Paragraph::new(text)
        .style(Style::default().fg(Color::White))
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title("SQL Query Editor (Ctrl+Enter or F5 to execute)")
                .border_style(Style::default().fg(Color::Cyan)),
        )
        .wrap(Wrap { trim: false });

    f.render_widget(editor, area);
}

fn render_query_results(f: &mut Frame, app: &App, area: Rect) {
    if let Some(result) = &app.query_result {
        if result.rows.is_empty() {
            let empty = Paragraph::new("Query executed successfully. No rows returned.")
                .style(Style::default().fg(Color::Green))
                .block(
                    Block::default()
                        .borders(Borders::ALL)
                        .title("Results")
                        .border_style(Style::default().fg(Color::Cyan)),
                );
            f.render_widget(empty, area);
            return;
        }

        // Create table header
        let header = Row::new(result.columns.clone())
            .style(Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD))
            .bottom_margin(1);

        // Create table rows
        let rows: Vec<Row> = result
            .rows
            .iter()
            .map(|row| Row::new(row.clone()))
            .collect();

        // Calculate column widths
        let col_count = result.columns.len();
        let col_width = 100 / col_count as u16;
        let constraints: Vec<Constraint> = (0..col_count)
            .map(|_| Constraint::Percentage(col_width))
            .collect();

        let table = Table::new(rows, constraints)
            .header(header)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .title(format!("Results ({} rows)", result.row_count))
                    .border_style(Style::default().fg(Color::Cyan)),
            );

        f.render_widget(table, area);
    } else {
        let help = Paragraph::new("No query results yet.\n\nWrite a SQL query above and press !e to execute.")
            .style(Style::default().fg(Color::DarkGray))
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .title("Results")
                    .border_style(Style::default().fg(Color::Cyan)),
            );
        f.render_widget(help, area);
    }
}
