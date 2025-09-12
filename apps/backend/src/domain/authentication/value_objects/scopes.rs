// OAuth2/OIDC Scopes Value Objects
// Represents access permissions and capabilities

use serde::{Serialize, Deserialize};
use std::collections::HashSet;
use std::str::FromStr;

/// OAuth2/OIDC scope value object
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Scope {
    // OIDC Standard Scopes
    OpenId,
    Profile,
    Email,
    Address,
    Phone,
    OfflineAccess,
    
    // EPSX Platform Scopes
    EpsxAnalytics,
    EpsxTrading,
    EpsxNotifications,
    EpsxAdmin,
    
    // Custom scope for extensibility
    Custom(String),
}

impl Scope {
    /// Get string representation of scope
    pub fn as_str(&self) -> &str {
        match self {
            Scope::OpenId => "openid",
            Scope::Profile => "profile", 
            Scope::Email => "email",
            Scope::Address => "address",
            Scope::Phone => "phone",
            Scope::OfflineAccess => "offline_access",
            Scope::EpsxAnalytics => "epsx:analytics",
            Scope::EpsxTrading => "epsx:trading",
            Scope::EpsxNotifications => "epsx:notifications",
            Scope::EpsxAdmin => "epsx:admin",
            Scope::Custom(s) => s,
        }
    }
    
    /// Check if this is an OIDC standard scope
    pub fn is_oidc_standard(&self) -> bool {
        matches!(self, 
            Scope::OpenId | 
            Scope::Profile | 
            Scope::Email | 
            Scope::Address | 
            Scope::Phone | 
            Scope::OfflineAccess
        )
    }
    
    /// Check if this scope requires user consent
    pub fn requires_consent(&self) -> bool {
        match self {
            Scope::OpenId | Scope::Profile => false, // Basic scopes don't need consent
            _ => true,
        }
    }
    
    /// Get human-readable description of scope
    pub fn description(&self) -> &str {
        match self {
            Scope::OpenId => "Authenticate your identity",
            Scope::Profile => "Access your basic profile information",
            Scope::Email => "Access your email address",
            Scope::Address => "Access your address information",
            Scope::Phone => "Access your phone number",
            Scope::OfflineAccess => "Access your account when you're not present",
            Scope::EpsxAnalytics => "Access EPSX analytics data",
            Scope::EpsxTrading => "Access EPSX trading functionality",
            Scope::EpsxNotifications => "Send you notifications",
            Scope::EpsxAdmin => "Administrative access to EPSX",
            Scope::Custom(s) => s,
        }
    }
}

impl FromStr for Scope {
    type Err = ScopeError;
    
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "openid" => Ok(Scope::OpenId),
            "profile" => Ok(Scope::Profile),
            "email" => Ok(Scope::Email),
            "address" => Ok(Scope::Address),
            "phone" => Ok(Scope::Phone),
            "offline_access" => Ok(Scope::OfflineAccess),
            "epsx:analytics" => Ok(Scope::EpsxAnalytics),
            "epsx:trading" => Ok(Scope::EpsxTrading),
            "epsx:notifications" => Ok(Scope::EpsxNotifications),
            "epsx:admin" => Ok(Scope::EpsxAdmin),
            custom if custom.len() <= 100 => Ok(Scope::Custom(custom.to_string())),
            _ => Err(ScopeError::InvalidScope(s.to_string())),
        }
    }
}

impl std::fmt::Display for Scope {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

/// Set of scopes with validation and operations
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ScopeSet {
    scopes: HashSet<Scope>,
}

impl ScopeSet {
    /// Create empty scope set
    pub fn new() -> Self {
        Self {
            scopes: HashSet::new(),
        }
    }
    
    /// Create scope set from vector
    pub fn from_scopes(scopes: Vec<Scope>) -> Self {
        Self {
            scopes: scopes.into_iter().collect(),
        }
    }
    
    /// Create from space-separated scope string (OAuth2 standard)
    pub fn from_scope_string(scope_str: &str) -> Result<Self, ScopeError> {
        let scopes = scope_str
            .split_whitespace()
            .map(|s| s.parse::<Scope>())
            .collect::<Result<Vec<_>, _>>()?;
        
        Ok(Self::from_scopes(scopes))
    }
    
    /// Add a scope
    pub fn add(&mut self, scope: Scope) {
        self.scopes.insert(scope);
    }
    
    /// Remove a scope
    pub fn remove(&mut self, scope: &Scope) -> bool {
        self.scopes.remove(scope)
    }
    
    /// Check if scope is contained
    pub fn contains(&self, scope: &Scope) -> bool {
        self.scopes.contains(scope)
    }
    
    /// Check if this set contains all scopes from another set
    pub fn contains_all(&self, other: &ScopeSet) -> bool {
        other.scopes.is_subset(&self.scopes)
    }
    
