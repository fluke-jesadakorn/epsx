// Response management system for brute force attacks

use super::models::*;
use super::BruteForceConfig;
use crate::infra::cache::{Cache, CacheExt};
use chrono::{Utc, Duration};
use std::sync::Arc;
use std::collections::HashMap;
use tracing::{info, warn};
// 

/// Manages automated responses to brute force attacks
pub struct ResponseManager {
    config: BruteForceConfig,
    db_pool: Arc<crate::infra::db::diesel::DbPool>,
    cache: Arc<dyn Cache>,
}

impl ResponseManager {
    pub fn new(
        config: BruteForceConfig,
        db_pool: Arc<crate::infra::db::diesel::DbPool>,
        cache: Arc<dyn Cache>,
    ) -> Self {
        Self {
            config,
            db_pool,
            cache,
        }
    }

    /// Execute appropriate response based on analysis
    pub async fn execute_response(
        &self,
        analysis: &AttackAnalysis,
        pattern_analysis: &PatternAnalysisResult,
    ) -> Result<ResponseExecutionResult, BruteForceError> {
        if !analysis.attack_detected {
            return Ok(ResponseExecutionResult {
                actions: vec![],
                blocked_until: None,
                recommendations: vec!["Continue monitoring".to_string()],
                strategy: ResponseStrategy::Progressive,
                escalation_level: 0,
            });
        }

        let strategy = self.determine_response_strategy(analysis, pattern_analysis);
        let escalation_level = self.calculate_escalation_level(analysis);
        
        info!(
            "Executing {} response strategy (level {}) for threat: {:?}",
            format!("{:?}", strategy), escalation_level, analysis.threat_level
        );

        let mut actions = Vec::new();
        let mut blocked_until = None;
        let mut recommendations = Vec::new();

        // Execute responses based on strategy
        match strategy {
            ResponseStrategy::Immediate => {
                actions.extend(self.execute_immediate_response(analysis).await?);
                blocked_until = Some(Utc::now() + Duration::minutes(
                    self.config.block_duration_minutes as i64
                ));
            },
            ResponseStrategy::Progressive => {
                actions.extend(self.execute_progressive_response(analysis, escalation_level).await?);
                if escalation_level >= 2 {
                    blocked_until = Some(Utc::now() + Duration::minutes(
                        (self.config.block_duration_minutes * escalation_level) as i64
                    ));
                }
            },
            ResponseStrategy::Adaptive => {
                actions.extend(self.execute_adaptive_response(analysis, pattern_analysis).await?);
                if analysis.threat_level == ThreatLevel::Critical {
                    blocked_until = Some(Utc::now() + Duration::hours(1));
                }
            },
            ResponseStrategy::ManualReview => {
                actions.push(ResponseActionResult {
                    action_type: ResponseActionType::HoneypotRedirect,
                    success: true,
                    message: "Redirected to security review queue".to_string(),
                    duration: None,
                });
                recommendations.push("Manual security review required".to_string());
            },
        }

        // Generate contextual recommendations
        recommendations.extend(self.generate_recommendations(analysis, &strategy));

        // Log response actions
        self.log_response_actions(analysis, &actions).await?;

        Ok(ResponseExecutionResult {
            actions,
            blocked_until,
            recommendations,
            strategy,
            escalation_level,
        })
    }

    /// Determine appropriate response strategy
    fn determine_response_strategy(
        &self,
        analysis: &AttackAnalysis,
        pattern_analysis: &PatternAnalysisResult,
    ) -> ResponseStrategy {
        // Immediate response for critical threats
        if analysis.threat_level == ThreatLevel::Critical {
            return ResponseStrategy::Immediate;
        }

        // Adaptive response for high ML confidence with patterns
        if pattern_analysis.ml_confidence >= 0.8 && !pattern_analysis.pattern_matches.is_empty() {
            return ResponseStrategy::Adaptive;
        }

        // Manual review for complex attack patterns
        if analysis.attack_types.len() > 2 && 
           analysis.attack_types.contains(&AttackType::HybridAttack) {
            return ResponseStrategy::ManualReview;
        }

        // Default to progressive response
        ResponseStrategy::Progressive
    }

