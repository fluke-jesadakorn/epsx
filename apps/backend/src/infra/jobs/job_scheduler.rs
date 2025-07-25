use std::sync::Arc;
use tokio_cron_scheduler::{JobScheduler as CronJobScheduler, Job};
use tracing::{info, error, warn};
use chrono::{DateTime, Utc};
use uuid::Uuid;

use crate::app::ports::repositories::*;
use crate::dom::services::auto_assignment::AutoAssignmentEngine;
use super::{ExpirationChecker, NotificationService};

pub struct JobScheduler {
    scheduler: CronJobScheduler,
    permission_profile_repo: Arc<dyn PermissionProfileRepo>,
    user_repo: Arc<dyn UserRepo>,
    audit_repo: Arc<dyn AuditRepo>,
    auto_assignment_service: Arc<AutoAssignmentEngine>,
    expiration_checker: Arc<ExpirationChecker>,
    notification_service: Arc<NotificationService>,
}

impl JobScheduler {
    pub async fn new(
        permission_profile_repo: Arc<dyn PermissionProfileRepo>,
        user_repo: Arc<dyn UserRepo>,
        audit_repo: Arc<dyn AuditRepo>,
        auto_assignment_service: Arc<AutoAssignmentEngine>,
        expiration_checker: Arc<ExpirationChecker>,
        notification_service: Arc<NotificationService>,
    ) -> Result<Self, JobSchedulerError> {
        let scheduler = CronJobScheduler::new().await.map_err(JobSchedulerError::InitializationFailed)?;
        
        Ok(Self {
            scheduler,
            permission_profile_repo,
            user_repo,
            audit_repo,
            auto_assignment_service,
            expiration_checker,
            notification_service,
        })
    }

    pub async fn start(&mut self) -> Result<(), JobSchedulerError> {
        info!("Starting job scheduler with background jobs");

        // Job 1: Check for feature expirations every hour
        self.schedule_expiration_check().await?;
        
        // Job 2: Auto-assign permission profiles to new users every 30 minutes
        self.schedule_auto_assignment().await?;
        
        // Job 3: Clean up expired permission assignments daily at midnight
        self.schedule_cleanup().await?;
        
        // Job 4: Generate analytics reports daily at 2 AM
        self.schedule_analytics_reports().await?;
        
        // Job 5: Health check and monitoring every 15 minutes
        self.schedule_health_check().await?;

        self.scheduler.start().await.map_err(JobSchedulerError::StartFailed)?;
        info!("Job scheduler started successfully");
        
        Ok(())
    }

    pub async fn stop(&self) -> Result<(), JobSchedulerError> {
        info!("Stopping job scheduler");
        self.scheduler.shutdown().await.map_err(JobSchedulerError::StopFailed)?;
        info!("Job scheduler stopped");
        Ok(())
    }

    pub async fn add_one_time_job<F>(&self, task: F, delay_seconds: u64) -> Result<Uuid, JobSchedulerError>
    where
        F: Fn() -> Result<(), String> + Send + Sync + 'static,
    {
        let job_id = Uuid::new_v4();
        let delay = chrono::Duration::seconds(delay_seconds as i64);
        let run_time = Utc::now() + delay;
        
        let job = Job::new_one_shot_async(run_time, move |_uuid, _lock| {
            Box::pin(async move {
                match task() {
                    Ok(()) => info!("One-time job completed successfully: {}", _uuid),
                    Err(e) => error!("One-time job failed: {}: {}", _uuid, e),
                }
            })
        }).map_err(JobSchedulerError::JobCreationFailed)?;

        self.scheduler.add(job).await.map_err(JobSchedulerError::JobAddFailed)?;
        
        info!("Added one-time job: {} scheduled for: {}", job_id, run_time);
        Ok(job_id)
    }

    async fn schedule_expiration_check(&self) -> Result<(), JobSchedulerError> {
        let expiration_checker = Arc::clone(&self.expiration_checker);
        let notification_service = Arc::clone(&self.notification_service);
        
        let job = Job::new_async("0 0 * * * *", move |_uuid, _lock| { // Every hour
            let checker = Arc::clone(&expiration_checker);
            let notifier = Arc::clone(&notification_service);
            
            Box::pin(async move {
                info!("Starting scheduled expiration check");
                
                match checker.check_and_handle_expirations().await {
                    Ok(results) => {
                        info!("Expiration check completed. Processed {} items", results.processed_count);
                        
                        // Send summary notification if there were expirations
                        if results.expired_count > 0 {
                            let summary = format!(
                                "Expiration Check Summary: {} items processed, {} expired, {} notified", 
                                results.processed_count, results.expired_count, results.notification_count
                            );
                            
                            if let Err(e) = notifier.send_admin_notification("Expiration Check Summary", &summary).await {
                                error!("Failed to send admin notification: {}", e);
                            }
                        }
                    },
                    Err(e) => {
                        error!("Expiration check failed: {}", e);
                        
                        // Send error notification to admin
                        if let Err(ne) = notifier.send_admin_notification(
                            "Expiration Check Failed", 
                            &format!("Error during expiration check: {}", e)
                        ).await {
                            error!("Failed to send error notification: {}", ne);
                        }
                    }
                }
            })
        }).map_err(JobSchedulerError::JobCreationFailed)?;

        self.scheduler.add(job).await.map_err(JobSchedulerError::JobAddFailed)?;
        info!("Scheduled expiration check job (hourly)");
        Ok(())
    }

