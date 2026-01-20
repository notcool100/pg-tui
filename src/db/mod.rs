mod connection;
mod queries;

pub use connection::DbConnection;
pub use queries::*;

#[derive(Debug, Clone)]
pub struct Database {
    pub name: String,
    pub owner: String,
}

#[derive(Debug, Clone)]
pub struct Schema {
    pub name: String,
}

#[derive(Debug, Clone)]
pub struct Table {
    pub schema: String,
    pub name: String,
    pub row_count: Option<i64>,
}

#[derive(Debug, Clone)]
pub struct Column {
    pub name: String,
    pub data_type: String,
    pub is_nullable: String,
    pub column_default: Option<String>,
}

#[derive(Debug, Clone)]
pub struct View {
    pub schema: String,
    pub name: String,
}

#[derive(Debug, Clone)]
pub struct Function {
    pub schema: String,
    pub name: String,
    pub function_type: String,
}

#[derive(Debug, Clone)]
pub struct QueryResult {
    pub columns: Vec<String>,
    pub rows: Vec<Vec<String>>,
    pub row_count: usize,
}
