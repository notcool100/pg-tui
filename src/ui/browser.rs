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
                BrowserItem::Folder(_, folder_type) => {
                    use crate::app::FolderType;
                    let folder_name = match folder_type {
                        FolderType::Tables => "Tables",
                        FolderType::Views => "Views",
                        FolderType::Functions => "Functions",
                    };
                    ("📂", folder_name, 2)
                }
                BrowserItem::Table(_, name) => ("📊", name.as_str(), 4),
                BrowserItem::View(_, name) => ("👁️", name.as_str(), 4),
                BrowserItem::Function(_, name) => ("⚙️", name.as_str(), 4),
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
    use ratatui::layout::{Constraint, Direction, Layout};

    if app.selected_table.is_none() {
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

    // Split area for tab bar and content
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3), // Tab bar
            Constraint::Min(0),    // Content
        ])
        .split(area);

    // Render tab bar
    let tabs = vec!["Columns", "Constraints", "Indexes", "Triggers", "Foreign Keys"];
    let active_tab_index = match app.table_detail_tab {
        crate::app::TableDetailTab::Columns => 0,
        crate::app::TableDetailTab::Constraints => 1,
        crate::app::TableDetailTab::Indexes => 2,
        crate::app::TableDetailTab::Triggers => 3,
        crate::app::TableDetailTab::ForeignKeys => 4,
    };

    let tab_titles: Vec<String> = tabs
        .iter()
        .enumerate()
        .map(|(i, name)| {
            if i == active_tab_index {
                format!(" [{}] ", name)
            } else {
                format!("  {}  ", name)
            }
        })
        .collect();

    let tab_text = tab_titles.join("|");
    let tab_widget = Paragraph::new(tab_text)
        .style(Style::default().fg(Color::Cyan))
        .block(Block::default().borders(Borders::ALL));
    
    f.render_widget(tab_widget, chunks[0]);

    // Render content based on active tab
    match app.table_detail_tab {
        crate::app::TableDetailTab::Columns => render_columns_tab(f, app, chunks[1]),
        crate::app::TableDetailTab::Constraints => render_constraints_tab(f, app, chunks[1]),
        crate::app::TableDetailTab::Indexes => render_indexes_tab(f, app, chunks[1]),
        crate::app::TableDetailTab::Triggers => render_triggers_tab(f, app, chunks[1]),
        crate::app::TableDetailTab::ForeignKeys => render_foreign_keys_tab(f, app, chunks[1]),
    }
}

fn render_columns_tab(f: &mut Frame, app: &App, area: Rect) {
    if app.columns.is_empty() {
        let empty = Paragraph::new("No columns found")
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .title("Columns")
                    .border_style(Style::default().fg(Color::Cyan)),
            );
        f.render_widget(empty, area);
        return;
    }

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
            .title("Columns")
            .border_style(Style::default().fg(Color::Cyan)),
    );

    f.render_widget(table, area);
}

fn render_constraints_tab(f: &mut Frame, app: &App, area: Rect) {
    if app.constraints.is_empty() {
        let empty = Paragraph::new("No constraints defined")
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .title("Constraints")
                    .border_style(Style::default().fg(Color::Cyan)),
            );
        f.render_widget(empty, area);
        return;
    }

    let header = Row::new(vec!["Name", "Type", "Columns"])
        .style(Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD))
        .bottom_margin(1);

    let rows: Vec<Row> = app
        .constraints
        .iter()
        .map(|con| {
            Row::new(vec![
                con.name.clone(),
                con.constraint_type.clone(),
                con.column_names.clone(),
            ])
        })
        .collect();

    let table = Table::new(
        rows,
        [
            ratatui::layout::Constraint::Percentage(30),
            ratatui::layout::Constraint::Percentage(25),
            ratatui::layout::Constraint::Percentage(45),
        ],
    )
    .header(header)
    .block(
        Block::default()
            .borders(Borders::ALL)
            .title("Constraints")
            .border_style(Style::default().fg(Color::Cyan)),
    );

    f.render_widget(table, area);
}

