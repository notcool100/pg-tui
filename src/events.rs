use crossterm::event::Event;
use std::time::Duration;

pub struct EventHandler;

impl EventHandler {
    pub fn new() -> Self {
        Self
    }

    pub fn next(&self) -> anyhow::Result<Option<Event>> {
        if crossterm::event::poll(Duration::from_millis(100))? {
            Ok(Some(crossterm::event::read()?))
        } else {
            Ok(None)
        }
    }
}

impl Default for EventHandler {
    fn default() -> Self {
        Self::new()
    }
}
