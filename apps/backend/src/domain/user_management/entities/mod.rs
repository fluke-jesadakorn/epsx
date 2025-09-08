// User Management Entities
// Entities are objects with identity that are not aggregates themselves
// In the User Management context, most business logic is in aggregates,
// but we may have supporting entities for complex relationships

// For now, this module is empty as User and Session are aggregates
// Additional entities can be added here as the domain grows

// Example entities that might be added:
// pub mod permission_grant; // For tracking who granted permissions to whom
// pub mod login_attempt;    // For tracking authentication attempts
// pub mod user_profile;     // For extended user information