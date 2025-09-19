// ============================================================================
// SHARED PERMISSION HEALTH UTILITIES
// ============================================================================
// Permission health monitoring, scoring, and analytics utilities

import { 
  GranularPermissionClaim, 
  PermissionHealthInfo, 
  UserPermissionSummary
} from '../types/core'
import { EnhancedUserClaims } from '../types/claims'
import { isClaimValid, isClaimExpiringSoon, countPermissionsByExpiryStatus } from './expiry'
import { HEALTH_SCORES, EXPIRY_THRESHOLDS } from '../constants'

// ============================================================================
// PERMISSION HEALTH CALCULATION
// ============================================================================

/**
 * Calculate permission health info for granular permissions
 */
export const calculatePermissionHealth = (
  permissions: Record<string, GranularPermissionClaim>
): PermissionHealthInfo => {
  const now = Date.now()
  const twentyFourHoursFromNow = now + (EXPIRY_THRESHOLDS.EXPIRING_SOON_HOURS * 60 * 60 * 1000)

  const allPermissions = Object.entries(permissions)
  const activePermissions = allPermissions.filter(([_, claim]) => isClaimValid(claim))
  const expiredPermissions = allPermissions.filter(([_, claim]) => !isClaimValid(claim))
  const expiringSoonPermissions = activePermissions.filter(([_, claim]) => 
    isClaimExpiringSoon(claim, EXPIRY_THRESHOLDS.EXPIRING_SOON_HOURS)
  )

  // Find next expiring permission
  const nextExpiry = activePermissions
    .filter(([_, claim]) => claim.expires_at)
    .sort(([_, a], [__, b]) => (a.expires_at || 0) - (b.expires_at || 0))[0]

  const timeUntilNextExpiry = nextExpiry?.[1].expires_at ? 
    (nextExpiry[1].expires_at * 1000) - now : undefined

  // Calculate health score (0-100)
  const healthScore = calculateHealthScore(permissions)

  return {
    total_permissions: allPermissions.length,
    active_permissions: activePermissions.length,
    expired_permissions: expiredPermissions.length,
    expiring_soon_permissions: expiringSoonPermissions.length,
    next_expiry: nextExpiry?.[1].expires_at,
    time_until_next_expiry: timeUntilNextExpiry,
    health_score: healthScore
  }
}

/**
 * Calculate health score for permissions (0-100)
 */
export const calculateHealthScore = (
  permissions: Record<string, GranularPermissionClaim>
): number => {
  const counts = countPermissionsByExpiryStatus(permissions)
  
  if (counts.total === 0) {
    return 100 // No permissions means perfect health
  }

  let score = 100
  
  // Deduct points for expired permissions
  const expiredRatio = counts.expired / counts.total
  score -= expiredRatio * 40 // Up to 40 points deduction
  
  // Deduct points for expiring soon permissions
  const activeCount = counts.total - counts.expired
  const expiringSoonRatio = counts.expiringSoon / activeCount
  if (activeCount > 0) {
    score -= expiringSoonRatio * 30 // Up to 30 points deduction
  }
  
  // Bonus for having permanent permissions
  const permanentRatio = counts.permanent / counts.total
  score += permanentRatio * 10 // Up to 10 bonus points
  
  // Ensure score is between 0 and 100
  return Math.max(0, Math.min(100, Math.floor(score)))
}

/**
 * Get health status category based on score
 */
export const getHealthStatus = (healthScore: number): 'excellent' | 'good' | 'warning' | 'critical' => {
  if (healthScore >= HEALTH_SCORES.EXCELLENT) return 'excellent'
  if (healthScore >= HEALTH_SCORES.GOOD) return 'good'
  if (healthScore >= HEALTH_SCORES.WARNING) return 'warning'
  return 'critical'
}

/**
 * Get health status with recommendations
 */