    /// Calculate escalation level based on attack severity
    fn calculate_escalation_level(&self, analysis: &AttackAnalysis) -> u32 {
        let mut level = 0u32;

        // Base level from threat level
        level += analysis.threat_level.to_score() as u32;

        // Add for confidence score
        if analysis.confidence_score >= 0.9 {
            level += 2;
        } else if analysis.confidence_score >= 0.7 {
            level += 1;
        }

        // Add for multiple attack types
        if analysis.attack_types.len() > 1 {
            level += 1;
        }

        // Add for high-risk factors
        let high_risk_factors = analysis.risk_factors.iter()
            .filter(|rf| rf.weight >= 0.8)
            .count();
        level += high_risk_factors as u32;

        level.min(5) // Cap at level 5
    }

    /// Execute immediate response (critical threats)
    async fn execute_immediate_response(
        &self,
        analysis: &AttackAnalysis,
    ) -> Result<Vec<ResponseActionResult>, BruteForceError> {
        let mut actions = Vec::new();
        let ip = self.extract_ip_from_analysis(analysis);

        // Immediate IP block
        if let Some(ip_addr) = &ip {
            let block_result = self.block_ip(ip_addr, "Critical threat detected").await;
            actions.push(ResponseActionResult {
                action_type: ResponseActionType::IpBlock,
                success: block_result.is_ok(),
                message: if block_result.is_ok() {
                    format!("IP {} blocked immediately", ip_addr)
                } else {
                    format!("Failed to block IP {}: {}", ip_addr, block_result.unwrap_err())
                },
                duration: Some(Duration::minutes(self.config.block_duration_minutes as i64)),
            });
        }

        // Rate limiting
        if let Some(ip_addr) = &ip {
            let rate_limit_result = self.apply_strict_rate_limit(ip_addr).await;
            actions.push(ResponseActionResult {
                action_type: ResponseActionType::RateLimit,
                success: rate_limit_result.is_ok(),
                message: "Applied strict rate limiting".to_string(),
                duration: Some(Duration::hours(1)),
            });
        }

        Ok(actions)
    }

