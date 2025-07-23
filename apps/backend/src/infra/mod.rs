// Infrastructure layer implementations

pub mod auth;
pub mod db;
pub mod repos;
pub mod services;
pub mod events;

// Re-export commonly used implementations
pub use auth::{FbAuthSvcImpl};
pub use db::{FsUserRepo, FsSessRepo, FsPayRepo, InMemoryLevelHistoryRepo};
pub use repos::{IamRepoImpl, AuditRepoImpl, TemplateRepoImpl};
pub use events::{SimpleEventDispatcher};

use std::sync::Arc;
use crate::app::ports::{repositories::*, services::*, events::*};

/// Factory for creating infrastructure implementations
pub struct InfraFactory {
    pub firestore_db: firestore::FirestoreDb,
    pub firebase_project_id: String,
}

impl InfraFactory {
    pub async fn new(
        firestore_project_id: String,
        firebase_project_id: String,
    ) -> Result<Self, Box<dyn std::error::Error>> {
        let firestore_db = firestore::FirestoreDb::new(&firestore_project_id).await?;
        
        Ok(Self {
            firestore_db,
            firebase_project_id,
        })
    }

    pub fn from_env() -> Result<Self, Box<dyn std::error::Error>> {
        let firestore_project_id = std::env::var("FIRESTORE_PROJECT_ID")
            .or_else(|_| std::env::var("FIREBASE_PROJECT_ID"))
            .map_err(|_| "FIRESTORE_PROJECT_ID or FIREBASE_PROJECT_ID must be set")?;
        
        let firebase_project_id = std::env::var("FIREBASE_PROJECT_ID")
            .map_err(|_| "FIREBASE_PROJECT_ID must be set")?;
        
        // Note: This is a simplified version - in practice you'd handle async properly
        tokio::task::block_in_place(|| {
            tokio::runtime::Handle::current().block_on(
                Self::new(firestore_project_id, firebase_project_id)
            )
        })
    }

    // Repository factories
    pub fn create_user_repo(&self) -> Arc<dyn UserRepo> {
        Arc::new(FsUserRepo::new(self.firestore_db.clone()))
    }

    pub fn create_session_repo(&self) -> Arc<dyn SessRepo> {
        Arc::new(FsSessRepo::new(self.firestore_db.clone()))
    }

    pub fn create_payment_repo(&self) -> Arc<dyn PayRepo> {
        Arc::new(FsPayRepo::new(self.firestore_db.clone()))
    }

    pub fn create_stock_repo(&self) -> Arc<dyn StockRepo> {
        // TODO: Implement StockRepo when needed
        unimplemented!("StockRepo not yet implemented")
    }

    pub fn create_level_history_repo(&self) -> Arc<dyn LevelHistoryRepo> {
        Arc::new(InMemoryLevelHistoryRepo::new())
    }

    pub fn create_iam_repo(&self) -> Arc<dyn IamRepo> {
        Arc::new(IamRepoImpl::new())
    }

    pub fn create_audit_repo(&self) -> Arc<dyn AuditRepo> {
        Arc::new(AuditRepoImpl::new())
    }

    pub fn create_template_repo(&self) -> Arc<dyn TemplateRepo> {
        Arc::new(TemplateRepoImpl::new())
    }

    // Service factories
    pub fn create_firebase_auth_svc(&self) -> Arc<dyn FbAuthSvc> {
        // Use from_env to get the proper credentials
        match FbAuthSvcImpl::from_env() {
            Ok(service) => Arc::new(service),
            Err(_) => {
                // Fallback to basic service without credentials for development
                Arc::new(FbAuthSvcImpl::new(
                    self.firebase_project_id.clone(), 
                    None, 
                    None
                ))
            }
        }
    }

    pub fn create_email_svc(&self) -> Arc<dyn EmailSvc> {
        // TODO: Implement EmailSvc when needed
        unimplemented!("EmailSvc not yet implemented")
    }

    pub fn create_payment_gateway(&self) -> Arc<dyn PayGw> {
        // TODO: Implement PayGw when needed
        unimplemented!("PayGw not yet implemented")
    }

    pub fn create_stock_data_svc(&self) -> Arc<dyn StockDataSvc> {
        // TODO: Implement StockDataSvc when needed
        unimplemented!("StockDataSvc not yet implemented")
    }

    pub fn create_websocket_svc(&self) -> Arc<dyn WebSocketSvc> {
        // TODO: Implement WebSocketSvc when needed
        unimplemented!("WebSocketSvc not yet implemented")
    }

    // Event services
    pub fn create_event_dispatcher(&self) -> Arc<dyn EventDispatcher> {
        Arc::new(SimpleEventDispatcher::new())
    }
}

/// Application-wide dependency injection container
pub struct AppContainer {
    pub infra: InfraFactory,
    
    // Repositories
    pub user_repo: Arc<dyn UserRepo>,
    pub session_repo: Arc<dyn SessRepo>,
    pub payment_repo: Arc<dyn PayRepo>,
    pub level_history_repo: Arc<dyn LevelHistoryRepo>,
    pub iam_repo: Arc<dyn IamRepo>,
    pub audit_repo: Arc<dyn AuditRepo>,
    pub template_repo: Arc<dyn TemplateRepo>,
    
    // Services  
    pub firebase_auth_svc: Arc<dyn FbAuthSvc>,
    pub event_dispatcher: Arc<dyn EventDispatcher>,
}

impl AppContainer {
    pub async fn new() -> Result<Self, Box<dyn std::error::Error>> {
        let infra = InfraFactory::from_env()?;
        
        let user_repo = infra.create_user_repo();
        let session_repo = infra.create_session_repo();
        let payment_repo = infra.create_payment_repo();
        let level_history_repo = infra.create_level_history_repo();
        let iam_repo = infra.create_iam_repo();
        let audit_repo = infra.create_audit_repo();
        let template_repo = infra.create_template_repo();
        let firebase_auth_svc = infra.create_firebase_auth_svc();
        let event_dispatcher = infra.create_event_dispatcher();
        
        Ok(Self {
            infra,
            user_repo,
            session_repo,
            payment_repo,
            level_history_repo,
            iam_repo,
            audit_repo,
            template_repo,
            firebase_auth_svc,
            event_dispatcher,
        })
    }
    
    pub fn from_infra(infra: InfraFactory) -> Self {
        let user_repo = infra.create_user_repo();
        let session_repo = infra.create_session_repo();
        let payment_repo = infra.create_payment_repo();
        let level_history_repo = infra.create_level_history_repo();
        let iam_repo = infra.create_iam_repo();
        let audit_repo = infra.create_audit_repo();
        let template_repo = infra.create_template_repo();
        let firebase_auth_svc = infra.create_firebase_auth_svc();
        let event_dispatcher = infra.create_event_dispatcher();
        
        Self {
            infra,
            user_repo,
            session_repo,
            payment_repo,
            level_history_repo,
            iam_repo,
            audit_repo,
            template_repo,
            firebase_auth_svc,
            event_dispatcher,
        }
    }
}