export const getHealthStatusWithRecommendations = (
  permissions: Record<string, GranularPermissionClaim>
): {
  status: 'excellent' | 'good' | 'warning' | 'critical'
  score: number
  recommendations: string[]
  issues: string[]
} => {
  const health = calculatePermissionHealth(permissions)
  const status = getHealthStatus(health.health_score)
  const counts = countPermissionsByExpiryStatus(permissions)
  
  const recommendations: string[] = []
  const issues: string[] = []
  
  // Generate recommendations based on health status
  if (counts.expired > 0) {
    issues.push(`${counts.expired} expired permission(s)`)
    recommendations.push('Clean up expired permissions to improve security')
  }
  
  if (counts.expiringSoon > 0) {
    issues.push(`${counts.expiringSoon} permission(s) expiring within 24 hours`)
    recommendations.push('Review and extend permissions that are expiring soon')
  }
  
  if (counts.permanent === 0 && counts.total > 0) {
    recommendations.push('Consider making critical permissions permanent to avoid unexpected expiry')
  }
  
  const temporaryRatio = counts.temporary / counts.total
  if (temporaryRatio > 0.8 && counts.total > 5) {
    recommendations.push('High ratio of temporary permissions may indicate over-provisioning')
  }
  
  if (health.time_until_next_expiry && health.time_until_next_expiry < (2 * 60 * 60 * 1000)) {
    issues.push('Critical: Permission expires within 2 hours')
    recommendations.push('Immediate action required to prevent service disruption')
  }
  
  return {
    status,
    score: health.health_score,
    recommendations,
    issues
  }
}

// ============================================================================
// USER PERMISSION SUMMARY
// ============================================================================

/**
 * Get comprehensive permission summary for a user
 */
export const getUserPermissionSummary = (
  userClaims: EnhancedUserClaims
): UserPermissionSummary => {
  const counts = countPermissionsByExpiryStatus(userClaims.permissions)
  const health = calculatePermissionHealth(userClaims.permissions)
  
  return {
    user_id: userClaims.sub,
    total_permissions: counts.total,
    permanent_permissions: counts.permanent,
    temporary_permissions: counts.temporary,
    expired_permissions: counts.expired,
    expiring_soon_permissions: counts.expiringSoon,
    permission_hash: userClaims.permission_hash,
    permission_version: userClaims.permission_version,
    last_updated: Date.now()
  }
}

// ============================================================================
// SYSTEM HEALTH MONITORING
// ============================================================================

/**
 * Calculate system-wide permission health
 */
export interface SystemPermissionHealth {
  overall_score: number
  status: 'excellent' | 'good' | 'warning' | 'critical'
  total_users: number
  users_with_permissions: number
  total_permissions: number
  healthy_users: number
  users_needing_attention: number
  critical_issues: string[]
  recommendations: string[]
  distribution: {
    permanent_permissions: number
    temporary_permissions: number
    expired_permissions: number
    expiring_soon_permissions: number
  }
}

export const calculateSystemPermissionHealth = (
  userPermissions: Array<{ userId: string; permissions: Record<string, GranularPermissionClaim> }>
): SystemPermissionHealth => {
  let totalPermissions = 0
  let totalPermanent = 0
  let totalTemporary = 0
  let totalExpired = 0
  let totalExpiringSoon = 0
  let healthyUsers = 0
  let usersNeedingAttention = 0
  
  const userHealthScores: number[] = []
  const criticalIssues: string[] = []
  const recommendations: string[] = []
  
  for (const { userId, permissions } of userPermissions) {
    const counts = countPermissionsByExpiryStatus(permissions)
    const healthInfo = calculatePermissionHealth(permissions)
    const healthStatus = getHealthStatusWithRecommendations(permissions)
    
    totalPermissions += counts.total
    totalPermanent += counts.permanent
    totalTemporary += counts.temporary
    totalExpired += counts.expired
    totalExpiringSoon += counts.expiringSoon
    
    userHealthScores.push(healthInfo.health_score)
    
    if (healthStatus.status === 'excellent' || healthStatus.status === 'good') {
      healthyUsers++
    } else {
      usersNeedingAttention++
      
      if (healthStatus.status === 'critical') {
        criticalIssues.push(`User ${userId}: ${healthStatus.issues.join(', ')}`)
      }
    }
    
    // Collect unique recommendations
    for (const rec of healthStatus.recommendations) {
      if (!recommendations.includes(rec)) {
        recommendations.push(rec)
      }
    }
  }
  
  // Calculate overall system score
  const averageUserScore = userHealthScores.length > 0 
    ? userHealthScores.reduce((sum, score) => sum + score, 0) / userHealthScores.length
    : 100
  
  // Apply system-level penalties
  let systemScore = averageUserScore
  
  // Penalty for high ratio of users needing attention
  const needAttentionRatio = usersNeedingAttention / userPermissions.length
  if (needAttentionRatio > 0.2) {
    systemScore -= 20 // 20% of users need attention is concerning
  }
  
  // Penalty for expired permissions
  const expiredRatio = totalExpired / totalPermissions
  if (expiredRatio > 0.1) {
    systemScore -= 15 // More than 10% expired is problematic
  }
  
  const overallScore = Math.max(0, Math.min(100, Math.floor(systemScore)))
  
  return {
    overall_score: overallScore,
    status: getHealthStatus(overallScore),
    total_users: userPermissions.length,
    users_with_permissions: userPermissions.filter(up => Object.keys(up.permissions).length > 0).length,
    total_permissions: totalPermissions,
    healthy_users: healthyUsers,
    users_needing_attention: usersNeedingAttention,
    critical_issues: criticalIssues,
    recommendations: recommendations,
    distribution: {
      permanent_permissions: totalPermanent,
      temporary_permissions: totalTemporary,
      expired_permissions: totalExpired,
      expiring_soon_permissions: totalExpiringSoon
    }
  }
}