    /// Execute progressive response (escalating measures)
    async fn execute_progressive_response(
        &self,
        analysis: &AttackAnalysis,
        escalation_level: u32,
    ) -> Result<Vec<ResponseActionResult>, BruteForceError> {
        let mut actions = Vec::new();
        let ip = self.extract_ip_from_analysis(analysis);

        match escalation_level {
            1 => {
                // Level 1: Basic rate limiting
                if let Some(ip_addr) = &ip {
                    let result = self.apply_rate_limit(ip_addr, 5, 60).await; // 5 requests per minute
                    actions.push(ResponseActionResult {
                        action_type: ResponseActionType::RateLimit,
                        success: result.is_ok(),
                        message: "Applied basic rate limiting (5/min)".to_string(),
                        duration: Some(Duration::minutes(30)),
                    });
                }
            },
            2 => {
                // Level 2: CAPTCHA challenge + stricter rate limiting
                if let Some(ip_addr) = &ip {
                    let rate_result = self.apply_rate_limit(ip_addr, 2, 60).await; // 2 requests per minute
                    actions.push(ResponseActionResult {
                        action_type: ResponseActionType::RateLimit,
                        success: rate_result.is_ok(),
                        message: "Applied strict rate limiting (2/min)".to_string(),
                        duration: Some(Duration::minutes(60)),
                    });

                    let captcha_result = self.enable_captcha_challenge(ip_addr).await;
                    actions.push(ResponseActionResult {
                        action_type: ResponseActionType::CaptchaChallenge,
                        success: captcha_result.is_ok(),
                        message: "Enabled CAPTCHA challenge".to_string(),
                        duration: Some(Duration::hours(2)),
                    });
                }
            },
            3 => {
                // Level 3: Temporary IP block
                if let Some(ip_addr) = &ip {
                    let block_result = self.block_ip_temporary(
                        ip_addr, 
                        self.config.block_duration_minutes,
                        "Progressive escalation - temporary block"
                    ).await;
                    actions.push(ResponseActionResult {
                        action_type: ResponseActionType::IpBlock,
                        success: block_result.is_ok(),
                        message: format!("Temporary IP block ({} minutes)", self.config.block_duration_minutes),
                        duration: Some(Duration::minutes(self.config.block_duration_minutes as i64)),
                    });
                }
            },
            4 => {
                // Level 4: Extended IP block + user agent block
                if let Some(ip_addr) = &ip {
                    let extended_block_result = self.block_ip_temporary(
                        ip_addr,
                        self.config.block_duration_minutes * 2,
                        "Progressive escalation - extended block"
                    ).await;
                    actions.push(ResponseActionResult {
                        action_type: ResponseActionType::IpBlock,
                        success: extended_block_result.is_ok(),
                        message: format!("Extended IP block ({} minutes)", self.config.block_duration_minutes * 2),
                        duration: Some(Duration::minutes((self.config.block_duration_minutes * 2) as i64)),
                    });

                    // Block user agent if available
                    if let Some(user_agent) = self.extract_user_agent_from_analysis(analysis) {
                        let ua_block_result = self.block_user_agent(&user_agent).await;
                        actions.push(ResponseActionResult {
                            action_type: ResponseActionType::UserAgentBlock,
                            success: ua_block_result.is_ok(),
                            message: "Blocked suspicious user agent".to_string(),
                            duration: Some(Duration::hours(24)),
                        });
                    }
                }
            },
            5 => {
                // Level 5: Permanent IP block + comprehensive blocking
                if let Some(ip_addr) = &ip {
                    let permanent_block_result = self.block_ip(ip_addr, "Persistent attack - permanent block").await;
                    actions.push(ResponseActionResult {
                        action_type: ResponseActionType::IpBlock,
                        success: permanent_block_result.is_ok(),
                        message: "Permanent IP block applied".to_string(),
                        duration: None, // Permanent
                    });

                    // Geographic blocking if from suspicious country
                    if let Some(geo) = &analysis.geographic_analysis {
                        if self.config.suspicious_countries.contains(&geo.country_code) {
                            let geo_block_result = self.apply_geographic_block(&geo.country_code).await;
                            actions.push(ResponseActionResult {
                                action_type: ResponseActionType::GeofenceBlock,
                                success: geo_block_result.is_ok(),
                                message: format!("Applied geographic block for country: {}", geo.country_code),
                                duration: Some(Duration::hours(24)),
                            });
                        }
                    }
                }
            },
            _ => {
                // Default: Basic monitoring
                actions.push(ResponseActionResult {
                    action_type: ResponseActionType::RateLimit,
                    success: true,
                    message: "Enhanced monitoring enabled".to_string(),
                    duration: Some(Duration::minutes(15)),
                });
            }
        }

        Ok(actions)
    }

