use anyhow::Result;
use crossterm::event::KeyCode;
use std::collections::HashSet;

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
    
    // UI state
    pub error_message: Option<String>,
    
    // Filter state
    pub filter_input: String,
    pub filter_active: bool,
    
    // Expanded items tracking
    pub expanded_items: HashSet<String>,
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
            error_message: None,
            filter_input: String::new(),
            filter_active: false,
            expanded_items: HashSet::new(),
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

    pub async fn execute_query(&mut self) -> Result<()> {
        if let Some(client) = self.db.client() {
            let sql = self.query_input.trim();
            if !sql.is_empty() {
                match crate::db::execute_query(client, sql).await {
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
}

impl Default for App {
    fn default() -> Self {
        Self::new()
    }
}
