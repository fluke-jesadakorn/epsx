use dotenv::dotenv;
use mongodb::{Client, Database};
use std::env;

pub mod models;

#[derive(Debug)]
#[allow(dead_code)]
pub struct DB {
    pub client: Client,
    pub database: Database,
}

impl DB {
    pub fn get_stock_data(&self) -> mongodb::Collection<crate::db::models::StockData> {
        self.database.collection("stock_data")
    }
}

pub async fn connect_db() -> Result<DB, mongodb::error::Error> {
    dotenv().ok();
    
    let mongodb_uri = env::var("MONGODB_URI")
        .expect("MONGODB_URI must be set");
    let database_name = env::var("MONGODB_DB")
        .expect("MONGODB_DB must be set");

    let client = Client::with_uri_str(&mongodb_uri).await?;
    let database = client.database(&database_name);

    Ok(DB {
        client,
        database,
    })
}
