use crate::{config::Config, error::AppError, services::firebase::FirebaseService};
use mongodb::{Client as MongoClient, Database};
use std::sync::Arc;

#[derive(Clone)]
pub struct AppState {
    pub config: Arc<Config>,
    pub db: Option<Database>,
    pub firebase: FirebaseService,
}

impl AppState {
    pub async fn new(config: Config) -> Result<Self, AppError> {
        // Initialize MongoDB
        let db = if !config.mongodb_uri.is_empty() {
            let mongo_client = MongoClient::with_uri_str(&config.mongodb_uri).await
                .map_err(AppError::Database)?;
            let db_name = config.mongodb_uri
                .split('/')
                .last()
                .unwrap_or("epsx")
                .split('?')
                .next()
                .unwrap_or("epsx");
            
            Some(mongo_client.database(db_name))
        } else {
            None
        };

        // Initialize Firebase
        let firebase = FirebaseService::new(config.firebase_api_key.clone());
        
        Ok(Self {
            config: Arc::new(config),
            db,
            firebase,
        })
    }

    pub fn get_collection<T: Send + Sync + 'static>(&self, name: &str) -> Option<mongodb::Collection<T>> {
        self.db.as_ref().map(|db| db.collection(name))
    }
}
