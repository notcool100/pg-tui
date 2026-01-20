use anyhow::{Context, Result};
use tokio_postgres::Client;

use super::{Column, Constraint, Database, ForeignKey, Function, Index, QueryResult, Schema, Table, Trigger, View};

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

pub async fn list_views(client: &Client, schema: &str) -> Result<Vec<View>> {
    let rows = client
        .query(
            "SELECT table_schema, table_name 
             FROM information_schema.views 
             WHERE table_schema = $1
             ORDER BY table_name",
            &[&schema],
        )
        .await
        .context("Failed to list views")?;

    let views = rows
        .iter()
        .map(|row| View {
            schema: row.get(0),
            name: row.get(1),
        })
        .collect();

    Ok(views)
}

pub async fn list_functions(client: &Client, schema: &str) -> Result<Vec<Function>> {
    let rows = client
        .query(
            "SELECT routine_schema, routine_name, routine_type
             FROM information_schema.routines
             WHERE routine_schema = $1
             ORDER BY routine_name",
            &[&schema],
        )
        .await
        .context("Failed to list functions")?;

    let functions = rows
        .iter()
        .map(|row| Function {
            schema: row.get(0),
            name: row.get(1),
            function_type: row.get(2),
        })
        .collect();

    Ok(functions)
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

pub async fn list_table_constraints(client: &Client, schema: &str, table: &str) -> Result<Vec<Constraint>> {
    let rows = client
        .query(
            "SELECT 
                tc.constraint_name,
                tc.constraint_type,
                string_agg(kcu.column_name, ', ' ORDER BY kcu.ordinal_position) as column_names
             FROM information_schema.table_constraints tc
             LEFT JOIN information_schema.key_column_usage kcu 
                ON tc.constraint_name = kcu.constraint_name 
                AND tc.table_schema = kcu.table_schema
             WHERE tc.table_schema = $1 AND tc.table_name = $2
             GROUP BY tc.constraint_name, tc.constraint_type
             ORDER BY tc.constraint_type, tc.constraint_name",
            &[&schema, &table],
        )
        .await
        .context("Failed to list table constraints")?;

    let constraints = rows
        .iter()
        .map(|row| Constraint {
            name: row.get(0),
            constraint_type: row.get(1),
            column_names: row.get::<_, Option<String>>(2).unwrap_or_else(|| "-".to_string()),
        })
        .collect();

    Ok(constraints)
}

pub async fn list_table_indexes(client: &Client, schema: &str, table: &str) -> Result<Vec<Index>> {
    let rows = client
        .query(
            "SELECT 
                i.indexname as name,
                string_agg(a.attname, ', ' ORDER BY array_position(ix.indkey, a.attnum)) as columns,
                ix.indisunique as is_unique,
                ix.indisprimary as is_primary
             FROM pg_indexes i
             JOIN pg_class c ON c.relname = i.indexname
             JOIN pg_index ix ON ix.indexrelid = c.oid
             JOIN pg_class t ON t.oid = ix.indrelid
             JOIN pg_attribute a ON a.attrelid = t.oid
             WHERE i.schemaname = $1 
                AND i.tablename = $2
                AND a.attnum = ANY(ix.indkey)
             GROUP BY i.indexname, ix.indisunique, ix.indisprimary
             ORDER BY i.indexname",
            &[&schema, &table],
        )
        .await
        .context("Failed to list table indexes")?;

    let indexes = rows
        .iter()
        .map(|row| Index {
            name: row.get(0),
            columns: row.get::<_, Option<String>>(1).unwrap_or_else(|| "-".to_string()),
            is_unique: row.get(2),
            is_primary: row.get(3),
        })
        .collect();

    Ok(indexes)
}

pub async fn list_table_triggers(client: &Client, schema: &str, table: &str) -> Result<Vec<Trigger>> {
    let rows = client
        .query(
            "SELECT 
                trigger_name,
                string_agg(DISTINCT event_manipulation, ', ' ORDER BY event_manipulation) as event,
                action_timing,
                action_statement
             FROM information_schema.triggers
             WHERE event_object_schema = $1 AND event_object_table = $2
             GROUP BY trigger_name, action_timing, action_statement
             ORDER BY trigger_name",
            &[&schema, &table],
        )
        .await
        .context("Failed to list table triggers")?;

    let triggers = rows
        .iter()
        .map(|row| Trigger {
            name: row.get(0),
            event: row.get::<_, Option<String>>(1).unwrap_or_else(|| "-".to_string()),
            timing: row.get(2),
            action_statement: row.get(3),
        })
        .collect();

    Ok(triggers)
}

pub async fn list_table_foreign_keys(client: &Client, schema: &str, table: &str) -> Result<Vec<ForeignKey>> {
    let rows = client
        .query(
            "SELECT 
                tc.constraint_name as name,
                string_agg(DISTINCT kcu.column_name, ', ' ORDER BY kcu.column_name) as column_names,
                ccu.table_name as referenced_table,
                string_agg(DISTINCT ccu.column_name, ', ' ORDER BY ccu.column_name) as referenced_columns
             FROM information_schema.table_constraints tc
             JOIN information_schema.key_column_usage kcu 
                ON tc.constraint_name = kcu.constraint_name 
                AND tc.table_schema = kcu.table_schema
             JOIN information_schema.constraint_column_usage ccu 
                ON ccu.constraint_name = tc.constraint_name 
                AND ccu.table_schema = tc.table_schema
             WHERE tc.constraint_type = 'FOREIGN KEY' 
                AND tc.table_schema = $1 
                AND tc.table_name = $2
             GROUP BY tc.constraint_name, ccu.table_name
             ORDER BY tc.constraint_name",
            &[&schema, &table],
        )
        .await
        .context("Failed to list table foreign keys")?;

    let foreign_keys = rows
        .iter()
        .map(|row| ForeignKey {
            name: row.get(0),
            column_names: row.get::<_, Option<String>>(1).unwrap_or_else(|| "-".to_string()),
            referenced_table: row.get(2),
            referenced_columns: row.get::<_, Option<String>>(3).unwrap_or_else(|| "-".to_string()),
        })
        .collect();

    Ok(foreign_keys)
}