fn render_indexes_tab(f: &mut Frame, app: &App, area: Rect) {
    if app.indexes.is_empty() {
        let empty = Paragraph::new("No indexes defined")
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .title("Indexes")
                    .border_style(Style::default().fg(Color::Cyan)),
            );
        f.render_widget(empty, area);
        return;
    }

    let header = Row::new(vec!["Name", "Columns", "Unique", "Primary"])
        .style(Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD))
        .bottom_margin(1);

    let rows: Vec<Row> = app
        .indexes
        .iter()
        .map(|idx| {
            Row::new(vec![
                idx.name.clone(),
                idx.columns.clone(),
                if idx.is_unique { "Yes" } else { "No" }.to_string(),
                if idx.is_primary { "Yes" } else { "No" }.to_string(),
            ])
        })
        .collect();

    let table = Table::new(
        rows,
        [
            ratatui::layout::Constraint::Percentage(35),
            ratatui::layout::Constraint::Percentage(35),
            ratatui::layout::Constraint::Percentage(15),
            ratatui::layout::Constraint::Percentage(15),
        ],
    )
    .header(header)
    .block(
        Block::default()
            .borders(Borders::ALL)
            .title("Indexes")
            .border_style(Style::default().fg(Color::Cyan)),
    );

    f.render_widget(table, area);
}

fn render_triggers_tab(f: &mut Frame, app: &App, area: Rect) {
    if app.triggers.is_empty() {
        let empty = Paragraph::new("No triggers defined")
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .title("Triggers")
                    .border_style(Style::default().fg(Color::Cyan)),
            );
        f.render_widget(empty, area);
        return;
    }

    let header = Row::new(vec!["Name", "Event", "Timing", "Action"])
        .style(Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD))
        .bottom_margin(1);

    let rows: Vec<Row> = app
        .triggers
        .iter()
        .map(|trg| {
            Row::new(vec![
                trg.name.clone(),
                trg.event.clone(),
                trg.timing.clone(),
                trg.action_statement.clone(),
            ])
        })
        .collect();

    let table = Table::new(
        rows,
        [
            ratatui::layout::Constraint::Percentage(20),
            ratatui::layout::Constraint::Percentage(20),
            ratatui::layout::Constraint::Percentage(15),
            ratatui::layout::Constraint::Percentage(45),
        ],
    )
    .header(header)
    .block(
        Block::default()
            .borders(Borders::ALL)
            .title("Triggers")
            .border_style(Style::default().fg(Color::Cyan)),
    );

    f.render_widget(table, area);
}

fn render_foreign_keys_tab(f: &mut Frame, app: &App, area: Rect) {
    if app.foreign_keys.is_empty() {
        let empty = Paragraph::new("No foreign keys defined")
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .title("Foreign Keys")
                    .border_style(Style::default().fg(Color::Cyan)),
            );
        f.render_widget(empty, area);
        return;
    }

    let header = Row::new(vec!["Name", "Columns", "References Table", "References Columns"])
        .style(Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD))
        .bottom_margin(1);

    let rows: Vec<Row> = app
        .foreign_keys
        .iter()
        .map(|fk| {
            Row::new(vec![
                fk.name.clone(),
                fk.column_names.clone(),
                fk.referenced_table.clone(),
                fk.referenced_columns.clone(),
            ])
        })
        .collect();

    let table = Table::new(
        rows,
        [
            ratatui::layout::Constraint::Percentage(25),
            ratatui::layout::Constraint::Percentage(25),
            ratatui::layout::Constraint::Percentage(25),
            ratatui::layout::Constraint::Percentage(25),
        ],
    )
    .header(header)
    .block(
        Block::default()
            .borders(Borders::ALL)
            .title("Foreign Keys")
            .border_style(Style::default().fg(Color::Cyan)),
    );

    f.render_widget(table, area);
}
