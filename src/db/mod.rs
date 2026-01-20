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

#[derive(Debug, Clone)]
pub struct Constraint {
    pub name: String,
    pub constraint_type: String,
    pub column_names: String,
}

#[derive(Debug, Clone)]
pub struct Index {
    pub name: String,
    pub columns: String,
    pub is_unique: bool,
    pub is_primary: bool,
}

#[derive(Debug, Clone)]
pub struct Trigger {
    pub name: String,
    pub event: String,
    pub timing: String,
    pub action_statement: String,
}

#[derive(Debug, Clone)]
pub struct ForeignKey {
    pub name: String,
    pub column_names: String,
    pub referenced_table: String,
    pub referenced_columns: String,
}
