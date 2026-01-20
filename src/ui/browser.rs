use ratatui::{
    layout::Rect,
    style::{Color, Modifier, Style},
    widgets::{Block, Borders, List, ListItem, Paragraph, Row, Table},
    Frame,
};

use crate::app::{App, BrowserItem};

pub fn render_browser(f: &mut Frame, app: &mut App, area: Rect) {
    use ratatui::layout::{Constraint, Direction, Layout};
    
    // Split area into filter input and browser list
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3), // Filter input
            Constraint::Min(0),    // Browser list
        ])
        .split(area);
    
    // Render filter input box
    let filter_text = if app.filter_active {
        format!(" Filter: {}_", app.filter_input)
    } else {
        " Press '/' to filter".to_string()
    };
    
    let filter_style = if app.filter_active {
        Style::default().fg(Color::Yellow)
    } else {
        Style::default().fg(Color::DarkGray)
    };
    
    let filter_widget = Paragraph::new(filter_text)
        .style(filter_style)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(if app.filter_active { Color::Yellow } else { Color::Cyan }))
        );
    
    f.render_widget(filter_widget, chunks[0]);
    
    // Get filtered items
    let filtered_indices = app.get_filtered_items();
    let visible_height = chunks[1].height.saturating_sub(2) as usize;
    
    // Adjust scroll offset for filtered view
    let filtered_selected = filtered_indices.iter().position(|&idx| idx == app.browser_selected).unwrap_or(0);
    let scroll_offset = if filtered_selected >= visible_height {
        filtered_selected.saturating_sub(visible_height - 1)
    } else {
        0
    };
    
    // Build list items from filtered results
    let items: Vec<ListItem> = filtered_indices
        .iter()
        .skip(scroll_offset)
        .take(visible_height)
        .map(|&idx| {
            let item = &app.browser_items[idx];
            let (icon, name, indent) = match item {
                BrowserItem::Schema(name) => ("📁", name.as_str(), 0),
                BrowserItem::Table(_, name) => ("📊", name.as_str(), 2),
            };

            let indent_str = " ".repeat(indent);
            let content = format!("{}{} {}", indent_str, icon, name);
            
            let style = if idx == app.browser_selected {
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default().fg(Color::White)
            };

            ListItem::new(content).style(style)
        })
        .collect();
    
    let title = if app.filter_active && !app.filter_input.is_empty() {
        format!("Database Browser ({} filtered / {} total)", filtered_indices.len(), app.browser_items.len())
    } else {
        format!("Database Browser ({}/{})", app.browser_selected + 1, app.browser_items.len())
    };

    let list = List::new(items).block(
        Block::default()
            .borders(Borders::ALL)
            .title(title)
            .border_style(Style::default().fg(Color::Cyan)),
    );

    f.render_widget(list, chunks[1]);
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
