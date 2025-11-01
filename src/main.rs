use std::env;

use crate::sql::Reader;

mod sql;

#[tokio::main]
async fn main() -> Result<(), anyhow::Error> {
    dotenv::dotenv().ok();

    let database_url = env::var("DATABASE_URL")
        .ok()
        .expect("missing DATABASE_URL env variable!");

    let reader = Reader::postgres(database_url).await.unwrap();

    let schemas = reader.schemas().await.unwrap();
    println!("{:?}", schemas);

    let tables = reader.tables("public".to_string()).await.unwrap();
    println!("{:?}", tables);

    let rows = reader
        .view("public".to_string(), "Game".to_string())
        .await
        .unwrap();
    for row in rows {
        println!("{:?}", row);
    }

    Ok(())
}
