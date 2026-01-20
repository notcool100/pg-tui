use anyhow::Result;
use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode, KeyEventKind, KeyModifiers},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{
    backend::CrosstermBackend,
    Terminal,
};
use std::io;

mod app;
mod config;
mod db;
mod events;
mod ui;

use app::{App, AppMode};

#[tokio::main]
async fn main() -> Result<()> {
    // Setup terminal
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // Create app state
    let mut app = App::new();

    // Run app
    let res = run_app(&mut terminal, &mut app).await;

    // Restore terminal
    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;

    if let Err(err) = res {
        eprintln!("Error: {:?}", err);
    }

    Ok(())
}

async fn run_app<B: ratatui::backend::Backend>(
    terminal: &mut Terminal<B>,
    app: &mut App,
) -> Result<()> {
    loop {
        terminal.draw(|f| ui::render(f, app))?;

        if event::poll(std::time::Duration::from_millis(100))? {
            if let Event::Key(key) = event::read()? {
                if key.kind == KeyEventKind::Press {
                    match app.mode {
                        AppMode::ConnectionSelector => {
                            if handle_selector_input(app, key.code) {
                                return Ok(());
                            }
                        }
                        AppMode::ConnectionEdit => {
                            if handle_connection_input(app, key.code).await {
                                return Ok(());
                            }
                        }
                        AppMode::Browser => {
                            if handle_browser_input(app, key.code).await? {
                                return Ok(());
                            }
                        }
                        AppMode::Query => {
                            // Check for Ctrl+Enter to execute query
                            if key.modifiers.contains(KeyModifiers::CONTROL) && key.code == KeyCode::Enter {
                                app.execute_query().await?;
                            } else if handle_query_input(app, key.code).await? {
                                return Ok(());
                            }
                        }
                    }
                }
            }
        }
    }
}


fn handle_selector_input(app: &mut App, key: KeyCode) -> bool {
    match key {
        KeyCode::Char('q') => return true,
        KeyCode::Esc => return true,
        KeyCode::Up => app.selector_up(),
        KeyCode::Down => app.selector_down(),
        KeyCode::Enter => {
            if !app.config.connections.is_empty() {
                app.load_selected_profile();
            }
        }
        KeyCode::Char('n') => app.create_new_connection(),
        KeyCode::Char('d') => {
            if let Err(e) = app.delete_selected_profile() {
                app.set_error(format!("Failed to delete profile: {}", e));
            }
        }
        _ => {}
    }
    false
}

async fn handle_connection_input(app: &mut App, key: KeyCode) -> bool {
    match key {
        KeyCode::Char('q') => return true,
        KeyCode::Esc => {
            app.mode = AppMode::ConnectionSelector;
            return false;
        }
        KeyCode::Tab => app.next_connection_field(),
        KeyCode::BackTab => app.prev_connection_field(),
        KeyCode::Enter => {
            if let Err(e) = app.connect().await {
                app.set_error(format!("Connection failed: {}", e));
            }
        }
        KeyCode::Char(c) => app.input_char(c),
        KeyCode::Backspace => app.delete_char(),
        _ => {}
    }
    false
}

async fn handle_browser_input(app: &mut App, key: KeyCode) -> Result<bool> {
    // Handle filter mode
    if app.filter_active {
        match key {
            KeyCode::Esc => {
                app.clear_filter();
                return Ok(false);
            }
            KeyCode::Enter => {
                // Select in filtered view
                app.browser_select().await?;
                return Ok(false);
            }
            KeyCode::Up => {
                // Navigate in filtered view
                let filtered = app.get_filtered_items();
                if let Some(current_pos) = filtered.iter().position(|&idx| idx == app.browser_selected) {
                    if current_pos > 0 {
                        app.browser_selected = filtered[current_pos - 1];
                    }
                }
                return Ok(false);
            }
            KeyCode::Down => {
                // Navigate in filtered view
                let filtered = app.get_filtered_items();
                if let Some(current_pos) = filtered.iter().position(|&idx| idx == app.browser_selected) {
                    if current_pos < filtered.len() - 1 {
                        app.browser_selected = filtered[current_pos + 1];
                    }
                }
                return Ok(false);
            }
            _ => {
                // Handle filter text input
                app.handle_filter_input(key);
                
                // Auto-adjust selection to first filtered item
                let filtered = app.get_filtered_items();
                if !filtered.is_empty() && !filtered.contains(&app.browser_selected) {
                    app.browser_selected = filtered[0];
                }
                return Ok(false);
            }
        }
    }
    
    // Normal browser mode
    match key {
        KeyCode::Char('q') => return Ok(true),
        KeyCode::Char('/') => {
            app.activate_filter();
            return Ok(false);
        }
        KeyCode::Up => app.browser_up(),
        KeyCode::Down => app.browser_down(),
        KeyCode::Enter => app.browser_select().await?,
        KeyCode::Tab => app.mode = AppMode::Query,
        KeyCode::Char('r') => app.refresh_browser().await?,
        _ => {}
    }
    Ok(false)
}

async fn handle_query_input(app: &mut App, key: KeyCode) -> Result<bool> {
    match key {
        KeyCode::Char('q') if app.query_input.is_empty() => return Ok(true),
        KeyCode::Tab => app.mode = AppMode::Browser,
        KeyCode::F(5) => {
            // F5 to execute query
            app.execute_query().await?;
        }
        _ => {
            // Handle text input in query editor
            app.handle_query_input(key);
        }
    }
    Ok(false)
}
