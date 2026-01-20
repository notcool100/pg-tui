use anyhow::{Context, Result};
use tokio_postgres::{Client, NoTls};

pub struct DbConnection {
    client: Option<Client>,
}

impl DbConnection {
    pub fn new() -> Self {
        Self { client: None }
    }

    pub async fn connect(
        &mut self,
        host: &str,
        port: u16,
        database: &str,
        user: &str,
        password: &str,
    ) -> Result<()> {
        let config = format!(
            "host={} port={} dbname={} user={} password={}",
            host, port, database, user, password
        );

        let (client, connection) = tokio_postgres::connect(&config, NoTls)
            .await
            .context("Failed to connect to database")?;

        // Spawn connection handler
        tokio::spawn(async move {
            if let Err(e) = connection.await {
                eprintln!("Connection error: {}", e);
            }
        });

        self.client = Some(client);
        Ok(())
    }

    pub fn client(&self) -> Option<&Client> {
        self.client.as_ref()
    }

    pub fn is_connected(&self) -> bool {
        self.client.is_some()
    }

    pub fn disconnect(&mut self) {
        self.client = None;
    }
}

impl Default for DbConnection {
    fn default() -> Self {
        Self::new()
    }
}
