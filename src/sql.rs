use futures::StreamExt;
use std::collections::HashMap;

use sqlx::{
    Column, PgPool, Row, TypeInfo, ValueRef,
    postgres::{PgPoolOptions, PgRow},
};

pub struct Reader {
    pool: PgPool,
}

impl Reader {
    pub async fn postgres(database_url: String) -> Result<Self, sqlx::Error> {
        // sqlx::any::install_default_drivers();

        let pool = PgPoolOptions::new()
            .max_connections(5)
            .connect(&database_url)
            .await?;

        Ok(Self { pool })
    }

    pub async fn schemas(&self) -> Result<Vec<String>, sqlx::Error> {
        let rows = sqlx::query(
            r#"
                SELECT nspname::text
                FROM pg_namespace
                WHERE nspname NOT LIKE 'pg_%'
                    AND nspname <> 'information_schema'
                ORDER BY nspname
            "#,
        )
        .fetch_all(&self.pool)
        .await?;

        let schemas = rows
            .into_iter()
            .map(|s| s.get::<String, _>("nspname"))
            .collect();

        Ok(schemas)
    }

    pub async fn tables(&self, schema: String) -> Result<Vec<String>, sqlx::Error> {
        let query = format!(
            "SELECT table_name::text FROM information_schema.tables WHERE table_schema = '{}' AND table_type = 'BASE TABLE'",
            schema
        );
        let rows = sqlx::query(&query).fetch_all(&self.pool).await?;

        Ok(rows
            .into_iter()
            .map(|r| r.get::<String, _>("table_name"))
            .collect())
    }

    pub async fn view(
        &self,
        schema: String,
        table: String,
    ) -> Result<Vec<HashMap<String, String>>, sqlx::Error> {
        let mut result: Vec<HashMap<String, String>> = vec![];

        let query = format!("SELECT * FROM \"{}\".\"{}\"", schema, table);
        let mut rows = sqlx::query(&query).fetch(&self.pool);

        while let Some(row_res) = rows.next().await {
            let row: PgRow = row_res?;
            let mut map = HashMap::new();

            for col in row.columns() {
                let name = col.name();
                let raw_value = row.try_get_raw(name)?;
                let stringified = if raw_value.is_null() {
                    "NULL".to_string()
                } else {
                    match row.try_get::<String, _>(name) {
                        Ok(v) => v,
                        Err(_) => format!("<{}>", col.type_info().name()),
                    }
                };

                map.insert(name.to_string(), stringified);
            }

            result.push(map);
        }

        Ok(result)
    }
}