    /// Execute adaptive response based on ML insights
    async fn execute_adaptive_response(
        &self,
        analysis: &AttackAnalysis,
        pattern_analysis: &PatternAnalysisResult,
    ) -> Result<Vec<ResponseActionResult>, BruteForceError> {
        let mut actions = Vec::new();
        let ip = self.extract_ip_from_analysis(analysis);

        // Analyze pattern-specific responses
        for pattern_match in &pattern_analysis.pattern_matches {
            if let Some(ip_addr) = &ip {
                match pattern_match.pattern_type.as_str() {
                    "credential_stuffing" => {
                        // Respond to credential stuffing with account locking
                        let account_lock_result = self.apply_account_protection(analysis).await;
                        actions.push(ResponseActionResult {
                            action_type: ResponseActionType::AccountLock,
                            success: account_lock_result.is_ok(),
                            message: "Applied account protection measures".to_string(),
                            duration: Some(Duration::hours(1)),
                        });
                    },
                    "password_spraying" => {
                        // Respond to password spraying with CAPTCHA
                        let captcha_result = self.enable_captcha_challenge(ip_addr).await;
                        actions.push(ResponseActionResult {
                            action_type: ResponseActionType::CaptchaChallenge,
                            success: captcha_result.is_ok(),
                            message: "Enabled CAPTCHA for password spraying".to_string(),
                            duration: Some(Duration::hours(4)),
                        });
                    },
                    "distributed_attack" => {
                        // Respond to distributed attacks with network-level blocking
                        let network_block_result = self.apply_network_level_block(ip_addr).await;
                        actions.push(ResponseActionResult {
                            action_type: ResponseActionType::IpBlock,
                            success: network_block_result.is_ok(),
                            message: "Applied network-level blocking".to_string(),
                            duration: Some(Duration::hours(6)),
                        });
                    },
                    "slow_brute_force" => {
                        // Respond to slow attacks with tar pitting
                        let tarpit_result = self.apply_tarpit_response(ip_addr).await;
                        actions.push(ResponseActionResult {
                            action_type: ResponseActionType::TarPit,
                            success: tarpit_result.is_ok(),
                            message: "Applied tar pit response".to_string(),
                            duration: Some(Duration::hours(12)),
                        });
                    },
                    _ => {
                        // Default adaptive response
                        let adaptive_limit_result = self.apply_adaptive_rate_limit(ip_addr, pattern_analysis.ml_confidence).await;
                        actions.push(ResponseActionResult {
                            action_type: ResponseActionType::RateLimit,
                            success: adaptive_limit_result.is_ok(),
                            message: format!("Applied adaptive rate limiting (confidence: {:.2})", pattern_analysis.ml_confidence),
                            duration: Some(Duration::hours(2)),
                        });
                    }
                }
            }
        }

        // If no specific patterns, apply ML-based response
        if actions.is_empty() && pattern_analysis.ml_confidence >= 0.7 {
            if let Some(ip_addr) = &ip {
                let ml_response_result = self.apply_ml_based_response(ip_addr, pattern_analysis.ml_confidence).await;
                actions.push(ResponseActionResult {
                    action_type: ResponseActionType::RateLimit,
                    success: ml_response_result.is_ok(),
                    message: format!("Applied ML-based response (confidence: {:.2})", pattern_analysis.ml_confidence),
                    duration: Some(Duration::hours((pattern_analysis.ml_confidence * 4.0) as i64)),
                });
            }
        }

        Ok(actions)
    }

    /// Generate contextual recommendations
    fn generate_recommendations(&self, analysis: &AttackAnalysis, strategy: &ResponseStrategy) -> Vec<String> {
        let mut recommendations = Vec::new();

        match analysis.threat_level {
            ThreatLevel::Critical => {
                recommendations.push("Consider implementing IP whitelist for known good sources".to_string());
                recommendations.push("Review and strengthen password policies".to_string());
                recommendations.push("Enable MFA for all admin accounts immediately".to_string());
            },
            ThreatLevel::High => {
                recommendations.push("Monitor for continued attack patterns".to_string());
                recommendations.push("Consider implementing additional authentication factors".to_string());
                recommendations.push("Review recent login activity for compromise indicators".to_string());
            },
            ThreatLevel::Medium => {
                recommendations.push("Continue monitoring attack patterns".to_string());
                recommendations.push("Review rate limiting effectiveness".to_string());
            },
            ThreatLevel::Low => {
                recommendations.push("Maintain current monitoring levels".to_string());
            }
        }

        // Strategy-specific recommendations
        match strategy {
            ResponseStrategy::Immediate => {
                recommendations.push("Verify blocking effectiveness within 15 minutes".to_string());
                recommendations.push("Prepare incident response team if attacks continue".to_string());
            },
            ResponseStrategy::Progressive => {
                recommendations.push("Monitor for escalation triggers".to_string());
                recommendations.push("Review escalation thresholds if needed".to_string());
            },
            ResponseStrategy::Adaptive => {
                recommendations.push("Review ML model predictions for accuracy".to_string());
                recommendations.push("Update training data with attack outcomes".to_string());
            },
            ResponseStrategy::ManualReview => {
                recommendations.push("Security team review required within 30 minutes".to_string());
                recommendations.push("Document attack patterns for future automation".to_string());
            },
        }

        // Attack-type specific recommendations
        for attack_type in &analysis.attack_types {
            match attack_type {
                AttackType::CredentialStuffing => {
                    recommendations.push("Check for compromised credentials in breach databases".to_string());
                    recommendations.push("Force password reset for affected accounts".to_string());
                },
                AttackType::PasswordSpraying => {
                    recommendations.push("Review password complexity requirements".to_string());
                    recommendations.push("Consider implementing account lockout policies".to_string());
                },
                AttackType::DistributedAttack => {
                    recommendations.push("Consider implementing CDN-level protection".to_string());
                    recommendations.push("Review network architecture for DDoS protection".to_string());
                },
                AttackType::BotnetAttack => {
                    recommendations.push("Implement challenge-response mechanisms".to_string());
                    recommendations.push("Consider threat intelligence integration".to_string());
                },
                _ => {}
            }
        }

        recommendations
    }

