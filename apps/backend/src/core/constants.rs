// Core application constants
// Moved from hardcoded values to improve maintainability

use std::time::Duration;

// ============================================================================
// TIME CONSTANTS (in seconds)
// ============================================================================

/// 1 minute in seconds
pub const MINUTE: i64 = 60;

/// 1 hour in seconds
pub const HOUR: i64 = 3600;

/// 1 day in seconds  
pub const DAY: i64 = 86400;

/// 1 week in seconds
pub const WEEK: i64 = 7 * DAY;

/// 1 month in seconds (30 days)
pub const MONTH: i64 = 30 * DAY;

/// 1 year in seconds (365 days)
pub const YEAR: i64 = 365 * DAY;

// ============================================================================
// DURATION CONSTANTS
// ============================================================================

/// 1 hour as Duration
pub const ONE_HOUR: Duration = Duration::from_secs(HOUR as u64);

/// 1 day as Duration
pub const ONE_DAY: Duration = Duration::from_secs(DAY as u64);

/// 1 week as Duration
pub const ONE_WEEK: Duration = Duration::from_secs(WEEK as u64);

// ============================================================================
// SESSION & AUTHENTICATION CONSTANTS
// ============================================================================

/// Default session timeout (1 hour)
pub const DEFAULT_SESSION_TIMEOUT: i64 = HOUR;

/// Maximum session age (24 hours)
pub const MAX_SESSION_AGE: i64 = DAY;

/// Session renewal threshold (1 hour before expiry)
pub const SESSION_RENEWAL_THRESHOLD: i64 = HOUR;

/// Token expiry time (1 hour)
pub const DEFAULT_TOKEN_EXPIRY: i64 = HOUR;

/// Maximum token validity (10 years)
pub const MAX_TOKEN_VALIDITY: i64 = 10 * YEAR;

// ============================================================================
// CACHE CONSTANTS
// ============================================================================

/// Default cache TTL (1 hour)
pub const DEFAULT_CACHE_TTL: u64 = HOUR as u64;

/// Long cache TTL (24 hours)
pub const LONG_CACHE_TTL: u64 = DAY as u64;

/// EPS cache TTL (1 hour)
pub const EPS_CACHE_TTL: u64 = HOUR as u64;

// ============================================================================
// CORS & SECURITY CONSTANTS
// ============================================================================

/// CORS max age for production (24 hours)
pub const CORS_MAX_AGE_PRODUCTION: u64 = DAY as u64;

/// CORS max age for development (1 hour)
pub const CORS_MAX_AGE_DEVELOPMENT: u64 = HOUR as u64;

/// HSTS max age (1 year)
pub const HSTS_MAX_AGE: u64 = YEAR as u64;

// ============================================================================
// RATE LIMITING CONSTANTS
// ============================================================================

/// Rate limit window for hourly limits
pub const RATE_LIMIT_HOUR_WINDOW: i64 = HOUR;

/// Rate limit window for daily limits
pub const RATE_LIMIT_DAY_WINDOW: i64 = DAY;

/// Minimum recovery time (24 hours)
pub const MIN_RECOVERY_TIME: i64 = DAY;

/// Recovery time hours for rate limiting
pub const RECOVERY_TIME_HOURS: u32 = 24;

// ============================================================================
// PERMISSION & USER LIMIT CONSTANTS  
// ============================================================================

/// Hours in a day for limit calculations
pub const HOURS_PER_DAY: i32 = 24;

/// Minutes in a day for limit calculations
pub const MINUTES_PER_DAY: i32 = 1440; // 24 * 60

/// Days in a week for weekly limits
pub const DAYS_PER_WEEK: i32 = 7;

/// Days in a month for monthly limits (30 days)
pub const DAYS_PER_MONTH: i32 = 30;

/// Days in a year for yearly limits
pub const DAYS_PER_YEAR: i32 = 365;

/// Expiry warning threshold (24 hours)
pub const EXPIRES_SOON_THRESHOLD_HOURS: i64 = 24;

// ============================================================================
// BUSINESS LOGIC CONSTANTS
// ============================================================================

/// Maximum country name length
pub const MAX_COUNTRY_NAME_LENGTH: usize = 100;

/// Maximum market sector name length
pub const MAX_MARKET_SECTOR_LENGTH: usize = 100;

/// Percentage multiplier for calculations
pub const PERCENTAGE_MULTIPLIER: f64 = 100.0;

/// Initial delay for error recovery (100ms)
pub const INITIAL_RECOVERY_DELAY_MS: u64 = 100;

/// Cleanup interval for refresh tokens (1 hour)
pub const REFRESH_TOKEN_CLEANUP_INTERVAL: Duration = ONE_HOUR;

// ============================================================================
// DATA ANALYSIS CONSTANTS
// ============================================================================

/// Time window for finding prices near announcement (1 day before/after)
pub const PRICE_SEARCH_WINDOW: i64 = DAY;

/// Default notification delay (1 hour)
pub const DEFAULT_NOTIFICATION_DELAY: u64 = HOUR as u64;