    async fn schedule_auto_assignment(&self) -> Result<(), JobSchedulerError> {
        let auto_assignment = Arc::clone(&self.auto_assignment_service);
        let user_repo = Arc::clone(&self.user_repo);
        let audit_repo = Arc::clone(&self.audit_repo);
        
        let job = Job::new_async("0 */30 * * * *", move |_uuid, _lock| { // Every 30 minutes
            let assignment_service = Arc::clone(&auto_assignment);
            let user_repository = Arc::clone(&user_repo);
            let audit_repository = Arc::clone(&audit_repo);
            
            Box::pin(async move {
                info!("Starting scheduled auto-assignment check");
                
                // Get users who may need permission profile assignments
                match user_repository.find_users_for_auto_assignment().await {
                    Ok(users) => {
                        let mut assigned_count = 0;
                        let mut error_count = 0;
                        
                        for user in users {
                            match assignment_service.evaluate_and_assign(&user).await {
                                Ok(assignments) => {
                                    assigned_count += assignments.len();
                                    
                                    // Log assignment for audit
                                    for assignment in assignments {
                                        if let Err(e) = audit_repository.log_permission_assignment(
                                            &user.id, 
                                            &assignment.permission_profile_id, 
                                            "auto_assignment_job", 
                                            &assignment.reason
                                        ).await {
                                            error!("Failed to log auto assignment: {}", e);
                                        }
                                    }
                                },
                                Err(e) => {
                                    error_count += 1;
                                    error!("Auto assignment failed for user {}: {}", user.id, e);
                                }
                            }
                        }
                        
                        info!("Auto-assignment completed: {} assignments made, {} errors", assigned_count, error_count);
                    },
                    Err(e) => error!("Failed to get users for auto-assignment: {}", e)
                }
            })
        }).map_err(JobSchedulerError::JobCreationFailed)?;

        self.scheduler.add(job).await.map_err(JobSchedulerError::JobAddFailed)?;
        info!("Scheduled auto-assignment job (every 30 minutes)");
        Ok(())
    }

    async fn schedule_cleanup(&self) -> Result<(), JobSchedulerError> {
        let permission_repo = Arc::clone(&self.permission_profile_repo);
        let audit_repo = Arc::clone(&self.audit_repo);
        
        let job = Job::new_async("0 0 0 * * *", move |_uuid, _lock| { // Daily at midnight
            let permission_repository = Arc::clone(&permission_repo);
            let audit_repository = Arc::clone(&audit_repo);
            
            Box::pin(async move {
                info!("Starting scheduled cleanup job");
                
                let mut cleanup_summary = CleanupSummary::default();
                
                // Clean up expired permission assignments
                match permission_repository.cleanup_expired_assignments().await {
                    Ok(count) => {
                        cleanup_summary.expired_assignments_cleaned = count;
                        info!("Cleaned up {} expired permission assignments", count);
                    },
                    Err(e) => {
                        cleanup_summary.errors.push(format!("Failed to cleanup expired assignments: {}", e));
                        error!("Failed to cleanup expired assignments: {}", e);
                    }
                }
                
                // Clean up old audit logs (keep last 90 days)
                match audit_repository.cleanup_old_logs(90).await {
                    Ok(count) => {
                        cleanup_summary.old_logs_cleaned = count;
                        info!("Cleaned up {} old audit logs", count);
                    },
                    Err(e) => {
                        cleanup_summary.errors.push(format!("Failed to cleanup old logs: {}", e));
                        error!("Failed to cleanup old audit logs: {}", e);
                    }
                }
                
                // Log cleanup summary
                if let Err(e) = audit_repository.log_system_event(
                    "scheduled_cleanup", 
                    &serde_json::to_string(&cleanup_summary).unwrap_or_default()
                ).await {
                    error!("Failed to log cleanup summary: {}", e);
                }
                
                info!("Cleanup job completed: {} expired assignments, {} old logs cleaned", 
                      cleanup_summary.expired_assignments_cleaned, cleanup_summary.old_logs_cleaned);
            })
        }).map_err(JobSchedulerError::JobCreationFailed)?;

        self.scheduler.add(job).await.map_err(JobSchedulerError::JobAddFailed)?;
        info!("Scheduled cleanup job (daily at midnight)");
        Ok(())
    }

