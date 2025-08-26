// Audit service for managing audit logging throughout the application

use std::sync::Arc;
use std::collections::HashMap;
use std::time::Instant;

use crate::app::ports::repositories::AuditRepository;
use crate::dom::entities::audit::{
  AuditLogEntry,
  AuditAction,
  ResourceType,
  AuditResult,
  AuditMetadata,
  AuditError,
};
use crate::dom::values::UserId;

/// Audit service for recording security and compliance events
pub struct AuditService {
  audit_repo: Arc<dyn AuditRepository>,
}

/// Context information for audit logging
pub struct AuditContext {
  pub actor_id: UserId,
  pub client_ip: Option<String>,
  pub user_agent: Option<String>,
  pub session_id: Option<String>,
}

/// Builder for creating audit log entries with timing
pub struct AuditLogBuilder {
  context: AuditContext,
  action: AuditAction,
  resource_type: ResourceType,
  resource_id: String,
  start_time: Instant,
  metadata_builder: MetadataBuilder,
}

/// Builder for audit metadata
pub struct MetadataBuilder {
  previous_values: Option<HashMap<String, String>>,
  new_values: Option<HashMap<String, String>>,
  error_message: Option<String>,
  additional_data: HashMap<String, String>,
  affected_count: Option<u32>,
}

impl AuditService {
  pub fn new(audit_repo: Arc<dyn AuditRepository>) -> Self {
    Self { audit_repo }
  }

  /// Create a new audit log builder for a specific operation
  pub fn log(&self, context: AuditContext) -> AuditServiceBuilder {
    AuditServiceBuilder {
      service: self,
      context,
    }
  }

  /// Log a simple successful operation
  pub async fn log_success(
    &self,
    context: AuditContext,
    action: AuditAction,
    resource_type: ResourceType,
    resource_id: String
  ) -> Result<(), AuditError> {
    let entry = AuditLogEntry::new(
      context.actor_id,
      action,
      resource_type,
      resource_id,
      AuditResult::Success
    )
      .with_client_info(context.client_ip, context.user_agent)
      .with_session_id(context.session_id.unwrap_or_default());

    self.audit_repo.store(&entry).await
  }

  /// Log a failed operation with error message
  pub async fn log_failure(
    &self,
    context: AuditContext,
    action: AuditAction,
    resource_type: ResourceType,
    resource_id: String,
    error_message: String
  ) -> Result<(), AuditError> {
    let metadata = AuditMetadata::with_error(error_message);

    let entry = AuditLogEntry::new(
      context.actor_id,
      action,
      resource_type,
      resource_id,
      AuditResult::Failure
    )
      .with_metadata(metadata)
      .with_client_info(context.client_ip, context.user_agent)
      .with_session_id(context.session_id.unwrap_or_default());

    self.audit_repo.store(&entry).await
  }

  /// Log permission evaluation (for compliance)
  pub async fn log_permission_check(
    &self,
    context: AuditContext,
    action: &str,
    resource: &str,
    granted: bool
  ) -> Result<(), AuditError> {
    let result = if granted {
      AuditResult::Success
    } else {
      AuditResult::Denied
    };

    let metadata = AuditMetadata::empty()
      .add_data("requested_action".to_string(), action.to_string())
      .add_data("requested_resource".to_string(), resource.to_string());

    let entry = AuditLogEntry::new(
      context.actor_id,
      AuditAction::PermissionEvaluated,
      ResourceType::Permission,
      format!("{}:{}", action, resource),
      result
    )
      .with_metadata(metadata)
      .with_client_info(context.client_ip, context.user_agent)
      .with_session_id(context.session_id.unwrap_or_default());

    self.audit_repo.store(&entry).await
  }

  /// Log authentication events
  pub async fn log_authentication(
    &self,
    context: AuditContext,
    success: bool,
    login_type: Option<String>
  ) -> Result<(), AuditError> {
    let (action, result) = if success {
      (AuditAction::Login, AuditResult::Success)
    } else {
      (AuditAction::LoginFailed, AuditResult::Failure)
    };

    let mut metadata = AuditMetadata::empty();
    if let Some(login_type) = login_type {
      metadata = metadata.add_data("login_type".to_string(), login_type);
    }

    let entry = AuditLogEntry::new(
      context.actor_id.clone(),
      action,
      ResourceType::Session,
      context.actor_id.to_string(),
      result
    )
      .with_metadata(metadata)
      .with_client_info(context.client_ip, context.user_agent)
      .with_session_id(context.session_id.unwrap_or_default());

    self.audit_repo.store(&entry).await
  }

  /// Log bulk operations with affected count
  pub async fn log_bulk_operation(
    &self,
    context: AuditContext,
    action: AuditAction,
    resource_type: ResourceType,
    affected_count: u32,
    success: bool
  ) -> Result<(), AuditError> {
    let result = if success {
      AuditResult::Success
    } else {
      AuditResult::PartialSuccess
    };

    let metadata = AuditMetadata::with_bulk_count(affected_count);

    let entry = AuditLogEntry::new(
      context.actor_id,
      action,
      resource_type,
      "bulk_operation".to_string(),
      result
    )
      .with_metadata(metadata)
      .with_client_info(context.client_ip, context.user_agent)
      .with_session_id(context.session_id.unwrap_or_default());

    self.audit_repo.store(&entry).await
  }
}

