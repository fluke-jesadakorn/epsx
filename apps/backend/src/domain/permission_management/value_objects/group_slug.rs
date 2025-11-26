use crate::prelude::*;

/// Group slug value object (URL-safe identifier)
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct GroupSlug(String);

impl GroupSlug {
    pub fn new(slug: impl Into<String>) -> AppResult<Self> {
        let slug = slug.into();

        // Validate slug format: lowercase, alphanumeric, hyphens only
        if !slug.chars().all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '-') {
            return Err(AppError::validation_error(
                "Slug must be lowercase alphanumeric with hyphens only"
            ));
        }

        if slug.is_empty() || slug.len() > 100 {
            return Err(AppError::validation_error(
                "Slug must be between 1 and 100 characters"
            ));
        }

        Ok(Self(slug))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }

    pub fn value(&self) -> String {
        self.0.clone()
    }
}

impl std::fmt::Display for GroupSlug {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}
