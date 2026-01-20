use ratatui::style::{Color, Style};

#[derive(Debug, Clone, PartialEq)]
pub enum TokenType {
    Keyword,
    String,
    Number,
    Identifier,
    Operator,
    Comment,
    Whitespace,
    Punctuation,
}

#[derive(Debug, Clone)]
pub struct Token {
    pub token_type: TokenType,
    pub text: String,
}

impl Token {
    fn new(token_type: TokenType, text: String) -> Self {
        Self { token_type, text }
    }
    
    pub fn style(&self) -> Style {
        match self.token_type {
            TokenType::Keyword => Style::default().fg(Color::Cyan),
            TokenType::String => Style::default().fg(Color::Green),
            TokenType::Number => Style::default().fg(Color::Yellow),
            TokenType::Comment => Style::default().fg(Color::DarkGray),
            TokenType::Operator => Style::default().fg(Color::Magenta),
            TokenType::Identifier => Style::default().fg(Color::White),
            TokenType::Whitespace => Style::default(),
            TokenType::Punctuation => Style::default().fg(Color::White),
        }
    }
}

pub struct SqlHighlighter {
    keywords: Vec<String>,
}

impl SqlHighlighter {
    pub fn new() -> Self {
        let keywords = vec![
            // DML
            "SELECT", "FROM", "WHERE", "INSERT", "INTO", "VALUES", "UPDATE", "SET", "DELETE",
            "JOIN", "INNER", "LEFT", "RIGHT", "FULL", "CROSS", "ON", "AND", "OR", "NOT",
            "IN", "BETWEEN", "LIKE", "IS", "NULL", "AS",
            "ORDER", "BY", "GROUP", "HAVING", "LIMIT", "OFFSET", "DISTINCT",
            "ASC", "DESC", "UNION", "INTERSECT", "EXCEPT", "ALL", "ANY",
            // DDL
            "CREATE", "ALTER", "DROP", "TABLE", "DATABASE", "INDEX", "VIEW", "SCHEMA",
            "PRIMARY", "KEY", "FOREIGN", "REFERENCES", "CONSTRAINT", "UNIQUE",
            "CHECK", "DEFAULT", "CASCADE",
            // Data types
            "INTEGER", "INT", "SMALLINT", "BIGINT", "SERIAL", "BIGSERIAL",
            "NUMERIC", "DECIMAL", "REAL", "DOUBLE", "PRECISION", "FLOAT",
            "VARCHAR", "CHAR", "TEXT", "BOOLEAN", "BOOL",
            "DATE", "TIME", "TIMESTAMP", "TIMESTAMPTZ", "INTERVAL",
            "JSON", "JSONB", "UUID", "BYTEA", "ARRAY",
            // Functions
            "COUNT", "SUM", "AVG", "MIN", "MAX", "COALESCE", "NULLIF",
            "CONCAT", "SUBSTRING", "LENGTH", "UPPER", "LOWER", "TRIM",
            "NOW", "CURRENT_DATE", "CURRENT_TIME", "CURRENT_TIMESTAMP",
            // Control flow
            "CASE", "WHEN", "THEN", "ELSE", "END", "EXISTS",
            "BEGIN", "COMMIT", "ROLLBACK", "TRANSACTION",
        ]
        .iter()
        .map(|s| s.to_string())
        .collect();

        Self { keywords }
    }

    pub fn tokenize(&self, input: &str) -> Vec<Token> {
        let mut tokens = Vec::new();
        let mut chars = input.char_indices().peekable();

        while let Some((i, ch)) = chars.next() {
            match ch {
                // Whitespace
                ' ' | '\t' | '\n' | '\r' => {
                    let mut text = String::from(ch);
                    while let Some(&(_, next_ch)) = chars.peek() {
                        if next_ch.is_whitespace() {
                            text.push(next_ch);
                            chars.next();
                        } else {
                            break;
                        }
                    }
                    tokens.push(Token::new(TokenType::Whitespace, text));
                }
                
                // String literals (single quotes)
                '\'' => {
                    let mut text = String::from(ch);
                    let mut escaped = false;
                    
                    while let Some((_, next_ch)) = chars.next() {
                        text.push(next_ch);
                        if escaped {
                            escaped = false;
                        } else if next_ch == '\\' {
                            escaped = true;
                        } else if next_ch == '\'' {
                            break;
                        }
                    }
                    tokens.push(Token::new(TokenType::String, text));
                }
                
                // Comments (-- style)
                '-' if chars.peek().map(|(_, c)| *c) == Some('-') => {
                    let mut text = String::from(ch);
                    text.push(chars.next().unwrap().1); // consume second '-'
                    
                    while let Some(&(_, next_ch)) = chars.peek() {
                        if next_ch == '\n' {
                            break;
                        }
                        text.push(next_ch);
                        chars.next();
                    }
                    tokens.push(Token::new(TokenType::Comment, text));
                }
                
                // Numbers
                '0'..='9' => {
                    let mut text = String::from(ch);
                    let mut has_dot = false;
                    
                    while let Some(&(_, next_ch)) = chars.peek() {
                        if next_ch.is_ascii_digit() {
                            text.push(next_ch);
                            chars.next();
                        } else if next_ch == '.' && !has_dot {
                            has_dot = true;
                            text.push(next_ch);
                            chars.next();
                        } else {
                            break;
                        }
                    }
                    tokens.push(Token::new(TokenType::Number, text));
                }
                
                // Operators and punctuation
                '=' | '>' | '<' | '!' | '+' | '-' | '*' | '/' | '%' | '|' | '&' => {
                    let mut text = String::from(ch);
                    // Handle multi-char operators like >=, <=, !=, <>
                    if let Some(&(_, next_ch)) = chars.peek() {
                        if (ch == '>' || ch == '<' || ch == '!') && (next_ch == '=' || next_ch == '>') {
                            text.push(next_ch);
                            chars.next();
                        }
                    }
                    tokens.push(Token::new(TokenType::Operator, text));
                }
                
                '(' | ')' | ',' | ';' | '.' => {
                    tokens.push(Token::new(TokenType::Punctuation, String::from(ch)));
                }
                
                // Identifiers and keywords
                _ if ch.is_alphabetic() || ch == '_' => {
                    let mut text = String::from(ch);
                    
                    while let Some(&(_, next_ch)) = chars.peek() {
                        if next_ch.is_alphanumeric() || next_ch == '_' {
                            text.push(next_ch);
                            chars.next();
                        } else {
                            break;
                        }
                    }
                    
                    let token_type = if self.keywords.contains(&text.to_uppercase()) {
                        TokenType::Keyword
                    } else {
                        TokenType::Identifier
                    };
                    
                    tokens.push(Token::new(token_type, text));
                }
                
                // Unknown characters - treat as punctuation
                _ => {
                    tokens.push(Token::new(TokenType::Punctuation, String::from(ch)));
                }
            }
        }

        tokens
    }
}

impl Default for SqlHighlighter {
    fn default() -> Self {
        Self::new()
    }
}
