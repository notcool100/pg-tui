use std::collections::HashMap;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SuggestionType {
    Keyword,
    Table,
    Column,
    Function,
}

#[derive(Debug, Clone)]
pub struct Suggestion {
    pub suggestion_type: SuggestionType,
    pub text: String,
    pub description: Option<String>,
}

impl Suggestion {
    pub fn new(suggestion_type: SuggestionType, text: String, description: Option<String>) -> Self {
        Self {
            suggestion_type,
            text,
            description,
        }
    }
}

pub struct AutocompleteEngine {
    keywords: Vec<String>,
    tables: Vec<String>,
    // Map of table name to list of column names
    columns: HashMap<String, Vec<String>>,
}

impl AutocompleteEngine {
    pub fn new() -> Self {
        let keywords = vec![
            // DML
            "SELECT", "FROM", "WHERE", "INSERT", "INTO", "VALUES", "UPDATE", "SET", "DELETE",
            "JOIN", "INNER JOIN", "LEFT JOIN", "RIGHT JOIN", "FULL JOIN", "CROSS JOIN",
            "ON", "AND", "OR", "NOT", "IN", "BETWEEN", "LIKE", "IS", "NULL",
            "ORDER BY", "GROUP BY", "HAVING", "LIMIT", "OFFSET",
            "DISTINCT", "AS", "ASC", "DESC",
            // DDL
            "CREATE", "ALTER", "DROP", "TABLE", "DATABASE", "INDEX", "VIEW",
            "PRIMARY KEY", "FOREIGN KEY", "REFERENCES", "CONSTRAINT", "UNIQUE",
            "CHECK", "DEFAULT", "AUTO_INCREMENT",
            // Data types
            "INTEGER", "INT", "SMALLINT", "BIGINT", "SERIAL", "BIGSERIAL",
            "NUMERIC", "DECIMAL", "REAL", "DOUBLE PRECISION", "FLOAT",
            "VARCHAR", "CHAR", "TEXT", "BOOLEAN", "BOOL",
            "DATE", "TIME", "TIMESTAMP", "TIMESTAMPTZ", "INTERVAL",
            "JSON", "JSONB", "UUID", "BYTEA",
            // Aggregate functions
            "COUNT", "SUM", "AVG", "MIN", "MAX",
            // String functions
            "CONCAT", "SUBSTRING", "LENGTH", "UPPER", "LOWER", "TRIM",
            // Date functions
            "NOW", "CURRENT_DATE", "CURRENT_TIME", "CURRENT_TIMESTAMP",
            // Other
            "CASE", "WHEN", "THEN", "ELSE", "END",
            "EXISTS", "ANY", "ALL", "UNION", "INTERSECT", "EXCEPT",
            "BEGIN", "COMMIT", "ROLLBACK", "TRANSACTION",
        ]
        .iter()
        .map(|s| s.to_string())
        .collect();

        Self {
            keywords,
            tables: Vec::new(),
            columns: HashMap::new(),
        }
    }

    pub fn update_schema(&mut self, tables: Vec<(String, Vec<String>)>) {
        self.tables.clear();
        self.columns.clear();
        
        for (table_name, columns) in tables {
            self.tables.push(table_name.clone());
            self.columns.insert(table_name, columns);
        }
    }

    pub fn get_suggestions(&self, query: &str, cursor_pos: usize) -> Vec<Suggestion> {
        // Extract the word being typed at cursor position
        let (current_word, word_start) = self.extract_current_word(query, cursor_pos);
        
        if current_word.is_empty() {
            return Vec::new();
        }

        let mut suggestions = Vec::new();
        let current_word_upper = current_word.to_uppercase();
        
        // Check if user is typing table.column pattern (e.g., users.id)
        if let Some(table_name) = self.extract_table_before_dot(query, word_start) {
            // Show ONLY columns from this specific table
            if let Some(columns) = self.columns.get(&table_name) {
                suggestions = columns
                    .iter()
                    .filter(|col| current_word.is_empty() || col.to_uppercase().starts_with(&current_word_upper))
                    .map(|col| Suggestion::new(
                        SuggestionType::Column,
                        col.clone(),
                        Some(format!("Column in {}", table_name)),
                    ))
                    .collect();
            }
            suggestions.truncate(10);
            return suggestions;
        }

        // Determine context to prioritize suggestions
        let context = self.analyze_context(query, word_start);

        match context {
            Context::TableName => {
                // Prioritize table suggestions
                suggestions.extend(self.match_tables(&current_word_upper));
                suggestions.extend(self.match_keywords(&current_word_upper));
            }
            Context::ColumnName => {
                // Prioritize column suggestions
                suggestions.extend(self.match_columns(&current_word_upper, query, word_start));
                suggestions.extend(self.match_keywords(&current_word_upper));
            }
            Context::General => {
                // General context: keywords first, then tables, then columns
                suggestions.extend(self.match_keywords(&current_word_upper));
                suggestions.extend(self.match_tables(&current_word_upper));
                suggestions.extend(self.match_all_columns(&current_word_upper));
            }
        }

        // Limit to top 10 suggestions
        suggestions.truncate(10);
        suggestions
    }

