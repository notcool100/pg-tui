use anyhow::{Context, Result};
use tokio_postgres::Client;

use super::{Column, Database, QueryResult, Schema, Table};

pub async fn list_databases(client: &Client) -> Result<Vec<Database>> {
    let rows = client
        .query(
            "SELECT datname, pg_catalog.pg_get_userbyid(datdba) as owner 
             FROM pg_database 
             WHERE datistemplate = false 
             ORDER BY datname",
            &[],
        )
        .await
        .context("Failed to list databases")?;

    let databases = rows
        .iter()
        .map(|row| Database {
            name: row.get(0),
            owner: row.get(1),
        })
        .collect();

    Ok(databases)
}

pub async fn list_schemas(client: &Client, _database: &str) -> Result<Vec<Schema>> {
    let rows = client
        .query(
            "SELECT schema_name 
             FROM information_schema.schemata 
             WHERE schema_name NOT IN ('pg_catalog', 'information_schema', 'pg_toast')
             ORDER BY schema_name",
            &[],
        )
        .await
        .context("Failed to list schemas")?;

    let schemas = rows
        .iter()
        .map(|row| Schema {
            name: row.get(0),
        })
        .collect();

    Ok(schemas)
}

pub async fn list_tables(client: &Client, schema: &str) -> Result<Vec<Table>> {
    let rows = client
        .query(
            "SELECT table_schema, table_name 
             FROM information_schema.tables 
             WHERE table_schema = $1 
             AND table_type = 'BASE TABLE'
             ORDER BY table_name",
            &[&schema],
        )
        .await
        .context("Failed to list tables")?;

    let tables = rows
        .iter()
        .map(|row| Table {
            schema: row.get(0),
            name: row.get(1),
            row_count: None,
        })
        .collect();

    Ok(tables)
}

pub async fn describe_table(client: &Client, schema: &str, table: &str) -> Result<Vec<Column>> {
    let rows = client
        .query(
            "SELECT column_name, data_type, is_nullable, column_default
             FROM information_schema.columns
             WHERE table_schema = $1 AND table_name = $2
             ORDER BY ordinal_position",
            &[&schema, &table],
        )
        .await
        .context("Failed to describe table")?;

    let columns = rows
        .iter()
        .map(|row| Column {
            name: row.get(0),
            data_type: row.get(1),
            is_nullable: row.get(2),
            column_default: row.get(3),
        })
        .collect();

    Ok(columns)
}

pub async fn execute_query(client: &Client, sql: &str) -> Result<QueryResult> {
    let rows = client
        .query(sql, &[])
        .await
        .context("Failed to execute query")?;

    if rows.is_empty() {
        return Ok(QueryResult {
            columns: vec![],
            rows: vec![],
            row_count: 0,
        });
    }

    let columns: Vec<String> = rows[0]
        .columns()
        .iter()
        .map(|col| col.name().to_string())
        .collect();

    let data_rows: Vec<Vec<String>> = rows
        .iter()
        .map(|row| {
            (0..row.len())
                .map(|i| {
                    row.try_get::<_, Option<String>>(i)
                        .unwrap_or(None)
                        .unwrap_or_else(|| "NULL".to_string())
                })
                .collect()
        })
        .collect();

    let row_count = data_rows.len();

    Ok(QueryResult {
        columns,
        rows: data_rows,
        row_count,
    })
}
