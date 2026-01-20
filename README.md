# PostgreSQL TUI Client

A modern, feature-rich terminal user interface (TUI) for PostgreSQL with DBeaver-like functionality, written in Rust.

![License](https://img.shields.io/badge/license-MIT-blue.svg)

## ✨ Features

### 🔌 Connection Management
- **Saved Connections** - Save and manage multiple database connections
- **Quick Connect** - Select from saved connections, only enter password
- **Secure** - Passwords never saved to disk

### 📁 Database Browser
- **Interactive Tree View** - Navigate schemas, tables, views, and functions
- **Table Details** - View columns, constraints, indexes, foreign keys, and triggers
- **Expandable** - Collapse/expand schemas for easy navigation

### ⌨️ SQL Query Editor

#### Smart Query Execution
- **Multi-Query Support** - Write multiple queries separated by `;`
- **Execute at Cursor** - Only executes the query where your cursor is
- **Ctrl+Enter or F5** - Quick execution

#### 🎨 Syntax Highlighting
- **Color-Coded** - Keywords (cyan), strings (green), numbers (yellow)
- **Comments** - SQL comments in gray
- **Real-Time** - Highlights as you type

#### 🔍 Intelligent Autocomplete
- **SQL Keywords** - 70+ SQL keywords with prefix matching
- **Table Names** - Autocomplete table names from your database
- **Column Names** - Context-aware column suggestions
- **Table.Column** - Type `users.` to see columns from `users` table
- **Keyboard Navigation** - Arrow keys to navigate, Tab to accept

#### 🎯 Query Formatting
- **Auto-Beautify** - Press **Alt+Shift+F** to format query
- **Proper Indentation** - 4-space indentation
- **Keywords Uppercase** - SQL keywords in UPPERCASE
- **Line Breaks** - Major clauses on new lines
- **Respects Semicolons** - Formats only the query at cursor

### 📊 Results Display
- **Table View** - Clean, scrollable results table
- **Horizontal Scroll** - Handle wide result sets
- **Row Count** - Shows number of rows returned
- **Filter Results** - Ctrl+F to search results

## 🚀 Installation

### Prerequisites

- Rust 1.70+ (install from [rustup.rs](https://rustup.rs))
- PostgreSQL database (local or remote)

### Build from Source

```bash
git clone <repository-url>
cd psql_cli
cargo build --release
```

The optimized binary will be at `target/release/psql_cli`.

### Quick Install

```bash
cargo install --path .
```

## 📖 Usage

### Starting the Application

```bash
./target/release/psql_cli
```

### First Connection

1. **Connection Manager** appears on first launch
2. Press **n** to create a new connection
3. Enter your PostgreSQL credentials:
   - Host (default: localhost)
   - Port (default: 5432)
   - Database name
   - Username
   - Password (masked)
4. Connection is saved automatically after successful login

### Next Connections

1. **Select** from saved connections with ↑/↓
2. Press **Enter**
3. **Enter password** only (other details remembered)
4. **Connect** and start working!

## ⌨️ Keyboard Shortcuts

### Connection Manager
| Key | Action |
|-----|--------|
| `↑` / `↓` | Navigate saved connections |
| `Enter` | Select connection |
| `n` | New connection |
| `d` | Delete selected connection |
| `q` | Quit |

### Browser Mode
| Key | Action |
|-----|--------|
| `↑` / `↓` | Navigate schemas/tables |
| `Enter` | Expand schema or view table details |
| `Tab` | Switch to query mode |
| `r` | Refresh browser |
| `q` | Quit |

### Query Mode
| Key | Action |
|-----|--------|
| `Ctrl+Enter` / `F5` | Execute query at cursor |
| `Alt+Shift+F` | Format/beautify query |
| `Tab` | Switch to browser mode |
| `Ctrl+F` | Filter results |
| `Shift+←/→` | Scroll results horizontally |
| `q` | Quit (when editor is empty) |

### Autocomplete (Query Mode)
| Key | Action |
|-----|--------|
| Type to trigger | Show suggestions |
| `↑` / `↓` | Navigate suggestions |
| `Tab` | Accept selected suggestion |
| `Esc` | Dismiss autocomplete |

## 🎯 Key Features Explained

### Smart Query Execution

Write multiple queries in the editor:
```sql
SELECT * FROM users;

SELECT * FROM orders WHERE status = 'active';

SELECT COUNT(*) FROM products;
```

Place cursor anywhere in a query and press **Ctrl+Enter** - only that query executes!

### Autocomplete Examples

**Keywords:**
```sql
SEL  → suggests SELECT
FRO  → suggests FROM
```

**Tables:**
```sql
SELECT * FROM use  → suggests users, user_sessions
```

**Columns:**
```sql
SELECT id, na  → suggests name, name_first, name_last
```

**Table.Column:**
```sql
users.  → shows all columns from users table
users.em  → suggests email
```

### Query Formatting

**Before:**
```sql
select id,name,email from users where age>18 and status='active' order by created_at desc;
```

**After (Alt+Shift+F):**
```sql
SELECT
    id,
    name,
    email
FROM users
WHERE age > 18
    AND status = 'active'
ORDER BY created_at DESC;
```

## 🏗️ Architecture

```
psql_cli/
├── src/
│   ├── main.rs           # Application entry point
│   ├── app.rs            # State management
│   ├── autocomplete.rs   # SQL autocomplete engine
│   ├── formatter.rs      # SQL query formatter
│   ├── syntax.rs         # Syntax highlighting
│   ├── config.rs         # Connection profiles
│   ├── db/               # Database layer
│   │   ├── connection.rs # PostgreSQL connection
│   │   ├── queries.rs    # SQL queries
│   │   └── mod.rs
│   └── ui/               # UI components
│       ├── browser.rs    # Database browser
│       ├── query.rs      # Query editor
│       ├── connection.rs # Connection screen
│       └── mod.rs
└── Cargo.toml
```

## 🛠️ Tech Stack

- **[ratatui](https://github.com/ratatui-org/ratatui)** - Terminal UI framework
- **[crossterm](https://github.com/crossterm-rs/crossterm)** - Terminal manipulation
- **[tokio](https://tokio.rs/)** - Async runtime
- **[tokio-postgres](https://github.com/sfackler/rust-postgres)** - PostgreSQL client
- **[serde](https://serde.rs/)** - Serialization
- **[anyhow](https://github.com/dtolnay/anyhow)** - Error handling

## 🎯 Comparison with DBeaver

| Feature | psql_cli | DBeaver |
|---------|----------|---------|
| SQL Autocomplete | ✅ | ✅ |
| Syntax Highlighting | ✅ | ✅ |
| Query Formatting | ✅ | ✅ |
| Smart Execution (cursor) | ✅ | ✅ |
| Terminal-based | ✅ | ❌ |
| Lightweight | ✅ (~MB) | ❌ (~100MB) |
| Fast Startup | ✅ (~1s) | ❌ (~10s) |

## 🚀 Performance

- **Binary Size**: ~5MB (release)
- **Memory Usage**: ~10-20MB
- **Startup Time**: <1 second
- **Query Execution**: Instant

## 📝 License

MIT License - see LICENSE file for details

## 🙏 Acknowledgments

Inspired by:
- [Gobang](https://github.com/TaKO8Ki/gobang) - TUI database client
- [DBeaver](https://dbeaver.io/) - Feature inspiration

## 🤝 Contributing

Contributions are welcome! Please feel free to submit a Pull Request.

## 🐛 Issues

Found a bug? Please [open an issue](https://github.com/your-repo/issues).

---

**Built with ❤️ using Rust and Ratatui**