// ============================================================================
// HEALTH TREND ANALYSIS
// ============================================================================

/**
 * Permission health trend over time
 */
export interface HealthTrend {
  timestamp: number
  health_score: number
  total_permissions: number
  expired_permissions: number
  expiring_soon_permissions: number
}

/**
 * Analyze health trend
 */
export const analyzeHealthTrend = (trends: HealthTrend[]): {
  direction: 'improving' | 'declining' | 'stable'
  rate_of_change: number
  predictions: {
    next_week_score: number
    risk_level: 'low' | 'medium' | 'high'
    action_required: boolean
  }
} => {
  if (trends.length < 2) {
    return {
      direction: 'stable',
      rate_of_change: 0,
      predictions: {
        next_week_score: trends[0]?.health_score || 100,
        risk_level: 'low',
        action_required: false
      }
    }
  }
  
  // Calculate trend direction
  const recent = trends.slice(-3) // Last 3 data points
  const scoreChanges = recent.slice(1).map((trend, i) => 
    trend.health_score - recent[i].health_score
  )
  
  const avgChange = scoreChanges.reduce((sum, change) => sum + change, 0) / scoreChanges.length
  
  let direction: 'improving' | 'declining' | 'stable'
  if (avgChange > 2) direction = 'improving'
  else if (avgChange < -2) direction = 'declining'
  else direction = 'stable'
  
  // Predict next week score
  const currentScore = trends[trends.length - 1].health_score
  const nextWeekScore = Math.max(0, Math.min(100, currentScore + (avgChange * 7)))
  
  // Determine risk level
  let riskLevel: 'low' | 'medium' | 'high'
  if (nextWeekScore >= 80) riskLevel = 'low'
  else if (nextWeekScore >= 60) riskLevel = 'medium'
  else riskLevel = 'high'
  
  const actionRequired = direction === 'declining' && nextWeekScore < 70
  
  return {
    direction,
    rate_of_change: avgChange,
    predictions: {
      next_week_score: Math.floor(nextWeekScore),
      risk_level: riskLevel,
      action_required: actionRequired
    }
  }
}

// ============================================================================
// HEALTH ALERTS
// ============================================================================

/**
 * Generate health alerts based on permission status
 */
export interface HealthAlert {
  id: string
  severity: 'info' | 'warning' | 'error' | 'critical'
  title: string
  message: string
  user_id?: string
  permission?: string
  expires_at?: number
  created_at: number
}

export const generateHealthAlerts = (
  permissions: Record<string, GranularPermissionClaim>,
  userId?: string
): HealthAlert[] => {
  const alerts: HealthAlert[] = []
  const now = Date.now()
  
  for (const [permission, claim] of Object.entries(permissions)) {
    const alertId = `${userId || 'system'}-${permission}-${now}`
    
    // Critical: Expired permission
    if (!isClaimValid(claim)) {
      alerts.push({
        id: alertId + '-expired',
        severity: 'critical',
        title: 'Permission Expired',
        message: `Permission '${permission}' has expired`,
        user_id: userId,
        permission,
        expires_at: claim.expires_at,
        created_at: now
      })
    }
    
    // Warning: Expiring soon
    else if (isClaimExpiringSoon(claim, 24)) {
      alerts.push({
        id: alertId + '-expiring',
        severity: 'warning',
        title: 'Permission Expiring Soon',
        message: `Permission '${permission}' expires within 24 hours`,
        user_id: userId,
        permission,
        expires_at: claim.expires_at,
        created_at: now
      })
    }
    
    // Info: Expiring within a week
    else if (isClaimExpiringSoon(claim, 24 * 7)) {
      alerts.push({
        id: alertId + '-expiring-week',
        severity: 'info',
        title: 'Permission Expiring This Week',
        message: `Permission '${permission}' expires within a week`,
        user_id: userId,
        permission,
        expires_at: claim.expires_at,
        created_at: now
      })
    }
  }
  
  return alerts
}