    async fn schedule_analytics_reports(&self) -> Result<(), JobSchedulerError> {
        let permission_repo = Arc::clone(&self.permission_profile_repo);
        let user_repo = Arc::clone(&self.user_repo);
        let notification_service = Arc::clone(&self.notification_service);
        
        let job = Job::new_async("0 0 2 * * *", move |_uuid, _lock| { // Daily at 2 AM
            let permission_repository = Arc::clone(&permission_repo);
            let user_repository = Arc::clone(&user_repo);
            let notifier = Arc::clone(&notification_service);
            
            Box::pin(async move {
                info!("Starting scheduled analytics report generation");
                
                match generate_daily_analytics_report(&*permission_repository, &*user_repository).await {
                    Ok(report) => {
                        info!("Analytics report generated successfully");
                        
                        // Send report to admin
                        if let Err(e) = notifier.send_admin_notification(
                            "Daily Analytics Report", 
                            &report
                        ).await {
                            error!("Failed to send analytics report: {}", e);
                        }
                    },
                    Err(e) => error!("Failed to generate analytics report: {}", e)
                }
            })
        }).map_err(JobSchedulerError::JobCreationFailed)?;

        self.scheduler.add(job).await.map_err(JobSchedulerError::JobAddFailed)?;
        info!("Scheduled analytics report job (daily at 2 AM)");
        Ok(())
    }

    async fn schedule_health_check(&self) -> Result<(), JobSchedulerError> {
        let permission_repo = Arc::clone(&self.permission_profile_repo);
        let user_repo = Arc::clone(&self.user_repo);
        let audit_repo = Arc::clone(&self.audit_repo);
        
        let job = Job::new_async("0 */15 * * * *", move |_uuid, _lock| { // Every 15 minutes
            let permission_repository = Arc::clone(&permission_repo);
            let user_repository = Arc::clone(&user_repo);
            let audit_repository = Arc::clone(&audit_repo);
            
            Box::pin(async move {
                let mut health_issues = Vec::new();
                
                // Check permission repository health
                if let Err(e) = permission_repository.health_check().await {
                    health_issues.push(format!("Permission repository unhealthy: {}", e));
                }
                
                // Check user repository health
                if let Err(e) = user_repository.health_check().await {
                    health_issues.push(format!("User repository unhealthy: {}", e));
                }
                
                // Check audit repository health
                if let Err(e) = audit_repository.health_check().await {
                    health_issues.push(format!("Audit repository unhealthy: {}", e));
                }
                
                if health_issues.is_empty() {
                    // Log successful health check periodically (every hour)
                    if Utc::now().minute() % 60 == 0 {
                        info!("System health check: All services healthy");
                    }
                } else {
                    for issue in &health_issues {
                        error!("Health check issue: {}", issue);
                    }
                    
                    // Log health issues
                    if let Err(e) = audit_repository.log_system_event(
                        "health_check_issues", 
                        &health_issues.join("; ")
                    ).await {
                        error!("Failed to log health check issues: {}", e);
                    }
                }
            })
        }).map_err(JobSchedulerError::JobCreationFailed)?;

        self.scheduler.add(job).await.map_err(JobSchedulerError::JobAddFailed)?;
        info!("Scheduled health check job (every 15 minutes)");
        Ok(())
    }
}

#[derive(Debug, serde::Serialize, Default)]
struct CleanupSummary {
    expired_assignments_cleaned: i64,
    old_logs_cleaned: i64,
    errors: Vec<String>,
}

async fn generate_daily_analytics_report(
    permission_repo: &dyn PermissionProfileRepo,
    user_repo: &dyn UserRepo,
) -> Result<String, String> {
    let total_users = user_repo.count_total_users().await.map_err(|e| e.to_string())?;
    let active_profiles = permission_repo.count_active_profiles().await.map_err(|e| e.to_string())?;
    let total_assignments = permission_repo.count_total_assignments().await.map_err(|e| e.to_string())?;
    
    Ok(format!(
        "Daily Analytics Report - {}\n\
        Total Users: {}\n\
        Active Permission Profiles: {}\n\
        Total Permission Assignments: {}\n\
        \n\
        System Status: Operational\n\
        Report Generated: {}",
        Utc::now().format("%Y-%m-%d"),
        total_users,
        active_profiles,
        total_assignments,
        Utc::now().format("%Y-%m-%d %H:%M:%S UTC")
    ))
}

#[derive(Debug, thiserror::Error)]
pub enum JobSchedulerError {
    #[error("Failed to initialize job scheduler: {0}")]
    InitializationFailed(#[from] tokio_cron_scheduler::JobSchedulerError),
    
    #[error("Failed to start job scheduler: {0}")]
    StartFailed(tokio_cron_scheduler::JobSchedulerError),
    
    #[error("Failed to stop job scheduler: {0}")]
    StopFailed(tokio_cron_scheduler::JobSchedulerError),
    
    #[error("Failed to create job: {0}")]
    JobCreationFailed(tokio_cron_scheduler::JobSchedulerError),
    
    #[error("Failed to add job: {0}")]
    JobAddFailed(tokio_cron_scheduler::JobSchedulerError),
    
    #[error("Configuration error: {0}")]
    ConfigurationError(String),
}