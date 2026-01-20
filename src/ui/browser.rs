use ratatui::{
    layout::Rect,
    style::{Color, Modifier, Style},
    widgets::{Block, Borders, List, ListItem, Paragraph, Row, Table},
    Frame,
};

use crate::app::{App, BrowserItem};

pub fn render_browser(f: &mut Frame, app: &mut App, area: Rect) {
    // Calculate visible height (subtract borders)
    let visible_height = area.height.saturating_sub(2) as usize;
    
    // Adjust scroll offset
    app.adjust_scroll(visible_height);
    
    let items: Vec<ListItem> = app
        .browser_items
        .iter()
        .enumerate()
        .skip(app.browser_scroll_offset)
        .take(visible_height)
        .map(|(i, item)| {
            let (icon, name, indent) = match item {
                BrowserItem::Schema(name) => ("📁", name.as_str(), 0),
                BrowserItem::Table(_, name) => ("📊", name.as_str(), 2),
            };

            let indent_str = " ".repeat(indent);
            let content = format!("{}{} {}", indent_str, icon, name);
            
            let style = if i == app.browser_selected {
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
            .title(format!("Database Browser ({}/{})", app.browser_selected + 1, app.browser_items.len()))
            .border_style(Style::default().fg(Color::Cyan)),
    );

    f.render_widget(list, area);
}

pub fn render_details(f: &mut Frame, app: &App, area: Rect) {
    if app.columns.is_empty() {
        let help = Paragraph::new("Select a table to view its structure\n\nKeyboard shortcuts:\n  ↑/↓ - Navigate\n  Enter - Expand/View\n  Tab - Switch to query mode\n  r - Refresh\n  q - Quit")
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .title("Details")
                    .border_style(Style::default().fg(Color::Cyan)),
            );
        f.render_widget(help, area);
        return;
    }

    // Render table structure
    let header = Row::new(vec!["Column", "Type", "Nullable", "Default"])
        .style(Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD))
        .bottom_margin(1);

    let rows: Vec<Row> = app
        .columns
        .iter()
        .map(|col| {
            Row::new(vec![
                col.name.clone(),
                col.data_type.clone(),
                col.is_nullable.clone(),
                col.column_default.clone().unwrap_or_else(|| "-".to_string()),
            ])
        })
        .collect();

    let table = Table::new(
        rows,
        [
            ratatui::layout::Constraint::Percentage(25),
            ratatui::layout::Constraint::Percentage(25),
            ratatui::layout::Constraint::Percentage(15),
            ratatui::layout::Constraint::Percentage(35),
        ],
    )
    .header(header)
    .block(
        Block::default()
            .borders(Borders::ALL)
            .title("Table Structure")
            .border_style(Style::default().fg(Color::Cyan)),
    );

    f.render_widget(table, area);
}
