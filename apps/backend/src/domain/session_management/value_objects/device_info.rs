// Device Info Value Object
// Tracks device information for session security and analytics

use chrono::{DateTime, Utc};
use serde::{Serialize, Deserialize};
use std::collections::HashMap;

/// Device information associated with a session
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeviceInfo {
    /// Device identification
    pub device_id: Option<String>,
    pub device_type: DeviceType,
    pub device_fingerprint: DeviceFingerprint,
    
    /// Browser/client information
    pub user_agent: String,
    pub browser_info: BrowserInfo,
    
    /// Operating system
    pub os_info: OperatingSystemInfo,
    
    /// Network information
    pub ip_address: String,
    pub location: Option<LocationInfo>,
    
    /// Device capabilities
    pub screen_resolution: Option<String>,
    pub timezone: Option<String>,
    pub language: Option<String>,
    
    /// Trust and security
    pub is_trusted: bool,
    pub trust_score: f64, // 0-100
    pub first_seen: DateTime<Utc>,
    pub last_seen: DateTime<Utc>,
}

impl DeviceInfo {
    /// Create new device info from user agent and IP
    pub fn new(user_agent: String, ip_address: String) -> Self {
        let now = Utc::now();
        let browser_info = BrowserInfo::parse_from_user_agent(&user_agent);
        let os_info = OperatingSystemInfo::parse_from_user_agent(&user_agent);
        let device_type = DeviceType::infer_from_user_agent(&user_agent);
        
        Self {
            device_id: None,
            device_type,
            device_fingerprint: DeviceFingerprint::generate_from_user_agent(&user_agent),
            user_agent,
            browser_info,
            os_info,
            ip_address,
            location: None,
            screen_resolution: None,
            timezone: None,
            language: None,
            is_trusted: false,
            trust_score: 50.0, // Neutral trust initially
            first_seen: now,
            last_seen: now,
        }
    }
    
    /// Update device info with additional data
    pub fn update(&mut self, 
        screen_resolution: Option<String>, 
        timezone: Option<String>, 
        language: Option<String>
    ) {
        self.screen_resolution = screen_resolution;
        self.timezone = timezone;
        self.language = language;
        self.last_seen = Utc::now();
        
        // Recalculate trust score with more data
        self.recalculate_trust_score();
    }
    
    /// Add location information
    pub fn add_location(&mut self, location: LocationInfo) {
        self.location = Some(location);
        self.recalculate_trust_score();
    }
    
    /// Mark device as trusted
    pub fn mark_trusted(&mut self) {
        self.is_trusted = true;
        self.trust_score = 95.0;
    }
    
    /// Mark device as suspicious
    pub fn mark_suspicious(&mut self, reason: String) {
        self.trust_score = (self.trust_score - 20.0).max(0.0);
        
        // Add to device fingerprint flags
        self.device_fingerprint.add_flag(format!("suspicious:{}", reason));
    }
    
    /// Check if device is mobile
    pub fn is_mobile(&self) -> bool {
        matches!(self.device_type, DeviceType::Mobile | DeviceType::Tablet)
    }
    
    /// Check if device is desktop
    pub fn is_desktop(&self) -> bool {
        matches!(self.device_type, DeviceType::Desktop | DeviceType::Laptop)
    }
    
    /// Get device summary for logging
    pub fn summary(&self) -> DeviceSummary {
        DeviceSummary {
            device_type: self.device_type.clone(),
            browser: self.browser_info.name.clone(),
            os: self.os_info.name.clone(),
            location: self.location.as_ref().map(|l| l.country.clone()),
            is_trusted: self.is_trusted,
            trust_score: self.trust_score,
            fingerprint_hash: self.device_fingerprint.hash.clone(),
        }
    }
    
    /// Recalculate trust score based on available data
    fn recalculate_trust_score(&mut self) {
        let mut score = 50.0; // Base score
        
        // Browser trust factors
        match self.browser_info.name.as_str() {
            "Chrome" | "Firefox" | "Safari" | "Edge" => score += 10.0,
            "Unknown" => score -= 20.0,
            _ => {} // No change for other browsers
        }
        
        // OS trust factors
        if !self.os_info.name.contains("Unknown") {
            score += 5.0;
        }
        
        // Location trust (if available)
        if self.location.is_some() {
            score += 5.0;
        }
        
        // Complete device info bonus
        if self.screen_resolution.is_some() && 
           self.timezone.is_some() && 
           self.language.is_some() {
            score += 10.0;
        }
        
        // Device age factor (older devices slightly more trusted)
        let device_age_days = (Utc::now() - self.first_seen).num_days();
        if device_age_days > 30 {
            score += 5.0;
        }
        
        self.trust_score = score.clamp(0.0, 100.0);
    }
}

