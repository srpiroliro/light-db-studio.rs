use std::env;

use crate::{sql::Reader, web::init};

mod sql;
mod web;

#[tokio::main]
async fn main() -> Result<(), anyhow::Error> {
    dotenv::dotenv().ok();

    let database_url = env::var("DATABASE_URL")
        .ok()
        .expect("missing DATABASE_URL env variable!");

    let reader = Reader::postgres(database_url).await.unwrap();

    let _ = init(reader).await;

    Ok(())
}