    // Response action implementations

    async fn block_ip(&self, ip: &str, reason: &str) -> Result<(), BruteForceError> {
        let block_key = format!("bf:blocked_ip:{}", ip);
        let block_data = serde_json::json!({
            "blocked_at": Utc::now(),
            "reason": reason,
            "permanent": true
        });

        self.cache.set(&block_key, &block_data, None).await?;
        
        warn!("Permanently blocked IP {} - Reason: {}", ip, reason);
        Ok(())
    }

    async fn block_ip_temporary(&self, ip: &str, duration_minutes: u32, reason: &str) -> Result<(), BruteForceError> {
        let block_key = format!("bf:blocked_ip:{}", ip);
        let blocked_until = Utc::now() + Duration::minutes(duration_minutes as i64);
        let block_data = serde_json::json!({
            "blocked_at": Utc::now(),
            "blocked_until": blocked_until,
            "reason": reason,
            "permanent": false
        });

        self.cache.set(&block_key, &block_data, Some(duration_minutes as i64 * 60)).await?;
        
        warn!("Temporarily blocked IP {} for {} minutes - Reason: {}", ip, duration_minutes, reason);
        Ok(())
    }

    async fn apply_rate_limit(&self, ip: &str, requests: u32, window_seconds: u32) -> Result<(), BruteForceError> {
        let rate_limit_key = format!("bf:rate_limit:{}", ip);
        let rate_limit_data = serde_json::json!({
            "max_requests": requests,
            "window_seconds": window_seconds,
            "applied_at": Utc::now()
        });

        self.cache.set(&rate_limit_key, &rate_limit_data, Some(window_seconds as i64)).await?;
        
        info!("Applied rate limit to IP {}: {} requests per {} seconds", ip, requests, window_seconds);
        Ok(())
    }

    async fn apply_strict_rate_limit(&self, ip: &str) -> Result<(), BruteForceError> {
        self.apply_rate_limit(ip, 1, 300).await // 1 request per 5 minutes
    }

    async fn enable_captcha_challenge(&self, ip: &str) -> Result<(), BruteForceError> {
        let captcha_key = format!("bf:captcha_required:{}", ip);
        let captcha_data = serde_json::json!({
            "enabled_at": Utc::now(),
            "challenge_level": "standard"
        });

        self.cache.set(&captcha_key, &captcha_data, Some(7200)).await?; // 2 hours
        
        info!("Enabled CAPTCHA challenge for IP {}", ip);
        Ok(())
    }

    async fn block_user_agent(&self, user_agent: &str) -> Result<(), BruteForceError> {
        use std::hash::{Hash, Hasher};
        let mut hasher = std::collections::hash_map::DefaultHasher::new();
        user_agent.hash(&mut hasher);
        let ua_hash = format!("{:x}", hasher.finish());
        let ua_block_key = format!("bf:blocked_ua:{}", ua_hash);
        let block_data = serde_json::json!({
            "user_agent": user_agent,
            "blocked_at": Utc::now(),
            "hash": ua_hash
        });

        self.cache.set(&ua_block_key, &block_data, Some(86400)).await?; // 24 hours
        
        warn!("Blocked user agent: {}", user_agent);
        Ok(())
    }

    async fn apply_geographic_block(&self, country_code: &str) -> Result<(), BruteForceError> {
        let geo_block_key = format!("bf:geo_block:{}", country_code);
        let block_data = serde_json::json!({
            "country_code": country_code,
            "blocked_at": Utc::now(),
            "duration_hours": 24
        });

        self.cache.set(&geo_block_key, &block_data, Some(86400)).await?; // 24 hours
        
        warn!("Applied geographic block for country: {}", country_code);
        Ok(())
    }