/// Type of device
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum DeviceType {
    Mobile,
    Tablet,
    Desktop,
    Laptop,
    Bot,
    Unknown,
}

impl DeviceType {
    /// Infer device type from user agent
    pub fn infer_from_user_agent(user_agent: &str) -> Self {
        let ua_lower = user_agent.to_lowercase();
        
        if ua_lower.contains("mobile") || ua_lower.contains("iphone") || ua_lower.contains("android") {
            DeviceType::Mobile
        } else if ua_lower.contains("tablet") || ua_lower.contains("ipad") {
            DeviceType::Tablet
        } else if ua_lower.contains("bot") || ua_lower.contains("crawler") || ua_lower.contains("spider") {
            DeviceType::Bot
        } else if ua_lower.contains("windows") || ua_lower.contains("macintosh") || ua_lower.contains("linux") {
            DeviceType::Desktop
        } else {
            DeviceType::Unknown
        }
    }
}

/// Device fingerprint for identification
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeviceFingerprint {
    /// Unique hash for this device combination
    pub hash: String,
    
    /// Components used in fingerprinting
    pub components: HashMap<String, String>,
    
    /// Security flags
    pub flags: Vec<String>,
    
    /// Confidence in fingerprint accuracy
    pub confidence: f64, // 0-100
}

impl DeviceFingerprint {
    /// Generate fingerprint from user agent
    pub fn generate_from_user_agent(user_agent: &str) -> Self {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};
        
        let mut hasher = DefaultHasher::new();
        user_agent.hash(&mut hasher);
        let hash = format!("fp_{:x}", hasher.finish());
        
        let mut components = HashMap::new();
        components.insert("user_agent".to_string(), user_agent.to_string());
        
        Self {
            hash,
            components,
            flags: vec![],
            confidence: 60.0, // Moderate confidence with just user agent
        }
    }
    
    /// Add additional fingerprint component
    pub fn add_component(&mut self, key: String, value: String) {
        self.components.insert(key, value);
        self.recalculate_hash();
        
        // Increase confidence with more data points
        self.confidence = (self.confidence + 5.0).min(100.0);
    }
    
    /// Add security flag
    pub fn add_flag(&mut self, flag: String) {
        if !self.flags.contains(&flag) {
            self.flags.push(flag);
        }
    }
    
    /// Recalculate hash from all components
    fn recalculate_hash(&mut self) {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};
        
        let mut hasher = DefaultHasher::new();
        
        // Sort components for consistent hashing
        let mut sorted_components: Vec<_> = self.components.iter().collect();
        sorted_components.sort_by_key(|(k, _)| *k);
        
        for (key, value) in sorted_components {
            key.hash(&mut hasher);
            value.hash(&mut hasher);
        }
        
        self.hash = format!("fp_{:x}", hasher.finish());
    }
}

/// Browser information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BrowserInfo {
    pub name: String,
    pub version: Option<String>,
    pub engine: Option<String>,
}

impl BrowserInfo {
    /// Parse browser info from user agent
    pub fn parse_from_user_agent(user_agent: &str) -> Self {
        // Simple browser detection - in production, use a proper user agent parser
        let ua_lower = user_agent.to_lowercase();
        
        let (name, engine) = if ua_lower.contains("chrome") && !ua_lower.contains("edg") {
            ("Chrome".to_string(), Some("Blink".to_string()))
        } else if ua_lower.contains("firefox") {
            ("Firefox".to_string(), Some("Gecko".to_string()))
        } else if ua_lower.contains("safari") && !ua_lower.contains("chrome") {
            ("Safari".to_string(), Some("WebKit".to_string()))
        } else if ua_lower.contains("edg") {
            ("Edge".to_string(), Some("Blink".to_string()))
        } else {
            ("Unknown".to_string(), None)
        };
        
        Self {
            name,
            version: None, // Would need regex parsing for version
            engine,
        }
    }
}

