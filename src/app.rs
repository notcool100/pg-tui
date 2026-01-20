use anyhow::Result;
use crossterm::event::KeyCode;
use std::collections::HashSet;

use crate::autocomplete::{AutocompleteEngine, Suggestion};
use crate::db::{Column, Constraint, DbConnection, ForeignKey, Index, QueryResult, Schema, Table, Trigger};

mod connection_selector;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AppMode {
    ConnectionSelector,
    ConnectionEdit,
    Browser,
    Query,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConnectionField {
    Host,
    Port,
    Database,
    User,
    Password,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FolderType {
    Tables,
    Views,
    Functions,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TableDetailTab {
    Columns,
    Constraints,
    Indexes,
    Triggers,
    ForeignKeys,
}

#[derive(Debug, Clone)]
pub enum BrowserItem {
    Schema(String),
    Folder(String, FolderType), // schema, folder_type
    Table(String, String),      // schema, table_name
    View(String, String),       // schema, view_name
    Function(String, String),   // schema, function_name
}

pub struct App {
    pub mode: AppMode,
    pub connection_field: ConnectionField,
    
    // Connection selector
    pub config: crate::config::Config,
    pub selected_profile: usize,
    pub editing_profile_name: bool,
    pub new_profile_name: String,
    
    // Connection fields
    pub host: String,
    pub port: String,
    pub database: String,
    pub user: String,
    pub password: String,
    
    // Database connection
    pub db: DbConnection,
    
    // Browser state
    pub schemas: Vec<Schema>,
    pub tables: Vec<Table>,
    pub columns: Vec<Column>,
    pub browser_items: Vec<BrowserItem>,
    pub browser_selected: usize,
    pub browser_scroll_offset: usize,
    
    // Table details tab state
    pub table_detail_tab: TableDetailTab,
    pub selected_table: Option<(String, String)>, // (schema, table_name)
    pub constraints: Vec<Constraint>,
    pub indexes: Vec<Index>,
    pub triggers: Vec<Trigger>,
    pub foreign_keys: Vec<ForeignKey>,
    
    // Query state
    pub query_input: String,
    pub query_result: Option<QueryResult>,
    pub query_cursor: usize,
    pub query_scroll_offset: usize,
    pub result_scroll_offset: usize,
    
    // UI state
    pub error_message: Option<String>,
    
    // Filter state (browser)
    pub filter_input: String,
    pub filter_active: bool,
    
    // Filter state (results)
    pub results_filter_input: String,
    pub results_filter_active: bool,
    
    // Expanded items tracking
    pub expanded_items: HashSet<String>,
    
    // Autocomplete
    pub autocomplete_engine: AutocompleteEngine,
    pub suggestions: Vec<Suggestion>,
    pub suggestion_selected: usize,
    pub show_autocomplete: bool,
    pub autocomplete_schema_loaded: bool,
}

impl App {
    pub fn new() -> Self {
        // Load saved config
        let config = crate::config::Config::load().unwrap_or_default();

        Self {
            mode: AppMode::ConnectionSelector,
            connection_field: ConnectionField::Host,
            config,
            selected_profile: 0,
            editing_profile_name: false,
            new_profile_name: String::new(),
            host: "localhost".to_string(),
            port: "5432".to_string(),
            database: "postgres".to_string(),
            user: "postgres".to_string(),
            password: String::new(),
            db: DbConnection::new(),
            schemas: Vec::new(),
            tables: Vec::new(),
            columns: Vec::new(),
            browser_items: Vec::new(),
            browser_selected: 0,
            browser_scroll_offset: 0,
            table_detail_tab: TableDetailTab::Columns,
            selected_table: None,
            constraints: Vec::new(),
            indexes: Vec::new(),
            triggers: Vec::new(),
            foreign_keys: Vec::new(),
            query_input: String::new(),
            query_result: None,
            query_cursor: 0,
            query_scroll_offset: 0,
            result_scroll_offset: 0,
            error_message: None,
            filter_input: String::new(),
            filter_active: false,
            results_filter_input: String::new(),
            results_filter_active: false,
            expanded_items: HashSet::new(),
            autocomplete_engine: AutocompleteEngine::new(),
            suggestions: Vec::new(),
            suggestion_selected: 0,
            show_autocomplete: false,
            autocomplete_schema_loaded: false,
        }
    }

    pub fn set_error(&mut self, msg: String) {
        self.error_message = Some(msg);
    }

    pub fn clear_error(&mut self) {
        self.error_message = None;
    }

    // Connection field navigation
    pub fn next_connection_field(&mut self) {
        self.connection_field = match self.connection_field {
            ConnectionField::Host => ConnectionField::Port,
            ConnectionField::Port => ConnectionField::Database,
            ConnectionField::Database => ConnectionField::User,
            ConnectionField::User => ConnectionField::Password,
            ConnectionField::Password => ConnectionField::Host,
        };
    }

    pub fn prev_connection_field(&mut self) {
        self.connection_field = match self.connection_field {
            ConnectionField::Host => ConnectionField::Password,
            ConnectionField::Port => ConnectionField::Host,
            ConnectionField::Database => ConnectionField::Port,
            ConnectionField::User => ConnectionField::Database,
            ConnectionField::Password => ConnectionField::User,
        };
    }

    pub fn input_char(&mut self, c: char) {
        let field = match self.connection_field {
            ConnectionField::Host => &mut self.host,
            ConnectionField::Port => &mut self.port,
            ConnectionField::Database => &mut self.database,
            ConnectionField::User => &mut self.user,
            ConnectionField::Password => &mut self.password,
        };
        field.push(c);
    }

    pub fn delete_char(&mut self) {
        let field = match self.connection_field {
            ConnectionField::Host => &mut self.host,
            ConnectionField::Port => &mut self.port,
            ConnectionField::Database => &mut self.database,
            ConnectionField::User => &mut self.user,
            ConnectionField::Password => &mut self.password,
        };
        field.pop();
    }

    // Database connection
    pub async fn connect(&mut self) -> Result<()> {
        let port: u16 = self.port.parse()?;
        self.db
            .connect(&self.host, port, &self.database, &self.user, &self.password)
            .await?;
        
        // Save/update connection profile
        let profile = crate::config::ConnectionProfile {
            name: format!("{}@{}", self.user, self.host),
            host: self.host.clone(),
            port: self.port.clone(),
            database: self.database.clone(),
            user: self.user.clone(),
        };
        
        // Check if this profile already exists
        let existing = self.config.connections.iter().position(|p| {
            p.host == profile.host && p.port == profile.port && 
            p.database == profile.database && p.user == profile.user
        });
        
        if existing.is_none() {
            self.config.connections.push(profile);
            if let Err(e) = self.config.save() {
                eprintln!("Warning: Could not save connection config: {}", e);
            }
        }
        
        // Load initial data
        self.mode = AppMode::Browser;
        self.refresh_browser().await?;
        Ok(())
    }

    pub async fn refresh_browser(&mut self) -> Result<()> {
        if let Some(client) = self.db.client() {
            self.schemas = crate::db::list_schemas(client, &self.database).await?;
            self.browser_items = self
                .schemas
                .iter()
                .map(|s| BrowserItem::Schema(s.name.clone()))
                .collect();
        }
        Ok(())
    }

    // Browser navigation
    pub fn browser_up(&mut self) {
        if self.browser_selected > 0 {
            self.browser_selected -= 1;
            // Adjust scroll offset if needed
            if self.browser_selected < self.browser_scroll_offset {
                self.browser_scroll_offset = self.browser_selected;
            }
        }
    }

    pub fn browser_down(&mut self) {
        if self.browser_selected < self.browser_items.len().saturating_sub(1) {
            self.browser_selected += 1;
        }
    }

    pub fn adjust_scroll(&mut self, visible_height: usize) {
        // Ensure selected item is visible
        if self.browser_selected >= self.browser_scroll_offset + visible_height {
            self.browser_scroll_offset = self.browser_selected - visible_height + 1;
        } else if self.browser_selected < self.browser_scroll_offset {
            self.browser_scroll_offset = self.browser_selected;
        }
    }

    pub async fn browser_select(&mut self) -> Result<()> {
        if self.browser_selected >= self.browser_items.len() {
            return Ok(());
        }

        if let Some(client) = self.db.client() {
            match &self.browser_items[self.browser_selected].clone() {
                BrowserItem::Schema(schema) => {
                    let key = format!("schema:{}", schema);
                    
                    if self.expanded_items.contains(&key) {
                        // COLLAPSE: Remove the 3 folders and their contents
                        self.collapse_schema(&key);
                    } else {                        // EXPAND: Insert folders after the schema
                        let insert_pos = self.browser_selected + 1;
                        self.browser_items.insert(
                            insert_pos,
                            BrowserItem::Folder(schema.clone(), FolderType::Tables),
                        );
                        self.browser_items.insert(
                            insert_pos + 1,
                            BrowserItem::Folder(schema.clone(), FolderType::Views),
                        );
                        self.browser_items.insert(
                            insert_pos + 2,
                            BrowserItem::Folder(schema.clone(), FolderType::Functions),
                        );
                        self.expanded_items.insert(key);
                    }
                }
                BrowserItem::Folder(schema, folder_type) => {
                    let key = format!("folder:{}:{:?}", schema, folder_type);
                    
                    if self.expanded_items.contains(&key) {
                        // COLLAPSE: Remove child items
                        self.collapse_folder(&key);
                    } else {
                        // EXPAND: Load and insert items
                        let insert_pos = self.browser_selected + 1;
                        
                        match folder_type {
                            FolderType::Tables => {
                                // Load and insert tables
                                self.tables = crate::db::list_tables(client, schema).await?;
                                for (i, table) in self.tables.iter().enumerate() {
                                    self.browser_items.insert(
                                        insert_pos + i,
                                        BrowserItem::Table(schema.clone(), table.name.clone()),
                                    );
                                }
                            }
                            FolderType::Views => {
                                let views = crate::db::list_views(client, schema).await?;
                                for (i, view) in views.iter().enumerate() {
                                    self.browser_items.insert(
                                        insert_pos + i,
                                        BrowserItem::View(schema.clone(), view.name.clone()),
                                    );
                                }
                            }
                            FolderType::Functions => {
                                // Load and insert functions
                                let functions = crate::db::list_functions(client, schema).await?;
                                for (i, func) in functions.iter().enumerate() {
                                    self.browser_items.insert(
                                        insert_pos + i,
                                        BrowserItem::Function(schema.clone(), func.name.clone()),
                                    );
                                }
                            }
                        }
                        self.expanded_items.insert(key);
                    }
                }
                BrowserItem::Table(schema, table) => {
                    self.selected_table = Some((schema.clone(), table.clone()));
                    self.table_detail_tab = TableDetailTab::Columns;
                    self.columns = crate::db::describe_table(client, schema, table).await?;
                    self.constraints = crate::db::list_table_constraints(client, schema, table).await?;
                    self.indexes = crate::db::list_table_indexes(client, schema, table).await?;
                    self.triggers = crate::db::list_table_triggers(client, schema, table).await?;
                    self.foreign_keys = crate::db::list_table_foreign_keys(client, schema, table).await?;
                }
                BrowserItem::View(schema, view) => {
                    self.selected_table = Some((schema.clone(), view.clone()));
                    self.table_detail_tab = TableDetailTab::Columns;
                    self.columns = crate::db::describe_table(client, schema, view).await?;
                    // Views don't have constraints, indexes, triggers, or foreign keys
                    self.constraints.clear();
                    self.indexes.clear();
                    self.triggers.clear();
                    self.foreign_keys.clear();
                }
                BrowserItem::Function(_schema, _function) => {
                    self.selected_table = None;
                    // For now, just show a message that function details aren't implemented yet
                    self.columns.clear();
                    self.constraints.clear();
                    self.indexes.clear();
                    self.triggers.clear();
                    self.foreign_keys.clear();
                }
            }
        }

        Ok(())
    }

    fn collapse_schema(&mut self, key: &str) {
        // Find how many items to remove (3 folders + their children)
        let mut remove_count = 0;
        let start_pos = self.browser_selected + 1;
        
        // Count folders (should be 3) and their children
        let mut i = start_pos;
        let mut folders_found = 0;
        
        while i < self.browser_items.len() && folders_found < 3 {
            match &self.browser_items[i] {
                BrowserItem::Folder(schema, folder_type) => {
                    // Remove this folder from expanded set
                    let folder_key = format!("folder:{}:{:?}", schema, folder_type);
                    self.expanded_items.remove(&folder_key);
                    remove_count += 1;
                    i += 1;
                    folders_found += 1;
                    
                    // Count children of this folder
                    while i < self.browser_items.len() {
                        match &self.browser_items[i] {
                            BrowserItem::Table(_, _) | BrowserItem::View(_, _) | BrowserItem::Function(_, _) => {
                                remove_count += 1;
                                i += 1;
                            }
                            _ => break,
                        }
                    }
                }
                _ => break,
            }
        }
        
        // Remove all items
        for _ in 0..remove_count {
            if start_pos < self.browser_items.len() {
                self.browser_items.remove(start_pos);
            }
        }
        
        // Adjust selection if it was on a removed item
        if self.browser_selected >= start_pos && self.browser_selected < start_pos + remove_count {
            self.browser_selected = start_pos - 1; // Move to the schema itself
        } else if self.browser_selected >= start_pos + remove_count {
            self.browser_selected -= remove_count;
        }
        
        self.expanded_items.remove(key);
    }

    fn collapse_folder(&mut self, key: &str) {
        // Find how many child items to remove
        let mut remove_count = 0;
        let start_pos = self.browser_selected + 1;
        
        // Count children
        let mut i = start_pos;
        while i < self.browser_items.len() {
            match &self.browser_items[i] {
                BrowserItem::Table(_, _) | BrowserItem::View(_, _) | BrowserItem::Function(_, _) => {
                    remove_count += 1;
                    i += 1;
                }
                _ => break,
            }
        }
        
        // Remove all child items
        for _ in 0..remove_count {
            if start_pos < self.browser_items.len() {
                self.browser_items.remove(start_pos);
            }
        }
        
        // Adjust selection if it was on a removed item
        if self.browser_selected >= start_pos && self.browser_selected < start_pos + remove_count {
            self.browser_selected = start_pos - 1; // Move to the folder itself
        } else if self.browser_selected >= start_pos + remove_count {
            self.browser_selected -= remove_count;
        }
        
        self.expanded_items.remove(key);
    }

    // Query handling
    pub fn handle_query_input(&mut self, key: KeyCode) {
        match key {
            KeyCode::Char(c) => {
                self.query_input.insert(self.query_cursor, c);
                self.query_cursor += 1;
            }
            KeyCode::Backspace => {
                if self.query_cursor > 0 {
                    self.query_input.remove(self.query_cursor - 1);
                    self.query_cursor -= 1;
                }
            }
            KeyCode::Left => {
                if self.query_cursor > 0 {
                    self.query_cursor -= 1;
                }
            }
            KeyCode::Right => {
                if self.query_cursor < self.query_input.len() {
                    self.query_cursor += 1;
                }
            }
            KeyCode::Enter => {
                self.query_input.push('\n');
                self.query_cursor += 1;
            }
            _ => {}
        }
    }

    pub fn adjust_query_scroll(&mut self, visible_lines: usize) {
        // Calculate which line the cursor is on
        let text_before_cursor = &self.query_input[..self.query_cursor.min(self.query_input.len())];
        let cursor_line = text_before_cursor.matches('\n').count();
        
        // Adjust scroll to keep cursor visible
        if cursor_line < self.query_scroll_offset {
            // Cursor is above visible area, scroll up
            self.query_scroll_offset = cursor_line;
        } else if cursor_line >= self.query_scroll_offset + visible_lines {
            // Cursor is below visible area, scroll down
            self.query_scroll_offset = cursor_line - visible_lines + 1;
        }
    }

    pub fn scroll_results_left(&mut self) {
        if self.result_scroll_offset > 0 {
            self.result_scroll_offset -= 1;
        }
    }

    pub fn scroll_results_right(&mut self) {
        if let Some(result) = &self.query_result {
            if self.result_scroll_offset < result.columns.len().saturating_sub(1) {
                self.result_scroll_offset += 1;
            }
        }
    }

    pub async fn execute_query(&mut self) -> Result<()> {
        if let Some(client) = self.db.client() {
            // Extract the query at cursor position (DBeaver-like behavior)
            let sql = self.extract_current_query();
            
            if !sql.trim().is_empty() {
                match crate::db::execute_query(client, &sql).await {
                    Ok(result) => {
                        self.query_result = Some(result);
                        self.clear_error();
                    }
                    Err(e) => {
                        self.set_error(format!("Query error: {}", e));
                    }
                }
            }
        }
        Ok(())
    }
    
    fn extract_current_query(&self) -> String {
        // If input is empty, return empty
        if self.query_input.is_empty() {
            return String::new();
        }
        
        // Find all semicolon positions
        let semicolons: Vec<usize> = self.query_input
            .char_indices()
            .filter_map(|(i, c)| if c == ';' { Some(i) } else { None })
            .collect();
        
        // If no semicolons, return the entire input
        if semicolons.is_empty() {
            return self.query_input.trim().to_string();
        }
        
        // Find which query the cursor is in
        let cursor_pos = self.query_cursor;
        
        // Find the start of current query (after previous semicolon or beginning)
        let query_start = semicolons
            .iter()
            .rev()
            .find(|&&pos| pos < cursor_pos)
            .map(|&pos| pos + 1) // Start after the semicolon
            .unwrap_or(0); // Or from the beginning
        
        // Find the end of current query (at next semicolon or end)
        let query_end = semicolons
            .iter()
            .find(|&&pos| pos >= cursor_pos)
            .copied()
            .unwrap_or(self.query_input.len()); // Or to the end
        
        // Extract the query
        let query = &self.query_input[query_start..query_end];
        query.trim().to_string()
    }

    // Results filter methods
    pub fn activate_results_filter(&mut self) {
        self.results_filter_active = true;
    }

    pub fn clear_results_filter(&mut self) {
        self.results_filter_input.clear();
        self.results_filter_active = false;
    }

    pub fn handle_results_filter_input(&mut self, key: KeyCode) {
        match key {
            KeyCode::Char(c) => {
                self.results_filter_input.push(c);
            }
            KeyCode::Backspace => {
                self.results_filter_input.pop();
            }
            _ => {}
        }
    }

    pub fn get_filtered_rows(&self) -> Option<Vec<usize>> {
        if !self.results_filter_active || self.results_filter_input.is_empty() {
            return None;
        }

        if let Some(result) = &self.query_result {
            let filter_lower = self.results_filter_input.to_lowercase();
            let mut filtered_indices = Vec::new();

            for (row_idx, row) in result.rows.iter().enumerate() {
                // Check if any cell in the row contains the filter text
                let matches = row.iter().any(|cell| {
                    cell.to_lowercase().contains(&filter_lower)
                });

                if matches {
                    filtered_indices.push(row_idx);
                }
            }

            Some(filtered_indices)
        } else {
            None
        }
    }

    // Filter methods
    pub fn activate_filter(&mut self) {
        self.filter_active = true;
    }

    pub fn clear_filter(&mut self) {
        self.filter_input.clear();
        self.filter_active = false;
    }

    pub fn handle_filter_input(&mut self, key: KeyCode) {
        match key {
            KeyCode::Char(c) => {
                self.filter_input.push(c);
            }
            KeyCode::Backspace => {
                self.filter_input.pop();
            }
            _ => {}
        }
    }

    pub fn get_filtered_items(&self) -> Vec<usize> {
        if !self.filter_active || self.filter_input.is_empty() {
            return (0..self.browser_items.len()).collect();
        }

        let filter_lower = self.filter_input.to_lowercase();
        let mut filtered = Vec::new();

        for (idx, item) in self.browser_items.iter().enumerate() {
            let matches = match item {
                BrowserItem::Schema(name) => {
                    name.to_lowercase().contains(&filter_lower)
                }
                BrowserItem::Folder(_, _) => {
                    false
                }
                BrowserItem::Table(schema, name) => {
                    name.to_lowercase().contains(&filter_lower)
                        || schema.to_lowercase().contains(&filter_lower)
                }
                BrowserItem::View(schema, name) => {
                    name.to_lowercase().contains(&filter_lower)
                        || schema.to_lowercase().contains(&filter_lower)
                }
                BrowserItem::Function(schema, name) => {
                    name.to_lowercase().contains(&filter_lower)
                        || schema.to_lowercase().contains(&filter_lower)
                }
            };

            if matches {
                filtered.push(idx);
            }
        }

        filtered
    }

    // Tab navigation
    pub fn next_tab(&mut self) {
        self.table_detail_tab = match self.table_detail_tab {
            TableDetailTab::Columns => TableDetailTab::Constraints,
            TableDetailTab::Constraints => TableDetailTab::Indexes,
            TableDetailTab::Indexes => TableDetailTab::Triggers,
            TableDetailTab::Triggers => TableDetailTab::ForeignKeys,
            TableDetailTab::ForeignKeys => TableDetailTab::Columns,
        };
    }

    pub fn prev_tab(&mut self) {
        self.table_detail_tab = match self.table_detail_tab {
            TableDetailTab::Columns => TableDetailTab::ForeignKeys,
            TableDetailTab::Constraints => TableDetailTab::Columns,
            TableDetailTab::Indexes => TableDetailTab::Constraints,
            TableDetailTab::Triggers => TableDetailTab::Indexes,
            TableDetailTab::ForeignKeys => TableDetailTab::Triggers,
        };
    }
    
    // Autocomplete methods
    pub async fn update_autocomplete(&mut self) -> Result<()> {
        // Lazy load schema on first use
        if !self.autocomplete_schema_loaded {
            if let Some(client) = self.db.client() {
                let mut tables_with_columns = Vec::new();
                
                for schema in &self.schemas {
                    let tables = crate::db::list_tables(client, &schema.name).await?;
                    
                    for table in tables {
                        let columns = crate::db::describe_table(client, &schema.name, &table.name).await?;
                        let column_names: Vec<String> = columns.iter().map(|c| c.name.clone()).collect();
                        tables_with_columns.push((table.name.clone(), column_names));
                    }
                }
                
                self.autocomplete_engine.update_schema(tables_with_columns);
                self.autocomplete_schema_loaded = true;
            }
        }
        
        self.suggestions = self.autocomplete_engine.get_suggestions(&self.query_input, self.query_cursor);
        self.show_autocomplete = !self.suggestions.is_empty();
        self.suggestion_selected = 0;
        Ok(())
    }
    
    pub fn select_next_suggestion(&mut self) {
        if !self.suggestions.is_empty() {
            self.suggestion_selected = (self.suggestion_selected + 1) % self.suggestions.len();
        }
    }
    
    pub fn select_prev_suggestion(&mut self) {
        if !self.suggestions.is_empty() {
            if self.suggestion_selected == 0 {
                self.suggestion_selected = self.suggestions.len() - 1;
            } else {
                self.suggestion_selected -= 1;
            }
        }
    }
    
    pub fn accept_suggestion(&mut self) {
        if self.suggestion_selected < self.suggestions.len() {
            let suggestion = &self.suggestions[self.suggestion_selected];
            
            // Find the start of the current word being typed
            let mut word_start = self.query_cursor;
            let chars: Vec<char> = self.query_input.chars().collect();
            
            while word_start > 0 {
                let prev_char = chars[word_start - 1];
                if prev_char.is_alphanumeric() || prev_char == '_' {
                    word_start -= 1;
                } else {
                    break;
                }
            }
            
            // Remove the partial word
            self.query_input.drain(word_start..self.query_cursor);
            
            // Insert the suggestion
            let insert_text = suggestion.text.clone();
            for (i, c) in insert_text.chars().enumerate() {
                self.query_input.insert(word_start + i, c);
            }
            
            // Move cursor to end of inserted text
            self.query_cursor = word_start + insert_text.len();
            
            // Add a space after keywords
            if matches!(suggestion.suggestion_type, crate::autocomplete::SuggestionType::Keyword) {
                self.query_input.insert(self.query_cursor, ' ');
                self.query_cursor += 1;
            }
            
            // Hide autocomplete
            self.show_autocomplete = false;
            self.suggestions.clear();
        }
    }
    
    pub fn hide_autocomplete(&mut self) {
        self.show_autocomplete = false;
        self.suggestions.clear();
        self.suggestion_selected = 0;
    }
    
    // Query formatting
    pub fn format_current_query(&mut self) {
        use crate::formatter::SqlFormatter;
        
        if self.query_input.is_empty() {
            return;
        }
        
        // Find all semicolon positions
        let semicolons: Vec<usize> = self.query_input
            .char_indices()
            .filter_map(|(i, c)| if c == ';' { Some(i) } else { None })
            .collect();
        
        // If no semicolons, format the entire input
        if semicolons.is_empty() {
            let formatter = SqlFormatter::new();
            let formatted = formatter.format(&self.query_input);
            self.query_cursor = formatted.len(); // Move cursor to end
            self.query_input = formatted;
            return;
        }
        
        // Find which query the cursor is in
        let cursor_pos = self.query_cursor;
        
        // Find the start of current query (after previous semicolon or beginning)
        let query_start = semicolons
            .iter()
            .rev()
            .find(|&&pos| pos < cursor_pos)
            .map(|&pos| pos + 1)
            .unwrap_or(0);
        
        // Find the end of current query (at next semicolon or end)
        let query_end = semicolons
            .iter()
            .find(|&&pos| pos >= cursor_pos)
            .copied()
            .unwrap_or(self.query_input.len());
        
        // Extract the query
        let query = &self.query_input[query_start..query_end];
        
        // Format it
        let formatter = SqlFormatter::new();
        let formatted = formatter.format(query.trim());
        
        // Replace in the original input
        let mut new_input = String::new();
        
        // Add everything before the query
        new_input.push_str(&self.query_input[..query_start]);
        
        // Add formatted query
        if query_start > 0 {
            new_input.push('\n');
        }
        new_input.push_str(&formatted);
        
        // Add everything after the query
        if query_end < self.query_input.len() {
            new_input.push_str(&self.query_input[query_end..]);
        }
        
        // Update cursor to end of formatted query
        self.query_cursor = query_start + formatted.len() + if query_start > 0 { 1 } else { 0 };
        self.query_input = new_input;
    }
}

impl Default for App {
    fn default() -> Self {
        Self::new()
    }
}

