// Brute Force Detection Engine - Complete Diesel Implementation
use diesel::prelude::*;
use chrono::{DateTime, Utc, Duration};
use uuid::Uuid;
use tracing::{info, error, warn, debug};
use std::collections::HashMap;
use ipnetwork::IpNetwork;

use crate::{
    core::errors::{AppError, AppResult},
    infra::{
        cache::Cache,
        db::diesel::{
            models::{
                DieselAttackAttempt, 
                NewDieselAttackAttempt,
                DieselIpBlacklist,
                NewDieselIpBlacklist,
            },
            pool::DbPool,
            schema::{attack_attempts, ip_blacklist},
        },
    },
};

pub struct BruteForceDetector {
    pool: DbPool,
    cache: Option<Box<dyn Cache>>,
    config: BruteForceConfig,
}

#[derive(Debug, Clone)]
pub struct BruteForceConfig {
    pub max_attempts: i32,
    pub window_minutes: i32,
    pub block_duration_minutes: i32,
    pub escalation_factor: f32,
    pub whitelist_ips: Vec<IpNetwork>,
}

#[derive(Debug, Clone)]
pub struct AttackPattern {
    pub ip_address: IpNetwork,
    pub attack_type: String,
    pub attempt_count: i32,
    pub first_attempt: DateTime<Utc>,
    pub last_attempt: DateTime<Utc>,
    pub blocked: bool,
}

#[derive(Debug, Clone)]
pub struct BruteForceAnalysis {
    pub is_attack: bool,
    pub risk_score: i32,
    pub should_block: bool,
    pub pattern: Option<AttackPattern>,
}

impl Default for BruteForceConfig {
    fn default() -> Self {
        Self {
            max_attempts: 5,
            window_minutes: 15,
            block_duration_minutes: 60,
            escalation_factor: 2.0,
            whitelist_ips: vec![],
        }
    }
}

impl BruteForceDetector {
    pub fn new(pool: DbPool) -> Self {
        Self {
            pool,
            cache: None,
            config: BruteForceConfig::default(),
        }
    }

    pub fn with_config(pool: DbPool, config: BruteForceConfig) -> Self {
        Self {
            pool,
            cache: None,
            config,
        }
    }

    pub fn with_cache(pool: DbPool, cache_instance: Box<dyn Cache>) -> Self {
        Self {
            pool,
            cache: Some(cache_instance),
            config: BruteForceConfig::default(),
        }
    }

    // Record an attack attempt
    pub async fn record_attempt(&self, ip_addr: IpNetwork, attack_type: &str, success: bool) -> AppResult<Uuid> {
        let mut conn = self.pool.get().await
            .map_err(|e| AppError::Database {
                message: "Failed to get database connection".to_string(),
                source: Some(Box::new(e)),
                correlation_id: None,
            })?;

        let attempt_id = Uuid::new_v4();
        let new_attempt = NewDieselAttackAttempt {
            id: attempt_id,
            ip_address: ip_addr,
            attack_type: attack_type.to_string(),
            success,
            user_agent: None,
            request_path: None,
            created_at: Utc::now(),
        };

        diesel::insert_into(attack_attempts::table)
            .values(&new_attempt)
            .execute(&mut conn)
            .await
            .map_err(|e| {
                error!("Failed to record attack attempt: {}", e);
                AppError::Database {
                    message: "Failed to record attack attempt".to_string(),
                    source: Some(Box::new(e)),
                    correlation_id: None,
                }
            })?;

        info!("Recorded attack attempt {} from {} (type: {}, success: {})", 
              attempt_id, ip_addr, attack_type, success);

        Ok(attempt_id)
    }

