// kernel extraction wave9 — re-export shim
//
// The canonical `AppError` / `ErrorKind` / `ErrorContext` / `AppResult` etc.
// now live in `epsx_contracts::errors` (the new `epsx-contracts` workspace
// crate). This file is a thin re-export so that the 146 file
// `use crate::core::errors::*` import sites in the backend keep working
// during the wave-10 bulk-rename pass.
//
// In addition, this shim hosts the `From<crate::application::shared::error::ApplicationError>`
// impl that bridges the application-layer error type into the kernel
// `AppError`. That impl depends on a backend-specific application-layer
// type, so it cannot live inside the kernel crate; the shim is the right
// place for it.

pub use epsx_contracts::errors::*;

/// Bridge impl from the application-layer `ApplicationError` to the kernel
/// `AppError`. Required so that `?`-propagation from `ApplicationResult<T>`
/// to `AppResult<T>` keeps working.
impl From<crate::application::shared::error::ApplicationError> for AppError {
    fn from(err: crate::application::shared::error::ApplicationError) -> Self {
        use crate::application::shared::error::ApplicationError;
        match err {
            ApplicationError::Validation { field, message } =>
                AppError::validation_error(format!("Validation failed for {}: {}", field, message)),
            ApplicationError::Authorization { action } =>
                AppError::unauthorized(format!("Authorization failed: {} not allowed", action)),
            ApplicationError::Domain(domain_err) =>
                AppError::business_rule_violation(domain_err.to_string()),
            ApplicationError::NotFound { resource_type, id } =>
                AppError::not_found(format!("{} with id {} not found", resource_type, id)),
            ApplicationError::Infrastructure { message } =>
                AppError::external_service_error(message),
            ApplicationError::Conflict { resource } =>
                AppError::conflict(format!("Conflict: {} already exists", resource)),
            ApplicationError::ExternalService { service, message } =>
                AppError::external_service_error(format!("{}: {}", service, message)),
            ApplicationError::Concurrency { message } =>
                AppError::conflict(format!("Concurrency error: {}", message)),
            ApplicationError::BusinessRule { rule } =>
                AppError::business_rule_violation(format!("Business rule violation: {}", rule)),
            ApplicationError::NotImplemented { feature } =>
                AppError::internal_error(format!("Feature not implemented: {}", feature)),
            ApplicationError::BusinessLogic { message } =>
                AppError::business_rule_violation(format!("Business logic error: {}", message)),
            ApplicationError::Security { message } =>
                AppError::unauthorized(format!("Security error: {}", message)),
            ApplicationError::WalletAddress(wallet_err) =>
                AppError::validation_error(format!("Invalid wallet address: {}", wallet_err)),
            ApplicationError::ValueObject(value_err) =>
                AppError::validation_error(format!("Value object error: {}", value_err)),
        }
    }
}
