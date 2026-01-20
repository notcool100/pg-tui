use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    widgets::{Block, Borders, Paragraph, Row, Table, Wrap},
    Frame,
};

use crate::app::App;

pub fn render_query(f: &mut Frame, app: &App, area: Rect) {
    // Only show results panel if there are actual results
    if app.query_result.is_some() {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Length(10), Constraint::Min(0)])
            .split(area);

        // Query editor
        render_query_editor(f, app, chunks[0]);

        // Results
        render_query_results(f, app, chunks[1]);
    } else {
        // No results yet - give full space to editor
        render_query_editor(f, app, area);
    }
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
        // Insert cursor at the current position
        let mut display_text = app.query_input.clone();
        // Ensure cursor position is valid
        let cursor_pos = app.query_cursor.min(display_text.len());
        // Insert a visible cursor character at the cursor position
        display_text.insert(cursor_pos, '█');
        
        // Split into lines for scrolling
        let lines: Vec<&str> = display_text.split('\n').collect();
        let total_lines = lines.len();
        
        // Calculate visible area (subtract 2 for borders)
        let visible_lines = (area.height.saturating_sub(2)) as usize;
        
        // Get visible lines based on scroll offset
        let start = app.query_scroll_offset;
        let end = (start + visible_lines).min(total_lines);
        let visible_text: Vec<&str> = lines[start..end].iter().copied().collect();
        
        // Add scroll indicators
        let mut result = visible_text.join("\n");
        if start > 0 {
            result = format!("▲ (scroll: line {}/{})\n{}", start + 1, total_lines, result);
        }
        if end < total_lines {
            result = format!("{}\n▼ (more below)", result);
        }
        
        result
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

        // Split area for filter input if active
        let (filter_area, table_area) = if app.results_filter_active {
            let chunks = Layout::default()
                .direction(Direction::Vertical)
                .constraints([Constraint::Length(3), Constraint::Min(0)])
                .split(area);
            (Some(chunks[0]), chunks[1])
        } else {
            (None, area)
        };

        // Render filter input if active
        if let Some(filter_area) = filter_area {
            let filter_text = if app.results_filter_input.is_empty() {
                "Type to filter rows... (ESC to clear)".to_string()
            } else {
                app.results_filter_input.clone()
            };
            
            let filter_widget = Paragraph::new(filter_text)
                .style(Style::default().fg(Color::Yellow))
                .block(
                    Block::default()
                        .borders(Borders::ALL)
                        .title("Filter")
                        .border_style(Style::default().fg(Color::Yellow)),
                );
            f.render_widget(filter_widget, filter_area);
        }

        // Get filtered row indices if filtering is active
        let filtered_indices = app.get_filtered_rows();
        let rows_to_display: Vec<&Vec<String>> = if let Some(indices) = &filtered_indices {
            indices.iter().map(|&idx| &result.rows[idx]).collect()
        } else {
            result.rows.iter().collect()
        };

        // Calculate optimal column widths based on content
        let mut col_widths: Vec<usize> = Vec::new();
        for (col_idx, col_name) in result.columns.iter().enumerate() {
            let mut max_width = col_name.len();
            // Check first 10 displayed rows to determine width
            for row in rows_to_display.iter().take(10) {
                if let Some(cell) = row.get(col_idx) {
                    max_width = max_width.max(cell.len());
                }
            }
            // Limit individual column width to 30 characters
            col_widths.push(max_width.min(30));
        }
        
        // Calculate visible columns based on scroll offset and available width
        let available_width = table_area.width.saturating_sub(4) as usize; // subtract borders and padding
        let mut visible_cols: Vec<usize> = Vec::new();
        let mut used_width = 0;
        let scroll_offset = app.result_scroll_offset;
        
        // Start from scroll offset and add columns until width is full
        for col_idx in scroll_offset..result.columns.len() {
            let col_width = col_widths[col_idx] + 3; // Add padding
            if used_width + col_width <= available_width || visible_cols.is_empty() {
                visible_cols.push(col_idx);
                used_width += col_width;
            } else {
                break;
            }
        }
        
        // Build title with scroll indicators and filter info
        let total_cols = result.columns.len();
        let displayed_rows = rows_to_display.len();
        let total_rows = result.row_count;
        
        let filter_info = if filtered_indices.is_some() {
            format!(" [filtered: {}/{}]", displayed_rows, total_rows)
        } else {
            format!(" ({} rows)", total_rows)
        };
        
        let title = if scroll_offset > 0 && scroll_offset + visible_cols.len() < total_cols {
            format!("Results{} ◄ cols {}-{}/{} ►", 
                filter_info,
                scroll_offset + 1, 
                scroll_offset + visible_cols.len(),
                total_cols)
        } else if scroll_offset > 0 {
            format!("Results{} ◄ cols {}-{}/{}", 
                filter_info,
                scroll_offset + 1, 
                total_cols,
                total_cols)
        } else if scroll_offset + visible_cols.len() < total_cols {
            format!("Results{} cols 1-{}/{} ►", 
                filter_info,
                visible_cols.len(),
                total_cols)
        } else {
            format!("Results{}", filter_info)
        };
        
        // Create header with only visible columns
        let header_cells: Vec<String> = visible_cols.iter()
            .map(|&idx| result.columns[idx].clone())
            .collect();
        let header = Row::new(header_cells)
            .style(Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD))
            .bottom_margin(1);

        // Create table rows with only visible columns from filtered rows
        let rows: Vec<Row> = rows_to_display
            .iter()
            .map(|row| {
                let cells: Vec<String> = visible_cols.iter()
                    .map(|&idx| row.get(idx).cloned().unwrap_or_else(|| "".to_string()))
                    .collect();
                Row::new(cells)
            })
            .collect();

        // Calculate constraints for visible columns
        let constraints: Vec<Constraint> = visible_cols.iter()
            .map(|&idx| {
                let width = col_widths[idx];
                Constraint::Length(width as u16 + 3)
            })
            .collect();

        let table = Table::new(rows, constraints)
            .header(header)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .title(title)
                    .border_style(Style::default().fg(Color::Cyan)),
            );

        f.render_widget(table, table_area);
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