    fn extract_current_word(&self, text: &str, cursor_pos: usize) -> (String, usize) {
        if text.is_empty() || cursor_pos == 0 {
            return (String::new(), 0);
        }

        let safe_pos = cursor_pos.min(text.len());
        
        // Find word boundaries (alphanumeric + underscore)
        let mut word_start = safe_pos;
        let chars: Vec<char> = text.chars().collect();
        
        // Move back to find start of word
        while word_start > 0 {
            let prev_char = chars[word_start - 1];
            if prev_char.is_alphanumeric() || prev_char == '_' {
                word_start -= 1;
            } else {
                break;
            }
        }

        // Extract the word from start to cursor
        let word: String = chars[word_start..safe_pos].iter().collect();
        (word, word_start)
    }
    
    // Helper to check if there's a table.column pattern
    fn extract_table_before_dot(&self, text: &str, word_start: usize) -> Option<String> {
        if word_start == 0 {
            return None;
        }
        
        let chars: Vec<char> = text.chars().collect();
        
        // Check if there's a dot right before the word
        if word_start > 0 && chars.get(word_start - 1) == Some(&'.') {
            // Find the table name before the dot
            let mut table_end = word_start - 1;
            let mut table_start = table_end;
            
            while table_start > 0 {
                let prev_char = chars[table_start - 1];
                if prev_char.is_alphanumeric() || prev_char == '_' {
                    table_start -= 1;
                } else {
                    break;
                }
            }
            
            let table_name: String = chars[table_start..table_end].iter().collect();
            if !table_name.is_empty() {
                return Some(table_name);
            }
        }
        
        None
    }

    fn analyze_context(&self, query: &str, cursor_pos: usize) -> Context {
        let before_cursor = &query[..cursor_pos.min(query.len())];
        let upper = before_cursor.to_uppercase();

        // Simple heuristics for context detection
        if upper.ends_with("FROM ") || upper.contains("FROM ") && !upper.contains("WHERE") {
            return Context::TableName;
        }
        
        if upper.starts_with("SELECT ") && !upper.contains("FROM") {
            return Context::ColumnName;
        }

        if upper.contains("WHERE ") || upper.contains("ON ") {
            return Context::ColumnName;
        }

        Context::General
    }

    fn match_keywords(&self, prefix: &str) -> Vec<Suggestion> {
        self.keywords
            .iter()
            .filter(|kw| kw.starts_with(prefix))
            .map(|kw| Suggestion::new(
                SuggestionType::Keyword,
                kw.clone(),
                Some("SQL Keyword".to_string()),
            ))
            .collect()
    }

    fn match_tables(&self, prefix: &str) -> Vec<Suggestion> {
        self.tables
            .iter()
            .filter(|table| table.to_uppercase().starts_with(prefix))
            .map(|table| Suggestion::new(
                SuggestionType::Table,
                table.clone(),
                Some("Table".to_string()),
            ))
            .collect()
    }

    fn match_columns(&self, prefix: &str, query: &str, _word_start: usize) -> Vec<Suggestion> {
        // Try to find the table in the query context
        let table_name = self.extract_table_from_query(query);
        
        if let Some(table) = table_name {
            if let Some(columns) = self.columns.get(&table) {
                return columns
                    .iter()
                    .filter(|col| col.to_uppercase().starts_with(prefix))
                    .map(|col| Suggestion::new(
                        SuggestionType::Column,
                        col.clone(),
                        Some(format!("Column in {}", table)),
                    ))
                    .collect();
            }
        }

        // Fall back to all columns
        self.match_all_columns(prefix)
    }

    fn match_all_columns(&self, prefix: &str) -> Vec<Suggestion> {
        let mut results = Vec::new();
        for (table, columns) in &self.columns {
            for col in columns {
                if col.to_uppercase().starts_with(prefix) {
                    results.push(Suggestion::new(
                        SuggestionType::Column,
                        col.clone(),
                        Some(format!("Column in {}", table)),
                    ));
                }
            }
        }
        results
    }

    fn extract_table_from_query(&self, query: &str) -> Option<String> {
        let upper = query.to_uppercase();
        
        // Look for "FROM table_name"
        if let Some(from_pos) = upper.find("FROM ") {
            let after_from = &query[from_pos + 5..];
            let words: Vec<&str> = after_from.split_whitespace().collect();
            if let Some(first_word) = words.first() {
                let table_name = first_word.trim_end_matches(|c: char| !c.is_alphanumeric() && c != '_');
                return Some(table_name.to_string());
            }
        }
        
        None
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Context {
    TableName,
    ColumnName,
    General,
}

impl Default for AutocompleteEngine {
    fn default() -> Self {
        Self::new()
    }
}
