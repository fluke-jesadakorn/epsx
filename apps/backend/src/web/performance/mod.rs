// Performance monitoring module
// Enterprise-grade performance analytics, alerting, and optimization

pub mod models;
// pub mod middleware;
// pub mod analytics;
// pub mod alerts;
// pub mod repo;
// pub mod handlers;
// pub mod routes;
// pub mod recommendations;
// Stub modules removed as they don't exist

// Re-export main types
pub use models::*;
// pub use middleware::PerformanceMiddleware;
// pub use analytics::PerformanceAnalytics;
// pub use alerts::AlertSystem;
// pub use recommendations::RecommendationEngine;

// use std::sync::Arc;
// Database types updated to Diesel

// /// Performance monitoring service container
// pub struct PerformanceService {
//     pub analytics: Arc<PerformanceAnalytics>,
//     pub alerts: Arc<AlertSystem>,
//     pub recommendations: Arc<RecommendationEngine>,
//     pub repo: Arc<repo::PerformanceRepo>,
// }

// impl PerformanceService {
//     /// Create new performance service with all components
//     pub fn new(db_pool: PgPool) -> Self {
//         let repo = Arc::new(repo::PerformanceRepo::new(db_pool.clone()));
//         let analytics = Arc::new(PerformanceAnalytics::new(repo.clone()));
//         let alerts = Arc::new(AlertSystem::new(repo.clone()));
//         let recommendations = Arc::new(RecommendationEngine::new(repo.clone(), analytics.clone()));

//         Self {
//             analytics,
//             alerts,
//             recommendations,
//             repo,
//         }
//     }

//     /// Get performance middleware with this service
//     pub fn middleware(&self) -> PerformanceMiddleware {
//         PerformanceMiddleware::new(self.repo.clone())
//     }
// }