    /// Get intersection with another scope set
    pub fn intersection(&self, other: &ScopeSet) -> ScopeSet {
        ScopeSet {
            scopes: self.scopes.intersection(&other.scopes).cloned().collect(),
        }
    }
    
    /// Convert to space-separated string (OAuth2 standard)
    pub fn to_scope_string(&self) -> String {
        let mut scope_strs: Vec<_> = self.scopes.iter()
            .map(|s| s.as_str())
            .collect();
        scope_strs.sort(); // Ensure deterministic output
        scope_strs.join(" ")
    }
    
    /// Get all scopes that require consent
    pub fn consent_required_scopes(&self) -> Vec<&Scope> {
        self.scopes.iter()
            .filter(|s| s.requires_consent())
            .collect()
    }
    
    /// Check if OpenID Connect is requested (contains openid scope)
    pub fn is_oidc(&self) -> bool {
        self.contains(&Scope::OpenId)
    }
    
    /// Validate scope set against business rules
    pub fn validate(&self) -> Result<(), ScopeError> {
        // Must not be empty
        if self.scopes.is_empty() {
            return Err(ScopeError::EmptyScopes);
        }
        
        // If requesting OIDC scopes, must include openid
        let has_oidc_scopes = self.scopes.iter().any(|s| match s {
            Scope::Profile | Scope::Email | Scope::Address | Scope::Phone => true,
            _ => false,
        });
        
        if has_oidc_scopes && !self.contains(&Scope::OpenId) {
            return Err(ScopeError::MissingOpenIdScope);
        }
        
        // Admin scope requires email verification
        if self.contains(&Scope::EpsxAdmin) && !self.contains(&Scope::Email) {
            return Err(ScopeError::AdminRequiresEmail);
        }
        
        Ok(())
    }
    
    /// Get iterator over scopes
    pub fn iter(&self) -> impl Iterator<Item = &Scope> {
        self.scopes.iter()
    }
    
    /// Get number of scopes
    pub fn len(&self) -> usize {
        self.scopes.len()
    }
    
    /// Check if empty
    pub fn is_empty(&self) -> bool {
        self.scopes.is_empty()
    }
}

impl Default for ScopeSet {
    fn default() -> Self {
        Self::new()
    }
}

impl FromIterator<Scope> for ScopeSet {
    fn from_iter<T: IntoIterator<Item = Scope>>(iter: T) -> Self {
        Self {
            scopes: iter.into_iter().collect(),
        }
    }
}

impl IntoIterator for ScopeSet {
    type Item = Scope;
    type IntoIter = std::collections::hash_set::IntoIter<Scope>;
    
    fn into_iter(self) -> Self::IntoIter {
        self.scopes.into_iter()
    }
}

/// Scope-related errors
#[derive(Debug, thiserror::Error)]
pub enum ScopeError {
    #[error("Invalid scope: {0}")]
    InvalidScope(String),
    
    #[error("Scope set cannot be empty")]
    EmptyScopes,
    
    #[error("OIDC scopes require 'openid' scope")]
    MissingOpenIdScope,
    
    #[error("Admin scope requires email access")]
    AdminRequiresEmail,
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn scope_parsing() {
        assert_eq!("openid".parse::<Scope>().unwrap(), Scope::OpenId);
        assert_eq!("epsx:analytics".parse::<Scope>().unwrap(), Scope::EpsxAnalytics);
        assert!("invalid_scope_that_is_way_too_long_and_exceeds_the_maximum_allowed_length".parse::<Scope>().is_err());
    }
    
    #[test]
    fn scope_set_operations() {
        let scope_set = ScopeSet::new();
        scope_set.add(Scope::OpenId);
        scope_set.add(Scope::Profile);
        
        assert!(scope_set.contains(&Scope::OpenId));
        assert!(scope_set.is_oidc());
        assert_eq!(scope_set.len(), 2);
    }
    
    #[test]
    fn scope_string_conversion() {
        let scope_set = ScopeSet::from_scope_string("openid profile email").unwrap();
        assert_eq!(scope_set.len(), 3);
        assert!(scope_set.contains(&Scope::OpenId));
        
        let scope_string = scope_set.to_scope_string();
        assert!(scope_string.contains("openid"));
        assert!(scope_string.contains("profile"));
        assert!(scope_string.contains("email"));
    }
    
    #[test]
    fn scope_validation() {
        // Valid OIDC scope set
        let valid_set = ScopeSet::from_scope_string("openid profile").unwrap();
        assert!(valid_set.validate().is_ok());
        
        // Invalid: OIDC scope without openid
        let invalid_set = ScopeSet::from_scope_string("profile email").unwrap();
        assert!(invalid_set.validate().is_err());
        
        // Empty scope set
        let empty_set = ScopeSet::new();
        assert!(empty_set.validate().is_err());
    }
}