/// Builder for creating audit logs with fluent interface
#[allow(dead_code)]
pub struct AuditServiceBuilder<'a> {
  service: &'a AuditService,
  context: AuditContext,
}

impl<'a> AuditServiceBuilder<'a> {
  pub fn action(
    self,
    action: AuditAction,
    resource_type: ResourceType,
    resource_id: String
  ) -> AuditLogBuilder {
    AuditLogBuilder {
      context: self.context,
      action,
      resource_type,
      resource_id,
      start_time: Instant::now(),
      metadata_builder: MetadataBuilder::new(),
    }
  }
}

impl AuditLogBuilder {
  /// Set previous values for update operations
  pub fn with_previous_values(
    mut self,
    previous: HashMap<String, String>
  ) -> Self {
    self.metadata_builder.previous_values = Some(previous);
    self
  }

  /// Set new values for update operations
  pub fn with_new_values(mut self, new: HashMap<String, String>) -> Self {
    self.metadata_builder.new_values = Some(new);
    self
  }

  /// Add additional context data
  pub fn with_data(mut self, key: String, value: String) -> Self {
    self.metadata_builder.additional_data.insert(key, value);
    self
  }

  /// Set affected count for bulk operations
  pub fn with_affected_count(mut self, count: u32) -> Self {
    self.metadata_builder.affected_count = Some(count);
    self
  }

  /// Record successful operation
  pub async fn success(self, service: &AuditService) -> Result<(), AuditError> {
    let duration_ms = self.start_time.elapsed().as_millis() as u64;
    let metadata = self.metadata_builder.build().with_duration(duration_ms);

    let entry = AuditLogEntry::new(
      self.context.actor_id,
      self.action,
      self.resource_type,
      self.resource_id,
      AuditResult::Success
    )
      .with_metadata(metadata)
      .with_client_info(self.context.client_ip, self.context.user_agent)
      .with_session_id(self.context.session_id.unwrap_or_default());

    service.audit_repo.store(&entry).await
  }

  /// Record failed operation
  pub async fn failure(
    mut self,
    service: &AuditService,
    error_message: String
  ) -> Result<(), AuditError> {
    let duration_ms = self.start_time.elapsed().as_millis() as u64;
    self.metadata_builder.error_message = Some(error_message);
    let metadata = self.metadata_builder.build().with_duration(duration_ms);

    let entry = AuditLogEntry::new(
      self.context.actor_id,
      self.action,
      self.resource_type,
      self.resource_id,
      AuditResult::Failure
    )
      .with_metadata(metadata)
      .with_client_info(self.context.client_ip, self.context.user_agent)
      .with_session_id(self.context.session_id.unwrap_or_default());

    service.audit_repo.store(&entry).await
  }

  /// Record denied operation (permission denied)
  pub async fn denied(
    self,
    service: &AuditService,
    reason: Option<String>
  ) -> Result<(), AuditError> {
    let duration_ms = self.start_time.elapsed().as_millis() as u64;
    let mut metadata_builder = self.metadata_builder;

    if let Some(reason) = reason {
      metadata_builder.additional_data.insert(
        "denial_reason".to_string(),
        reason
      );
    }

    let metadata = metadata_builder.build().with_duration(duration_ms);

    let entry = AuditLogEntry::new(
      self.context.actor_id,
      self.action,
      self.resource_type,
      self.resource_id,
      AuditResult::Denied
    )
      .with_metadata(metadata)
      .with_client_info(self.context.client_ip, self.context.user_agent)
      .with_session_id(self.context.session_id.unwrap_or_default());

    service.audit_repo.store(&entry).await
  }
}

impl MetadataBuilder {
  pub fn new() -> Self {
    Self {
      previous_values: None,
      new_values: None,
      error_message: None,
      additional_data: HashMap::new(),
      affected_count: None,
    }
  }

  pub fn build(self) -> AuditMetadata {
    AuditMetadata {
      previous_values: self.previous_values,
      new_values: self.new_values,
      error_message: self.error_message,
      additional_data: self.additional_data,
      affected_count: self.affected_count,
      duration_ms: None, // Set by the log builder
    }
  }
}

impl AuditContext {
  pub fn new(actor_id: UserId) -> Self {
    Self {
      actor_id,
      client_ip: None,
      user_agent: None,
      session_id: None,
    }
  }

  pub fn with_client_info(
    mut self,
    client_ip: Option<String>,
    user_agent: Option<String>
  ) -> Self {
    self.client_ip = client_ip;
    self.user_agent = user_agent;
    self
  }

  pub fn with_session_id(mut self, session_id: String) -> Self {
    self.session_id = Some(session_id);
    self
  }
}
