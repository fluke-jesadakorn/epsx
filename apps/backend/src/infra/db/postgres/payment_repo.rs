// PostgreSQL Payment Repository Implementation - Diesel-based

// Re-export the Diesel implementation
pub use crate::infra::db::diesel::repos::DieselPaymentRepo as PostgresPaymentRepo;