    async fn apply_account_protection(&self, _analysis: &AttackAnalysis) -> Result<(), BruteForceError> {
        // This would implement account-specific protection measures
        // For now, just log the action
        info!("Applied account protection measures for attack analysis");
        Ok(())
    }

    async fn apply_network_level_block(&self, ip: &str) -> Result<(), BruteForceError> {
        // This would integrate with network infrastructure
        self.block_ip(ip, "Network-level distributed attack block").await
    }

    async fn apply_tarpit_response(&self, ip: &str) -> Result<(), BruteForceError> {
        let tarpit_key = format!("bf:tarpit:{}", ip);
        let tarpit_data = serde_json::json!({
            "enabled_at": Utc::now(),
            "delay_seconds": 30,
            "max_delay_seconds": 300
        });

        self.cache.set(&tarpit_key, &tarpit_data, Some(43200)).await?; // 12 hours
        
        info!("Applied tar pit response for IP {}", ip);
        Ok(())
    }

    async fn apply_adaptive_rate_limit(&self, ip: &str, confidence: f64) -> Result<(), BruteForceError> {
        let max_requests = ((1.0 - confidence) * 10.0).max(1.0) as u32;
        let window_seconds = (confidence * 300.0).max(60.0) as u32;
        
        self.apply_rate_limit(ip, max_requests, window_seconds).await
    }

    async fn apply_ml_based_response(&self, ip: &str, confidence: f64) -> Result<(), BruteForceError> {
        if confidence >= 0.9 {
            self.block_ip_temporary(ip, 60, "High ML confidence block").await
        } else if confidence >= 0.8 {
            self.apply_strict_rate_limit(ip).await
        } else {
            self.apply_adaptive_rate_limit(ip, confidence).await
        }
    }

    // Helper methods

    fn extract_ip_from_analysis(&self, analysis: &AttackAnalysis) -> Option<String> {
        // Extract IP from risk factors or other analysis data
        for risk_factor in &analysis.risk_factors {
            if let Ok(evidence) = serde_json::from_value::<HashMap<String, String>>(risk_factor.evidence.clone()) {
                if let Some(ip) = evidence.get("ip") {
                    return Some(ip.clone());
                }
            }
        }
        None
    }

    fn extract_user_agent_from_analysis(&self, analysis: &AttackAnalysis) -> Option<String> {
        // Extract user agent from behavioral analysis or risk factors
        for risk_factor in &analysis.risk_factors {
            if risk_factor.factor_type.contains("Automated Behavior") {
                if let Ok(evidence) = serde_json::from_value::<HashMap<String, String>>(risk_factor.evidence.clone()) {
                    if let Some(ua) = evidence.get("user_agent") {
                        return Some(ua.clone());
                    }
                }
            }
        }
        None
    }

    async fn log_response_actions(
        &self,
        _analysis: &AttackAnalysis,
        actions: &[ResponseActionResult],
    ) -> Result<(), BruteForceError> {
        for action in actions {
            let response_action = ResponseActionRecord {
                id: uuid::Uuid::new_v4(),
                attack_attempt_id: uuid::Uuid::new_v4(), // Would link to actual attack attempt
                action_type: format!("{:?}", action.action_type),
                action_details: serde_json::json!({
                    "message": action.message,
                    "duration": action.duration.map(|d| d.num_seconds()),
                    "success": action.success
                }),
                executed_at: Utc::now(),
                success: action.success,
                error_message: if action.success { None } else { Some("Action failed".to_string()) },
                duration_minutes: action.duration.map(|d| d.num_minutes() as i32),
                automated: true,
                operator_id: None,
                created_at: Utc::now(),
            };

            // TODO: Log to database using Diesel
            // For now, just log the action
            tracing::info!("Response action logged: {:?}", response_action);
            
            // Stub implementation - replace with Diesel insert
            // diesel::insert_into(response_actions::table)
            //     .values(&response_action)
            //     .execute(conn)?;
        }

        info!("Logged {} response actions to database", actions.len());
        Ok(())
    }
}