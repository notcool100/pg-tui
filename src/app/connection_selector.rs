use anyhow::Result;
use crossterm::event::KeyCode;

use crate::app::App;

// Connection selector navigation and actions
impl App {
    pub fn selector_up(&mut self) {
        if self.selected_profile > 0 {
            self.selected_profile -= 1;
        }
    }

    pub fn selector_down(&mut self) {
        if self.selected_profile < self.config.connections.len().saturating_sub(1) {
            self.selected_profile += 1;
        }
    }

    pub fn load_selected_profile(&mut self) {
        if let Some(profile) = self.config.connections.get(self.selected_profile) {
            self.host = profile.host.clone();
            self.port = profile.port.clone();
            self.database = profile.database.clone();
            self.user = profile.user.clone();
            self.password = String::new();
            self.mode = crate::app::AppMode::ConnectionEdit;
            self.connection_field = crate::app::ConnectionField::Password;
        }
    }

    pub fn create_new_connection(&mut self) {
        self.host = "localhost".to_string();
        self.port = "5432".to_string();
        self.database = "postgres".to_string();
        self.user = "postgres".to_string();
        self.password = String::new();
        self.mode = crate::app::AppMode::ConnectionEdit;
        self.connection_field = crate::app::ConnectionField::Host;
    }

    pub fn delete_selected_profile(&mut self) -> Result<()> {
        if self.selected_profile < self.config.connections.len() {
            self.config.connections.remove(self.selected_profile);
            if self.selected_profile > 0 {
                self.selected_profile -= 1;
            }
            self.config.save()?;
        }
        Ok(())
    }
}
