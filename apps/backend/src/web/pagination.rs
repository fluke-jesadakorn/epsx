use serde::{Deserialize, Serialize};
use utoipa::{ToSchema, IntoParams};

/// Reusable pagination parameters extracted from query params
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Pagination {
    pub page: u32,
    pub limit: u32,
    pub offset: i64,
}

impl Pagination {
    /// Create pagination from optional params with defaults
    /// - page defaults to 1
    /// - limit defaults to `default_limit`, capped at `max_limit`
    pub fn new(page: Option<u32>, limit: Option<u32>, default_limit: u32, max_limit: u32) -> Self {
        let page = page.unwrap_or(1).max(1);
        let limit = limit.unwrap_or(default_limit).min(max_limit);
        let offset = ((page - 1) * limit) as i64;
        Self { page, limit, offset }
    }

    /// Standard pagination: default 20, max 100
    pub fn standard(page: Option<u32>, limit: Option<u32>) -> Self {
        Self::new(page, limit, 20, 100)
    }

    /// Standard pagination from signed integer params (common in query structs)
    pub fn from_signed(page: Option<impl Into<i64>>, limit: Option<impl Into<i64>>, default_limit: u32, max_limit: u32) -> Self {
        Self::new(
            page.map(|p| p.into().max(0) as u32),
            limit.map(|l| l.into().max(0) as u32),
            default_limit,
            max_limit,
        )
    }

    /// Small pagination: default 10, max 50
    pub fn small(page: Option<u32>, limit: Option<u32>) -> Self {
        Self::new(page, limit, 10, 50)
    }

    /// Large pagination: default 50, max 1000
    pub fn large(page: Option<u32>, limit: Option<u32>) -> Self {
        Self::new(page, limit, 50, 1000)
    }

    /// Calculate total pages from total count
    pub fn total_pages(&self, total: u64) -> u32 {
        ((total as f64 / self.limit as f64).ceil() as u32).max(1)
    }

    pub fn has_next(&self, total: u64) -> bool {
        (self.page as u64) < self.total_pages(total) as u64
    }

    pub fn has_prev(&self) -> bool {
        self.page > 1
    }
}

/// Shared query params extractor for paginated endpoints.
/// Use with `Query<PaginationQuery>` in handler signatures.
#[derive(Debug, Clone, Deserialize, ToSchema, IntoParams)]
pub struct PaginationQuery {
    #[param(example = 1)]
    pub page: Option<u32>,
    #[param(example = 20)]
    pub limit: Option<u32>,
}

impl PaginationQuery {
    /// Convert to Pagination with standard defaults (20, max 100)
    pub fn standard(&self) -> Pagination {
        Pagination::standard(self.page, self.limit)
    }

    /// Convert to Pagination with small defaults (10, max 50)
    pub fn small(&self) -> Pagination {
        Pagination::small(self.page, self.limit)
    }

    /// Convert to Pagination with large defaults (50, max 1000)
    pub fn large(&self) -> Pagination {
        Pagination::large(self.page, self.limit)
    }

    /// Convert to Pagination with custom defaults
    pub fn with_defaults(&self, default: u32, max: u32) -> Pagination {
        Pagination::new(self.page, self.limit, default, max)
    }
}