/// Operating system information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OperatingSystemInfo {
    pub name: String,
    pub version: Option<String>,
    pub architecture: Option<String>,
}

impl OperatingSystemInfo {
    /// Parse OS info from user agent
    pub fn parse_from_user_agent(user_agent: &str) -> Self {
        let ua_lower = user_agent.to_lowercase();
        
        let name = if ua_lower.contains("windows") {
            "Windows".to_string()
        } else if ua_lower.contains("macintosh") || ua_lower.contains("mac os") {
            "macOS".to_string()
        } else if ua_lower.contains("linux") {
            "Linux".to_string()
        } else if ua_lower.contains("android") {
            "Android".to_string()
        } else if ua_lower.contains("iphone") || ua_lower.contains("ipad") {
            "iOS".to_string()
        } else {
            "Unknown".to_string()
        };
        
        Self {
            name,
            version: None, // Would need regex parsing for version
            architecture: None,
        }
    }
}

/// Geographic location information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LocationInfo {
    pub country: String,
    pub region: Option<String>,
    pub city: Option<String>,
    pub latitude: Option<f64>,
    pub longitude: Option<f64>,
    pub timezone: Option<String>,
}

/// Device summary for monitoring
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeviceSummary {
    pub device_type: DeviceType,
    pub browser: String,
    pub os: String,
    pub location: Option<String>,
    pub is_trusted: bool,
    pub trust_score: f64,
    pub fingerprint_hash: String,
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn create_device_info() {
        let user_agent = "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/91.0.4472.124 Safari/537.36".to_string();
        let ip = "192.168.1.1".to_string();
        
        let device_info = DeviceInfo::new(user_agent.clone(), ip.clone());
        
        assert_eq!(device_info.user_agent, user_agent);
        assert_eq!(device_info.ip_address, ip);
        assert_eq!(device_info.device_type, DeviceType::Desktop);
        assert_eq!(device_info.browser_info.name, "Chrome");
        assert_eq!(device_info.os_info.name, "Windows");
        assert!(!device_info.is_trusted);
    }
    
    #[test]
    fn device_type_inference() {
        assert_eq!(
            DeviceType::infer_from_user_agent("Mozilla/5.0 (iPhone; CPU iPhone OS 14_0 like Mac OS X)"),
            DeviceType::Mobile
        );
        
        assert_eq!(
            DeviceType::infer_from_user_agent("Mozilla/5.0 (iPad; CPU OS 14_0 like Mac OS X)"),
            DeviceType::Tablet
        );
        
        assert_eq!(
            DeviceType::infer_from_user_agent("Googlebot/2.1"),
            DeviceType::Bot
        );
        
        assert_eq!(
            DeviceType::infer_from_user_agent("Mozilla/5.0 (Windows NT 10.0; Win64; x64)"),
            DeviceType::Desktop
        );
    }
    
    #[test]
    fn device_fingerprint_generation() {
        let user_agent = "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36";
        let fingerprint = DeviceFingerprint::generate_from_user_agent(user_agent);
        
        assert!(fingerprint.hash.starts_with("fp_"));
        assert_eq!(fingerprint.components.get("user_agent"), Some(&user_agent.to_string()));
        assert_eq!(fingerprint.confidence, 60.0);
    }
    
    #[test]
    fn device_trust_scoring() {
        let user_agent = "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/91.0.4472.124 Safari/537.36";
        let mut device_info = DeviceInfo::new(user_agent.to_string(), "192.168.1.1".to_string());
        
        let initial_score = device_info.trust_score;
        
        // Add more device info
        device_info.update(
            Some("1920x1080".to_string()),
            Some("America/New_York".to_string()),
            Some("en-US".to_string())
        );
        
        // Trust score should increase with more complete info
        assert!(device_info.trust_score > initial_score);
        
        // Mark as suspicious
        device_info.mark_suspicious("multiple_failed_logins".to_string());
        
        // Trust score should decrease
        assert!(device_info.trust_score < 50.0);
        
        // Mark as trusted
        device_info.mark_trusted();
        assert!(device_info.is_trusted);
        assert_eq!(device_info.trust_score, 95.0);
    }
}