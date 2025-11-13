// Validation constants: patterns and limits

/// Common validation patterns
pub mod patterns {
    pub const EMAIL_REGEX: &str = r"^[a-zA-Z0-9._%+-]+@[a-zA-Z0-9.-]+\.[a-zA-Z]{2,}$";
    pub const PASSWORD_REGEX: &str = r"^(?=.*[a-z])(?=.*[A-Z])(?=.*\d)(?=.*[@$!%*?&])[A-Za-z\d@$!%*?&]{8,}$";
    pub const PHONE_REGEX: &str = r"^\+?[1-9]\d{1,14}$";
    pub const URL_REGEX: &str = r"^https?://[^\s/$.?#].[^\s]*$";
    pub const UUID_REGEX: &str = r"^[0-9a-f]{8}-[0-9a-f]{4}-[1-5][0-9a-f]{3}-[89ab][0-9a-f]{3}-[0-9a-f]{12}$";
    pub const ALPHANUMERIC_REGEX: &str = r"^[a-zA-Z0-9]+$";
    pub const SLUG_REGEX: &str = r"^[a-z0-9-]+$";
    pub const SAFE_TEXT_REGEX: &str = r"^[a-zA-Z0-9\s\-_.,!?()]+$";
}

/// Validation limits
pub mod limits {
    pub const MAX_EMAIL_LENGTH: usize = 254;
    pub const MAX_PASSWORD_LENGTH: usize = 128;
    pub const MIN_PASSWORD_LENGTH: usize = 8;
    pub const MAX_NAME_LENGTH: usize = 100;
    pub const MAX_DESCRIPTION_LENGTH: usize = 1000;
    pub const MAX_TITLE_LENGTH: usize = 200;
    pub const MAX_JSON_DEPTH: usize = 10;
    pub const MAX_ARRAY_SIZE: usize = 1000;
    pub const MAX_FILE_SIZE: usize = 10 * 1024 * 1024; // 10MB
    pub const MAX_REQUEST_SIZE: usize = 1024 * 1024; // 1MB
}
