use crate::syntax::{SqlHighlighter, TokenType};

pub struct SqlFormatter {
    indent_size: usize,
    keyword_case: KeywordCase,
}

#[derive(Debug, Clone, Copy)]
pub enum KeywordCase {
    Upper,
    Lower,
}

impl SqlFormatter {
    pub fn new() -> Self {
        Self {
            indent_size: 4,
            keyword_case: KeywordCase::Upper,
        }
    }

    pub fn format(&self, sql: &str) -> String {
        let highlighter = SqlHighlighter::new();
        let tokens = highlighter.tokenize(sql);
        
        let mut result = String::new();
        let mut indent_level = 0;
        let mut after_select = false;
        let mut after_major_clause = false;
        let mut first_column = true;
        
        for (i, token) in tokens.iter().enumerate() {
            let next_token = tokens.get(i + 1);
            let prev_token = if i > 0 { tokens.get(i - 1) } else { None };
            
            match token.token_type {
                TokenType::Keyword => {
                    let keyword_upper = token.text.to_uppercase();
                    
                    // Major clauses that should start on new line
                    if matches!(
                        keyword_upper.as_str(),
                        "SELECT" | "FROM" | "WHERE" | "GROUP" | "HAVING" | 
                        "ORDER" | "LIMIT" | "OFFSET" | "UNION" | "INTERSECT" | "EXCEPT"
                    ) {
                        if !result.is_empty() && !result.ends_with('\n') {
                            result.push('\n');
                        }
                        result.push_str(&self.indent(indent_level));
                        result.push_str(&self.apply_keyword_case(&keyword_upper));
                        
                        if keyword_upper == "SELECT" {
                            after_select = true;
                            first_column = true;
                        }
                        after_major_clause = true;
                    }
                    // JOIN keywords
                    else if matches!(
                        keyword_upper.as_str(),
                        "JOIN" | "INNER" | "LEFT" | "RIGHT" | "FULL" | "CROSS"
                    ) {
                        // JOIN on new line
                        if keyword_upper.contains("JOIN") || 
                           (next_token.map(|t| t.text.to_uppercase().contains("JOIN")).unwrap_or(false)) {
                            if !result.ends_with('\n') {
                                result.push('\n');
                            }
                            result.push_str(&self.indent(indent_level));
                        } else {
                            result.push(' ');
                        }
                        result.push_str(&self.apply_keyword_case(&keyword_upper));
                    }
                    // ON keyword
                    else if keyword_upper == "ON" {
                        result.push('\n');
                        result.push_str(&self.indent(indent_level + 1));
                        result.push_str(&self.apply_keyword_case(&keyword_upper));
                    }
                    // AND/OR in WHERE clause
                    else if matches!(keyword_upper.as_str(), "AND" | "OR") {
                        result.push('\n');
                        result.push_str(&self.indent(indent_level + 1));
                        result.push_str(&self.apply_keyword_case(&keyword_upper));
                    }
                    // BY following GROUP/ORDER
                    else if keyword_upper == "BY" {
                        result.push(' ');
                        result.push_str(&self.apply_keyword_case(&keyword_upper));
                    }
                    // Other keywords
                    else {
                        if after_major_clause && !result.ends_with(' ') && !result.ends_with('\n') {
                            result.push(' ');
                        }
                        result.push_str(&self.apply_keyword_case(&keyword_upper));
                    }
                }
                
                TokenType::Punctuation if token.text == "," => {
                    result.push(',');
                    
                    // After comma in SELECT, add newline and indent
                    if after_select {
                        result.push('\n');
                        result.push_str(&self.indent(indent_level + 1));
                        first_column = false;
                    }
                }
                
                TokenType::Punctuation if token.text == "(" => {
                    result.push('(');
                    indent_level += 1;
                }
                
                TokenType::Punctuation if token.text == ")" => {
                    if indent_level > 0 {
                        indent_level -= 1;
                    }
                    result.push(')');
                }
                
                TokenType::Punctuation if token.text == ";" => {
                    result.push(';');
                }
                
                TokenType::Whitespace => {
                    // Skip most whitespace - we control it
                    continue;
                }
                
                TokenType::Identifier | TokenType::String | TokenType::Number => {
                    // Add appropriate spacing
                    if after_major_clause && !result.ends_with(' ') && !result.ends_with('\n') {
                        after_major_clause = false;
                    }
                    
                    if after_select && first_column {
                        result.push('\n');
                        result.push_str(&self.indent(indent_level + 1));
                        first_column = false;
                    } else if !result.is_empty() && 
                              !result.ends_with('\n') && 
                              !result.ends_with(' ') && 
                              !result.ends_with('(') {
                        result.push(' ');
                    }
                    
                    result.push_str(&token.text);
                    
                    if after_select && 
                       next_token.map(|t| matches!(t.token_type, TokenType::Keyword)).unwrap_or(false) {
                        after_select = false;
                    }
                }
                
                TokenType::Operator => {
                    // Add space before operator
                    if !result.ends_with(' ') && !result.is_empty() {
                        result.push(' ');
                    }
                    result.push_str(&token.text);
                    // Space after operator will be added by next identifier
                }
                
                TokenType::Comment => {
                    result.push_str(&token.text);
                }
                
                _ => {
                    result.push_str(&token.text);
                }
            }
        }
        
        result.trim().to_string()
    }
    
    fn indent(&self, level: usize) -> String {
        " ".repeat(self.indent_size * level)
    }
    
    fn apply_keyword_case(&self, keyword: &str) -> String {
        match self.keyword_case {
            KeywordCase::Upper => keyword.to_uppercase(),
            KeywordCase::Lower => keyword.to_lowercase(),
        }
    }
}

impl Default for SqlFormatter {
    fn default() -> Self {
        Self::new()
    }
}
