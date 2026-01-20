use anyhow::Result;
use crossterm::event::KeyCode;

use crate::db::{Column, DbConnection, QueryResult, Schema, Table};

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

#[derive(Debug, Clone)]
pub enum BrowserItem {
    Schema(String),
    Table(String, String), // schema, table
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
    
    // Query state
    pub query_input: String,
    pub query_result: Option<QueryResult>,
    pub query_cursor: usize,
    
    // UI state
    pub error_message: Option<String>,
    
    // Filter state
    pub filter_input: String,
    pub filter_active: bool,
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
            query_input: String::new(),
            query_result: None,
            query_cursor: 0,
            error_message: None,
            filter_input: String::new(),
            filter_active: false,
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
                    // Load tables for this schema
                    self.tables = crate::db::list_tables(client, schema).await?;
                    
                    // Insert tables after the schema
                    let insert_pos = self.browser_selected + 1;
                    for (i, table) in self.tables.iter().enumerate() {
                        self.browser_items.insert(
                            insert_pos + i,
                            BrowserItem::Table(schema.clone(), table.name.clone()),
                        );
                    }
                }
                BrowserItem::Table(schema, table) => {
                    // Load columns for this table
                    self.columns = crate::db::describe_table(client, schema, table).await?;
                }
            }
        }

        Ok(())
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
                BrowserItem::Table(schema, name) => {
                    // Match if table name or schema name matches
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
}

impl Default for App {
    fn default() -> Self {
        Self::new()
    }
}