    // Analyze IP for brute force patterns
    pub async fn analyze_ip(&self, ip_addr: IpNetwork) -> AppResult<BruteForceAnalysis> {
        // Check if IP is whitelisted
        if self.is_whitelisted(&ip_addr) {
            debug!("IP {} is whitelisted, skipping brute force analysis", ip_addr);
            return Ok(BruteForceAnalysis {
                is_attack: false,
                risk_score: 0,
                should_block: false,
                pattern: None,
            });
        }

        let mut conn = self.pool.get().await
            .map_err(|e| AppError::Database {
                message: "Failed to get database connection".to_string(),
                source: Some(Box::new(e)),
                correlation_id: None,
            })?;

        let window_start = Utc::now() - Duration::minutes(self.config.window_minutes as i64);

        // Get recent attempts from this IP
        let attempts = attack_attempts::table
            .filter(attack_attempts::ip_address.eq(ip_addr))
            .filter(attack_attempts::created_at.gt(window_start))
            .order(attack_attempts::created_at.desc())
            .load::<DieselAttackAttempt>(&mut conn)
            .await
            .map_err(|e| AppError::Database {
                message: "Failed to load attack attempts".to_string(),
                source: Some(Box::new(e)),
                correlation_id: None,
            })?;

        if attempts.is_empty() {
            return Ok(BruteForceAnalysis {
                is_attack: false,
                risk_score: 0,
                should_block: false,
                pattern: None,
            });
        }

        // Group attempts by attack type
        let mut attack_patterns: HashMap<String, Vec<&DieselAttackAttempt>> = HashMap::new();
        for attempt in &attempts {
            attack_patterns.entry(attempt.attack_type.clone())
                .or_insert_with(Vec::new)
                .push(attempt);
        }

        let mut max_risk_score = 0;
        let mut should_block = false;
        let mut detected_pattern: Option<AttackPattern> = None;

        // Analyze each attack pattern
        for (attack_type, type_attempts) in attack_patterns {
            let attempt_count = type_attempts.len() as i32;
            let failed_attempts = type_attempts.iter().filter(|a| !a.success).count() as i32;
            
            // Calculate risk score
            let base_risk = (failed_attempts as f32 / self.config.max_attempts as f32 * 100.0) as i32;
            let frequency_multiplier = if attempt_count > self.config.max_attempts {
                self.config.escalation_factor
            } else {
                1.0
            };
            let risk_score = (base_risk as f32 * frequency_multiplier) as i32;

            if risk_score > max_risk_score {
                max_risk_score = risk_score;
            }

            // Determine if this pattern constitutes an attack
            let is_pattern_attack = failed_attempts >= self.config.max_attempts;
            if is_pattern_attack {
                should_block = true;
                detected_pattern = Some(AttackPattern {
                    ip_address: ip_addr,
                    attack_type: attack_type.clone(),
                    attempt_count: failed_attempts,
                    first_attempt: type_attempts.last().unwrap().created_at,
                    last_attempt: type_attempts.first().unwrap().created_at,
                    blocked: false,
                });
            }
        }

        let analysis = BruteForceAnalysis {
            is_attack: max_risk_score >= 75, // High risk threshold
            risk_score: max_risk_score,
            should_block,
            pattern: detected_pattern,
        };

        if analysis.should_block {
            warn!("Brute force attack detected from {} - risk score: {}", ip_addr, max_risk_score);
        } else if analysis.is_attack {
            warn!("Suspicious activity from {} - risk score: {}", ip_addr, max_risk_score);
        }

        Ok(analysis)
    }

    // Block an IP address
    pub async fn block_ip(&self, ip_addr: IpNetwork, reason: &str, duration_minutes: Option<i32>) -> AppResult<Uuid> {
        let mut conn = self.pool.get().await
            .map_err(|e| AppError::Database {
                message: "Failed to get database connection".to_string(),
                source: Some(Box::new(e)),
                correlation_id: None,
            })?;

        let block_id = Uuid::new_v4();
        let block_duration = duration_minutes.unwrap_or(self.config.block_duration_minutes);
        let expires_at = Utc::now() + Duration::minutes(block_duration as i64);

        let new_block = NewDieselIpBlacklist {
            id: block_id,
            ip_address: ip_addr,
            reason: reason.to_string(),
            blocked_until: Some(expires_at),
            created_at: Utc::now(),
            is_permanent: false,
        };

        diesel::insert_into(ip_blacklist::table)
            .values(&new_block)
            .execute(&mut conn)
            .await
            .map_err(|e| {
                error!("Failed to block IP {}: {}", ip_addr, e);
                AppError::Database {
                    message: "Failed to block IP address".to_string(),
                    source: Some(Box::new(e)),
                    correlation_id: None,
                }
            })?;

        info!("Blocked IP {} for {} minutes (reason: {})", ip_addr, block_duration, reason);
        Ok(block_id)
    }

    // Check if an IP is currently blocked
    pub async fn is_blocked(&self, ip_addr: IpNetwork) -> AppResult<bool> {
        let mut conn = self.pool.get().await
            .map_err(|e| AppError::Database {
                message: "Failed to get database connection".to_string(),
                source: Some(Box::new(e)),
                correlation_id: None,
            })?;

        let active_blocks = ip_blacklist::table
            .filter(ip_blacklist::ip_address.eq(ip_addr))
            .filter(
                ip_blacklist::is_permanent.eq(true)
                    .or(ip_blacklist::blocked_until.gt(Utc::now()))
            )
            .count()
            .get_result::<i64>(&mut conn)
            .await
            .map_err(|e| AppError::Database {
                message: "Failed to check IP block status".to_string(),
                source: Some(Box::new(e)),
                correlation_id: None,
            })?;

        Ok(active_blocks > 0)
    }

