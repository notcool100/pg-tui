# PostgreSQL TUI Client

A Gobang-inspired terminal user interface (TUI) database client for PostgreSQL, written in Rust.

![License](https://img.shields.io/badge/license-MIT-blue.svg)

## Features

- 🔌 **PostgreSQL Connection Management** - Connect to local or remote PostgreSQL databases
- 📁 **Database Browser** - Navigate through schemas and tables with an intuitive tree interface
- 📊 **Table Structure Viewer** - View column definitions, types, and constraints
- ⌨️ **SQL Query Execution** - Run arbitrary SQL queries with results display
- 🎨 **Beautiful TUI** - Clean, modern terminal interface built with ratatui
- ⚡ **Fast & Lightweight** - Minimal resource usage, blazing fast performance

## Installation

### Prerequisites

- Rust 1.70+ (install from [rustup.rs](https://rustup.rs))
- PostgreSQL database (local or remote)

### Build from Source

```bash
git clone <repository-url>
cd psql_cli
cargo build --release
```

The compiled binary will be at `target/release/psql_cli`.

## Usage

### Starting the Application

```bash
cargo run
```

Or if you've built the release version:

```bash
./target/release/psql_cli
```

### Connection Manager  

On first launch (or when you have saved connections), you'll see the connection manager:

- **↑/↓**: Navigate through saved connections
- **Enter**: Select a connection to use (you'll just need to enter the password)
- **n**: Create a new connection
- **d**: Delete the selected connection
- **q**: Quit

The app automatically saves your connection details (except password) after successful login, so next time you can just select it from the list!

### Connection Screen

When you first launch the application, you'll see the connection screen:

1. Enter your PostgreSQL credentials:
   - **Host**: Database server address (default: localhost)
   - **Port**: Database port (default: 5432)
   - **Database**: Name of the database to connect to
   - **User**: PostgreSQL username
   - **Password**: User password (masked)

2. Navigate between fields using **Tab** / **Shift+Tab**
3. Press **Enter** to connect

### Database Browser

Once connected, you'll enter the browser mode:

- **↑/↓**: Navigate through schemas and tables
- **Enter**: Expand a schema to view its tables, or select a table to view its structure
- **Tab**: Switch to query mode
- **r**: Refresh the browser view
- **q**: Quit the application

### Query Editor

Switch to query mode to run SQL queries:

1. Type your SQL query in the editor (supports multi-line)
2. Press **Ctrl+Enter** or **F5** to execute
3. Results will appear in the panel below
4. Press **Tab** to switch back to browser mode

## Keyboard Shortcuts

### Connection Manager
| Key | Action |
|-----|--------|
| `↑` / `↓` | Navigate saved connections |
| `Enter` | Select connection |
| `n` | New connection |
| `d` | Delete selected connection |
| `q` | Quit |

### General
| Key | Action |
|-----|--------|
| `q` | Quit application |
| `Tab` | Switch between browser and query modes |
| `↑` / `↓` | Navigate lists |
| `Enter` | Select/Expand item or connect |
| `Ctrl+Enter` / `F5` | Execute SQL query (in query mode) |
| `r` | Refresh browser (in browser mode) |
| `Esc` | Exit current screen / Go back |

## Architecture

The application is structured into several key modules:

- **`main.rs`** - Application entry point and event loop
- **`app.rs`** - Application state management
- **`db/`** - Database connection and query layer
  - `connection.rs` - PostgreSQL connection management
  - `queries.rs` - Database query functions
- **`ui/`** - User interface components
  - `connection.rs` - Connection screen
  - `browser.rs` - Database browser and detail panels
  - `query.rs` - Query editor and results display
- **`config.rs`** - Configuration management (future: saved connections)
- **`events.rs`** - Event handling utilities

## Tech Stack

- **[ratatui](https://github.com/ratatui-org/ratatui)** - Terminal UI framework
- **[crossterm](https://github.com/crossterm-rs/crossterm)** - Cross-platform terminal manipulation
- **[tokio](https://tokio.rs/)** - Async runtime
- **[tokio-postgres](https://github.com/sfackler/rust-postgres)** - PostgreSQL async client
- **[serde](https://serde.rs/)** - Serialization framework
- **[anyhow](https://github.com/dtolnay/anyhow)** - Error handling

## Roadmap

- [ ] Save and manage connection profiles
- [ ] Query history and recall
- [ ] Data editing capabilities
- [ ] Transaction support
- [ ] Export results to CSV/JSON
- [ ] Syntax highlighting in query editor
- [ ] Auto-completion for SQL keywords
- [ ] Support for multiple database types (MySQL, SQLite)
- [ ] SSH tunnel support

## License

MIT License - see LICENSE file for details

## Acknowledgments

Inspired by [Gobang](https://github.com/TaKO8Ki/gobang), the excellent TUI database client.

## Contributing

Contributions are welcome! Please feel free to submit a Pull Request.