    // Get attack statistics for the last N hours
    pub async fn get_attack_stats(&self, hours_back: i32) -> AppResult<AttackStats> {
        let mut conn = self.pool.get().await
            .map_err(|e| AppError::Database {
                message: "Failed to get database connection".to_string(),
                source: Some(Box::new(e)),
                correlation_id: None,
            })?;

        let cutoff_time = Utc::now() - Duration::hours(hours_back as i64);

        let total_attempts = attack_attempts::table
            .filter(attack_attempts::created_at.gt(cutoff_time))
            .count()
            .get_result::<i64>(&mut conn)
            .await
            .map_err(|e| AppError::Database {
                message: "Failed to count total attempts".to_string(),
                source: Some(Box::new(e)),
                correlation_id: None,
            })?;

        let successful_attacks = attack_attempts::table
            .filter(attack_attempts::created_at.gt(cutoff_time))
            .filter(attack_attempts::success.eq(true))
            .count()
            .get_result::<i64>(&mut conn)
            .await
            .map_err(|e| AppError::Database {
                message: "Failed to count successful attacks".to_string(),
                source: Some(Box::new(e)),
                correlation_id: None,
            })?;

        let unique_ips = attack_attempts::table
            .filter(attack_attempts::created_at.gt(cutoff_time))
            .select(attack_attempts::ip_address)
            .distinct()
            .count()
            .get_result::<i64>(&mut conn)
            .await
            .map_err(|e| AppError::Database {
                message: "Failed to count unique IPs".to_string(),
                source: Some(Box::new(e)),
                correlation_id: None,
            })?;

        let blocked_ips = ip_blacklist::table
            .filter(ip_blacklist::created_at.gt(cutoff_time))
            .count()
            .get_result::<i64>(&mut conn)
            .await
            .map_err(|e| AppError::Database {
                message: "Failed to count blocked IPs".to_string(),
                source: Some(Box::new(e)),
                correlation_id: None,
            })?;

        Ok(AttackStats {
            total_attempts,
            successful_attacks,
            unique_attacking_ips: unique_ips,
            blocked_ips,
            period_hours: hours_back,
        })
    }

    // Clean up expired blocks
    pub async fn cleanup_expired_blocks(&self) -> AppResult<usize> {
        let mut conn = self.pool.get().await
            .map_err(|e| AppError::Database {
                message: "Failed to get database connection".to_string(),
                source: Some(Box::new(e)),
                correlation_id: None,
            })?;

        let deleted_count = diesel::delete(
            ip_blacklist::table
                .filter(ip_blacklist::is_permanent.eq(false))
                .filter(ip_blacklist::blocked_until.lt(Utc::now()))
        )
        .execute(&mut conn)
        .await
        .map_err(|e| AppError::Database {
            message: "Failed to cleanup expired blocks".to_string(),
            source: Some(Box::new(e)),
            correlation_id: None,
        })?;

        if deleted_count > 0 {
            info!("Cleaned up {} expired IP blocks", deleted_count);
        }

        Ok(deleted_count)
    }

    // Check if IP is in whitelist
    fn is_whitelisted(&self, ip_addr: &IpNetwork) -> bool {
        self.config.whitelist_ips.iter().any(|whitelist_ip| {
            whitelist_ip.contains(*ip_addr)
        })
    }

    // Get recent blocked IPs
    pub async fn get_recent_blocks(&self, limit: i32) -> AppResult<Vec<DieselIpBlacklist>> {
        let mut conn = self.pool.get().await
            .map_err(|e| AppError::Database {
                message: "Failed to get database connection".to_string(),
                source: Some(Box::new(e)),
                correlation_id: None,
            })?;

        let blocks = ip_blacklist::table
            .order(ip_blacklist::created_at.desc())
            .limit(limit as i64)
            .load::<DieselIpBlacklist>(&mut conn)
            .await
            .map_err(|e| AppError::Database {
                message: "Failed to load recent blocks".to_string(),
                source: Some(Box::new(e)),
                correlation_id: None,
            })?;

        Ok(blocks)
    }
}

#[derive(Debug, Clone)]
pub struct AttackStats {
    pub total_attempts: i64,
    pub successful_attacks: i64,
    pub unique_attacking_ips: i64,
    pub blocked_ips: i64,
    pub period_hours: i32,
}
