/**
 * Data Exposure Prevention System
 * 
 * SECURITY CRITICAL: This module provides comprehensive protection against
 * sensitive data exposure in client-side code, API responses, and application state.
 * 
 * Features:
 * - Automatic sensitive data detection and redaction
 * - API response sanitization and filtering
 * - Client-side data masking and obfuscation
 * - Real-time data leakage monitoring and prevention
 * - Comprehensive audit logging for sensitive data access
 * - Configurable sensitivity levels and protection policies
 */

import { permissionErrorAnalytics } from '@/lib/analytics/permission-error-analytics'

// Data sensitivity classification
export type DataSensitivityLevel = 'public' | 'internal' | 'confidential' | 'restricted' | 'top_secret'

// Data exposure threat types
export type DataExposureThreatType = 
  | 'sensitive_data_exposed'
  | 'api_response_leakage'
  | 'client_state_exposure'
  | 'logging_exposure'
  | 'debug_data_leak'
  | 'storage_exposure'
  | 'network_exposure'

// Data exposure event
export interface DataExposureEvent {
  type: DataExposureThreatType
  severity: 'low' | 'medium' | 'high' | 'critical'
  timestamp: number
  userId?: string
  component?: string
  details: {
    sensitiveFields: string[]
    exposureVector: string
    dataType: string
    originalData?: any
    redactedData?: any
    accessPattern: string
    userAgent?: string
    ipAddress?: string
  }
  action: 'redacted' | 'blocked' | 'logged' | 'quarantined'
  complianceImpact: 'none' | 'gdpr' | 'hipaa' | 'pci' | 'sox' | 'multiple'
}

// Sensitive data pattern configuration
export interface SensitiveDataPattern {
  name: string
  pattern: RegExp | string
  sensitivityLevel: DataSensitivityLevel
  category: 'personal' | 'financial' | 'authentication' | 'system' | 'business'
  redactionStrategy: 'mask' | 'truncate' | 'hash' | 'remove' | 'encrypt'
  complianceFlags: ('gdpr' | 'hipaa' | 'pci' | 'sox')[]
  description: string
}

// Data protection policy
export interface DataProtectionPolicy {
  name: string
  sensitivityThreshold: DataSensitivityLevel
  enableRealTimeMonitoring: boolean
  enableAuditLogging: boolean
  allowedExposureVectors: string[]
  redactionLevel: 'minimal' | 'moderate' | 'aggressive' | 'total'
  complianceMode: boolean
  emergencyBypass: boolean
}

// Data sanitization result
export interface DataSanitizationResult {
  sanitizedData: any
  exposureEvents: DataExposureEvent[]
  redactedFields: string[]
  sensitivityScore: number
  complianceStatus: 'compliant' | 'violation' | 'warning'
  recommendedActions: string[]
}

// Secure data container
export interface SecureDataContainer<T = any> {
  data: T
  metadata: {
    sensitivityLevel: DataSensitivityLevel
    lastAccessed: number
    accessCount: number
    userId?: string
    component?: string
    encryptionKey?: string
    expiresAt?: number
  }
  accessLog: DataAccessRecord[]
  integrityHash: string
}

// Data access record
export interface DataAccessRecord {
  timestamp: number
  userId?: string
  component?: string
  accessType: 'read' | 'write' | 'modify' | 'copy' | 'export'
  sensitiveFieldsAccessed: string[]
  permitted: boolean
  reason?: string
}

/**
 * Data Exposure Prevention Manager
 * Comprehensive protection against sensitive data leakage
 */
export class DataExposurePreventionManager {
  private static instance: DataExposurePreventionManager
  private sensitivePatterns: Map<string, SensitiveDataPattern>
  private protectionPolicies: Map<string, DataProtectionPolicy>
  private exposureEvents: DataExposureEvent[]
  private secureContainers: Map<string, SecureDataContainer>
  private dataAccessAudit: DataAccessRecord[]

  constructor() {
    this.sensitivePatterns = new Map()
    this.protectionPolicies = new Map()
    this.exposureEvents = []
    this.secureContainers = new Map()
    this.dataAccessAudit = []

    this.initializeSensitivePatterns()
    this.initializeProtectionPolicies()
    this.startDataExpirationCleanup()
  }

  static getInstance(): DataExposurePreventionManager {
    if (!DataExposurePreventionManager.instance) {
      DataExposurePreventionManager.instance = new DataExposurePreventionManager()
    }
    return DataExposurePreventionManager.instance
  }

  /**
   * Initialize comprehensive sensitive data patterns
   */
  private initializeSensitivePatterns(): void {
    // Personal Identifiable Information (PII)
    this.addSensitivePattern({
      name: 'email_address',
      pattern: /[a-zA-Z0-9._%+-]+@[a-zA-Z0-9.-]+\.[a-zA-Z]{2,}/g,
      sensitivityLevel: 'confidential',
      category: 'personal',
      redactionStrategy: 'mask',
      complianceFlags: ['gdpr'],
      description: 'Email addresses - GDPR protected'
    })

    this.addSensitivePattern({
      name: 'phone_number',
      pattern: /(\+\d{1,3}[-.\s]?)?(\(?\d{3}\)?[-.\s]?)?\d{3}[-.\s]?\d{4}/g,
      sensitivityLevel: 'confidential',
      category: 'personal',
      redactionStrategy: 'mask',
      complianceFlags: ['gdpr'],
      description: 'Phone numbers - personal contact information'
    })

    this.addSensitivePattern({
      name: 'ssn',
      pattern: /\b\d{3}-?\d{2}-?\d{4}\b/g,
      sensitivityLevel: 'restricted',
      category: 'personal',
      redactionStrategy: 'remove',
      complianceFlags: ['gdpr', 'hipaa'],
      description: 'Social Security Numbers - highly sensitive'
    })

    // Financial Data
    this.addSensitivePattern({
      name: 'credit_card',
      pattern: /\b(?:\d{4}[-\s]?){3}\d{4}\b/g,
      sensitivityLevel: 'restricted',
      category: 'financial',
      redactionStrategy: 'mask',
      complianceFlags: ['pci'],
      description: 'Credit card numbers - PCI DSS protected'
    })

    this.addSensitivePattern({
      name: 'bank_account',
      pattern: /\b\d{8,17}\b/g,
      sensitivityLevel: 'restricted',
      category: 'financial',
      redactionStrategy: 'mask',
      complianceFlags: ['pci', 'sox'],
      description: 'Bank account numbers'
    })

    // Authentication Credentials
    this.addSensitivePattern({
      name: 'api_key',
      pattern: /(?:api[_-]?key|apikey|access[_-]?token)[\"\':\s=]*[\"\']*([a-zA-Z0-9]{20,})[\"']*/gi,
      sensitivityLevel: 'top_secret',
      category: 'authentication',
      redactionStrategy: 'remove',
      complianceFlags: ['sox'],
      description: 'API keys and access tokens'
    })

    this.addSensitivePattern({
      name: 'password',
      pattern: /(?:password|pwd|pass)[\"\':\s=]*[\"\']*([^\"\'\\s]{6,})[\"']*/gi,
      sensitivityLevel: 'top_secret',
      category: 'authentication',
      redactionStrategy: 'remove',
      complianceFlags: ['gdpr', 'sox'],
      description: 'Passwords and authentication secrets'
    })

    this.addSensitivePattern({
      name: 'jwt_token',
      pattern: /eyJ[a-zA-Z0-9_-]*\.[a-zA-Z0-9_-]*\.[a-zA-Z0-9_-]*/g,
      sensitivityLevel: 'restricted',
      category: 'authentication',
      redactionStrategy: 'truncate',
      complianceFlags: ['gdpr'],
      description: 'JWT tokens - authentication sensitive'
    })

    // System Information
    this.addSensitivePattern({
      name: 'ip_address',
      pattern: /\b(?:[0-9]{1,3}\.){3}[0-9]{1,3}\b/g,
      sensitivityLevel: 'internal',
      category: 'system',
      redactionStrategy: 'mask',
      complianceFlags: ['gdpr'],
      description: 'IP addresses - potentially identifying'
    })

    this.addSensitivePattern({
      name: 'database_url',
      pattern: /(?:mongodb|mysql|postgres|sqlite):\/\/[^\\s\"\']+/gi,
      sensitivityLevel: 'restricted',
      category: 'system',
      redactionStrategy: 'remove',
      complianceFlags: ['sox'],
      description: 'Database connection strings'
    })

    // Business Sensitive
    this.addSensitivePattern({
      name: 'permission_admin',
      pattern: /admin:\*:\*/g,
      sensitivityLevel: 'restricted',
      category: 'business',
      redactionStrategy: 'hash',
      complianceFlags: ['sox'],
      description: 'Administrative permissions'
    })

    this.addSensitivePattern({
      name: 'user_id_internal',
      pattern: /usr_[a-zA-Z0-9]{10,}/g,
      sensitivityLevel: 'confidential',
      category: 'business',
      redactionStrategy: 'truncate',
      complianceFlags: ['gdpr'],
      description: 'Internal user identifiers'
    })
  }

  /**
   * Initialize data protection policies
   */
  private initializeProtectionPolicies(): void {
    // Development policy - more permissive for debugging
    this.protectionPolicies.set('development', {
      name: 'development',
      sensitivityThreshold: 'internal',
      enableRealTimeMonitoring: true,
      enableAuditLogging: true,
      allowedExposureVectors: ['console', 'debug', 'api_response'],
      redactionLevel: 'moderate',
      complianceMode: false,
      emergencyBypass: true
    })

    // Production policy - strict protection
    this.protectionPolicies.set('production', {
      name: 'production',
      sensitivityThreshold: 'confidential',
      enableRealTimeMonitoring: true,
      enableAuditLogging: true,
      allowedExposureVectors: ['api_response'],
      redactionLevel: 'aggressive',
      complianceMode: true,
      emergencyBypass: false
    })

    // High security policy - maximum protection
    this.protectionPolicies.set('high_security', {
      name: 'high_security',
      sensitivityThreshold: 'internal',
      enableRealTimeMonitoring: true,
      enableAuditLogging: true,
      allowedExposureVectors: [],
      redactionLevel: 'total',
      complianceMode: true,
      emergencyBypass: false
    })
  }

  /**
   * Sanitize data for client-side exposure
   */
  public sanitizeData(
    data: any,
    userId?: string,
    component?: string,
    policyName = 'production'
  ): DataSanitizationResult {
    const policy = this.protectionPolicies.get(policyName) || this.protectionPolicies.get('production')!
    const exposureEvents: DataExposureEvent[] = []
    const redactedFields: string[] = []
    let sensitivityScore = 0

    try {
      const sanitizedData = this.sanitizeObjectRecursive(
        data,
        '',
        policy,
        exposureEvents,
        redactedFields,
        userId,
        component
      )

      // Calculate sensitivity score
      sensitivityScore = this.calculateSensitivityScore(redactedFields, exposureEvents)

      // Determine compliance status
      const complianceStatus = this.assessComplianceStatus(exposureEvents, policy)

      // Generate recommendations
      const recommendedActions = this.generateRecommendations(exposureEvents, sensitivityScore, policy)

      // Log exposure events
      if (policy.enableAuditLogging && exposureEvents.length > 0) {
        exposureEvents.forEach(event => this.logExposureEvent(event))
      }

      return {
        sanitizedData,
        exposureEvents,
        redactedFields,
        sensitivityScore,
        complianceStatus,
        recommendedActions
      }

    } catch (error) {
      console.error('Data sanitization failed:', error)

      const criticalEvent: DataExposureEvent = {
        type: 'sensitive_data_exposed',
        severity: 'critical',
        timestamp: Date.now(),
        userId,
        component,
        details: {
          sensitiveFields: ['sanitization_error'],
          exposureVector: 'system_error',
          dataType: 'unknown',
          originalData: 'sanitization_failed',
          accessPattern: 'error_fallback',
          userAgent: navigator.userAgent
        },
        action: 'quarantined',
        complianceImpact: 'multiple'
      }

      this.logExposureEvent(criticalEvent)

      return {
        sanitizedData: {},
        exposureEvents: [criticalEvent],
        redactedFields: ['all_data_quarantined'],
        sensitivityScore: 100,
        complianceStatus: 'violation',
        recommendedActions: ['Block data access', 'Investigate sanitization failure', 'Emergency security review']
      }
    }
  }

  /**
   * Recursively sanitize object properties
   */
  private sanitizeObjectRecursive(
    obj: any,
    keyPath: string,
    policy: DataProtectionPolicy,
    exposureEvents: DataExposureEvent[],
    redactedFields: string[],
    userId?: string,
    component?: string
  ): any {
    if (obj === null || obj === undefined) {
      return obj
    }

    // Handle different data types
    if (typeof obj === 'string') {
      return this.sanitizeStringValue(obj, keyPath, policy, exposureEvents, redactedFields, userId, component)
    }

    if (typeof obj === 'number' || typeof obj === 'boolean') {
      return obj
    }

    if (Array.isArray(obj)) {
      return obj.map((item, index) => 
        this.sanitizeObjectRecursive(
          item,
          `${keyPath}[${index}]`,
          policy,
          exposureEvents,
          redactedFields,
          userId,
          component
        )
      )
    }

    if (typeof obj === 'object') {
      const sanitized: any = {}
      
      for (const [key, value] of Object.entries(obj)) {
        const currentPath = keyPath ? `${keyPath}.${key}` : key
        
        // Check if field name itself is sensitive
        if (this.isFieldNameSensitive(key)) {
          const event: DataExposureEvent = {
            type: 'sensitive_data_exposed',
            severity: 'medium',
            timestamp: Date.now(),
            userId,
            component,
            details: {
              sensitiveFields: [currentPath],
              exposureVector: 'object_property',
              dataType: 'field_name',
              originalData: key,
              accessPattern: 'object_access',
              userAgent: navigator.userAgent
            },
            action: 'redacted',
            complianceImpact: 'gdpr'
          }

          exposureEvents.push(event)
          redactedFields.push(currentPath)
          
          // Use redacted field name
          const redactedKey = this.redactFieldName(key)
          sanitized[redactedKey] = this.sanitizeObjectRecursive(
            value,
            currentPath,
            policy,
            exposureEvents,
            redactedFields,
            userId,
            component
          )
        } else {
          sanitized[key] = this.sanitizeObjectRecursive(
            value,
            currentPath,
            policy,
            exposureEvents,
            redactedFields,
            userId,
            component
          )
        }
      }
      
      return sanitized
    }

    return obj
  }

  /**
   * Sanitize string values using sensitive patterns
   */
  private sanitizeStringValue(
    value: string,
    keyPath: string,
    policy: DataProtectionPolicy,
    exposureEvents: DataExposureEvent[],
    redactedFields: string[],
    userId?: string,
    component?: string
  ): string {
    let sanitized = value
    const detectedPatterns: SensitiveDataPattern[] = []

    // Check against all sensitive patterns
    for (const [patternName, pattern] of this.sensitivePatterns) {
      const regex = typeof pattern.pattern === 'string' ? 
        new RegExp(pattern.pattern, 'gi') : pattern.pattern

      if (regex.test(value)) {
        detectedPatterns.push(pattern)

        // Check if pattern meets policy threshold
        if (this.meetsSensitivityThreshold(pattern.sensitivityLevel, policy.sensitivityThreshold)) {
          // Apply redaction strategy
          sanitized = this.applyRedactionStrategy(sanitized, pattern, regex)

          // Log exposure event
          const event: DataExposureEvent = {
            type: 'sensitive_data_exposed',
            severity: this.getSeverityFromSensitivity(pattern.sensitivityLevel),
            timestamp: Date.now(),
            userId,
            component,
            details: {
              sensitiveFields: [keyPath],
              exposureVector: 'string_content',
              dataType: pattern.category,
              originalData: policy.complianceMode ? '[REDACTED]' : value,
              redactedData: sanitized,
              accessPattern: 'direct_access',
              userAgent: navigator.userAgent
            },
            action: this.getActionFromStrategy(pattern.redactionStrategy),
            complianceImpact: pattern.complianceFlags.length > 0 ? 
              (pattern.complianceFlags.length > 1 ? 'multiple' : pattern.complianceFlags[0]) : 'none'
          }

          exposureEvents.push(event)
          redactedFields.push(keyPath)

          // Real-time monitoring alert
          if (policy.enableRealTimeMonitoring && pattern.sensitivityLevel === 'top_secret') {
            this.triggerRealTimeAlert(event)
          }
        }
      }
    }

    return sanitized
  }

  /**
   * Apply redaction strategy to sensitive data
   */
  private applyRedactionStrategy(
    value: string,
    pattern: SensitiveDataPattern,
    regex: RegExp
  ): string {
    switch (pattern.redactionStrategy) {
      case 'mask':
        return value.replace(regex, (match) => {
          if (match.length <= 4) return '*'.repeat(match.length)
          return match.substring(0, 2) + '*'.repeat(match.length - 4) + match.substring(match.length - 2)
        })

      case 'truncate':
        return value.replace(regex, (match) => match.substring(0, Math.min(8, match.length)) + '...')

      case 'hash':
        return value.replace(regex, (match) => `[HASH:${this.simpleHash(match)}]`)

      case 'remove':
        return value.replace(regex, '[REDACTED]')

      case 'encrypt':
        return value.replace(regex, (match) => `[ENCRYPTED:${btoa(match).slice(0, 16)}...]`)

      default:
        return value.replace(regex, '[PROTECTED]')
    }
  }

  /**
   * Create secure data container
   */
  public createSecureContainer<T>(
    data: T,
    sensitivityLevel: DataSensitivityLevel,
    userId?: string,
    component?: string,
    expirationMinutes = 60
  ): string {
    const containerId = this.generateContainerId()
    const encryptionKey = this.generateEncryptionKey()
    const expiresAt = Date.now() + (expirationMinutes * 60 * 1000)

    const container: SecureDataContainer<T> = {
      data,
      metadata: {
        sensitivityLevel,
        lastAccessed: Date.now(),
        accessCount: 0,
        userId,
        component,
        encryptionKey,
        expiresAt
      },
      accessLog: [],
      integrityHash: this.generateIntegrityHash(data, encryptionKey)
    }

    this.secureContainers.set(containerId, container)

    // Log container creation
    permissionErrorAnalytics.trackEvent('secure_container_created', {
      container_id: containerId,
      user_id: userId,
      component,
      sensitivity_level: sensitivityLevel,
      expires_at: expiresAt
    })

    return containerId
  }

  /**
   * Access secure container with audit logging
   */
  public accessSecureContainer<T>(
    containerId: string,
    userId?: string,
    component?: string,
    accessType: 'read' | 'write' | 'modify' | 'copy' | 'export' = 'read'
  ): { data: T | null; permitted: boolean; reason?: string } {
    const container = this.secureContainers.get(containerId)

    if (!container) {
      return { data: null, permitted: false, reason: 'Container not found' }
    }

    // Check expiration
    if (container.metadata.expiresAt && Date.now() > container.metadata.expiresAt) {
      this.secureContainers.delete(containerId)
      return { data: null, permitted: false, reason: 'Container expired' }
    }

    // Verify integrity
    const currentHash = this.generateIntegrityHash(container.data, container.metadata.encryptionKey!)
    if (currentHash !== container.integrityHash) {
      this.secureContainers.delete(containerId)
      
      // Log integrity violation
      const event: DataExposureEvent = {
        type: 'storage_exposure',
        severity: 'critical',
        timestamp: Date.now(),
        userId,
        component,
        details: {
          sensitiveFields: ['container_data'],
          exposureVector: 'integrity_violation',
          dataType: 'secure_container',
          accessPattern: 'container_access',
          userAgent: navigator.userAgent
        },
        action: 'quarantined',
        complianceImpact: 'multiple'
      }

      this.logExposureEvent(event)
      return { data: null, permitted: false, reason: 'Integrity violation detected' }
    }

    // Update access metadata
    container.metadata.lastAccessed = Date.now()
    container.metadata.accessCount++

    // Log access
    const accessRecord: DataAccessRecord = {
      timestamp: Date.now(),
      userId,
      component,
      accessType,
      sensitiveFieldsAccessed: ['container_data'],
      permitted: true
    }

    container.accessLog.push(accessRecord)
    this.dataAccessAudit.push(accessRecord)

    return { data: container.data as T, permitted: true }
  }

  /**
   * Utility functions
   */
  private addSensitivePattern(pattern: SensitiveDataPattern): void {
    this.sensitivePatterns.set(pattern.name, pattern)
  }

  private isFieldNameSensitive(fieldName: string): boolean {
    const sensitiveFieldNames = [
      'password', 'secret', 'key', 'token', 'credential',
      'ssn', 'social_security', 'credit_card', 'cvv',
      'private_key', 'api_key', 'access_token'
    ]

    return sensitiveFieldNames.some(sensitive => 
      fieldName.toLowerCase().includes(sensitive.toLowerCase())
    )
  }

  private redactFieldName(fieldName: string): string {
    if (fieldName.length <= 3) return '***'
    return fieldName.substring(0, 1) + '*'.repeat(fieldName.length - 2) + fieldName.substring(fieldName.length - 1)
  }

  private meetsSensitivityThreshold(
    patternLevel: DataSensitivityLevel,
    threshold: DataSensitivityLevel
  ): boolean {
    const levels = ['public', 'internal', 'confidential', 'restricted', 'top_secret']
    return levels.indexOf(patternLevel) >= levels.indexOf(threshold)
  }

  private getSeverityFromSensitivity(level: DataSensitivityLevel): 'low' | 'medium' | 'high' | 'critical' {
    switch (level) {
      case 'top_secret': return 'critical'
      case 'restricted': return 'high'
      case 'confidential': return 'medium'
      default: return 'low'
    }
  }

  private getActionFromStrategy(strategy: string): 'redacted' | 'blocked' | 'logged' | 'quarantined' {
    switch (strategy) {
      case 'remove': return 'blocked'
      case 'encrypt': return 'quarantined'
      default: return 'redacted'
    }
  }

  private calculateSensitivityScore(redactedFields: string[], events: DataExposureEvent[]): number {
    let score = 0
    
    score += redactedFields.length * 10
    score += events.filter(e => e.severity === 'critical').length * 25
    score += events.filter(e => e.severity === 'high').length * 15
    score += events.filter(e => e.severity === 'medium').length * 10
    score += events.filter(e => e.severity === 'low').length * 5

    return Math.min(100, score)
  }

  private assessComplianceStatus(
    events: DataExposureEvent[],
    policy: DataProtectionPolicy
  ): 'compliant' | 'violation' | 'warning' {
    if (!policy.complianceMode) return 'compliant'

    const violations = events.filter(e => 
      e.complianceImpact !== 'none' && (e.severity === 'critical' || e.severity === 'high')
    )

    if (violations.length > 0) return 'violation'

    const warnings = events.filter(e => e.complianceImpact !== 'none')
    return warnings.length > 0 ? 'warning' : 'compliant'
  }

  private generateRecommendations(
    events: DataExposureEvent[],
    sensitivityScore: number,
    policy: DataProtectionPolicy
  ): string[] {
    const recommendations: string[] = []

    if (sensitivityScore > 50) {
      recommendations.push('High sensitivity data detected - review data handling practices')
    }

    if (events.some(e => e.complianceImpact !== 'none')) {
      recommendations.push('Compliance-sensitive data detected - ensure proper handling')
    }

    if (events.some(e => e.severity === 'critical')) {
      recommendations.push('Critical data exposure risks - implement additional protection')
    }

    if (!policy.complianceMode) {
      recommendations.push('Consider enabling compliance mode for better protection')
    }

    return recommendations
  }

  private generateContainerId(): string {
    return 'sec_' + Date.now().toString(36) + '_' + Math.random().toString(36).substr(2, 9)
  }

  private generateEncryptionKey(): string {
    return btoa(Date.now() + Math.random().toString()).slice(0, 32)
  }

  private generateIntegrityHash(data: any, key: string): string {
    const dataString = JSON.stringify(data)
    return btoa(`${dataString}:${key}`).slice(0, 20)
  }

  private simpleHash(input: string): string {
    let hash = 0
    for (let i = 0; i < input.length; i++) {
      hash = ((hash << 5) - hash + input.charCodeAt(i)) & 0xffffffff
    }
    return Math.abs(hash).toString(36)
  }

  private triggerRealTimeAlert(event: DataExposureEvent): void {
    console.error('🚨 CRITICAL DATA EXPOSURE:', event)
    
    // In production, this would trigger immediate alerts
    permissionErrorAnalytics.trackEvent('critical_data_exposure_alert', {
      type: event.type,
      user_id: event.userId,
      component: event.component,
      severity: event.severity,
      compliance_impact: event.complianceImpact
    })
  }

  private logExposureEvent(event: DataExposureEvent): void {
    this.exposureEvents.push(event)

    // Keep only recent events
    if (this.exposureEvents.length > 1000) {
      this.exposureEvents = this.exposureEvents.slice(-1000)
    }

    // Log to analytics
    permissionErrorAnalytics.trackEvent('data_exposure_event', {
      type: event.type,
      severity: event.severity,
      user_id: event.userId,
      component: event.component,
      compliance_impact: event.complianceImpact,
      action: event.action
    })
  }

  private startDataExpirationCleanup(): void {
    setInterval(() => {
      const now = Date.now()
      const expiredContainers: string[] = []

      for (const [containerId, container] of this.secureContainers) {
        if (container.metadata.expiresAt && container.metadata.expiresAt < now) {
          expiredContainers.push(containerId)
        }
      }

      expiredContainers.forEach(id => this.secureContainers.delete(id))
    }, 5 * 60 * 1000) // Clean up every 5 minutes
  }

  /**
   * Get data exposure statistics
   */
  public getDataExposureStatistics(): {
    totalExposureEvents: number
    criticalEvents: number
    complianceViolations: number
    activeSecureContainers: number
    sensitiveDataDetections: number
  } {
    const criticalEvents = this.exposureEvents.filter(e => e.severity === 'critical').length
    const complianceViolations = this.exposureEvents.filter(e => e.complianceImpact !== 'none').length
    const sensitiveDataDetections = this.exposureEvents.filter(e => e.type === 'sensitive_data_exposed').length

    return {
      totalExposureEvents: this.exposureEvents.length,
      criticalEvents,
      complianceViolations,
      activeSecureContainers: this.secureContainers.size,
      sensitiveDataDetections
    }
  }
}

// Singleton instance
export const dataExposurePreventionManager = DataExposurePreventionManager.getInstance()

// Convenience functions
export const sanitizeApiResponse = (response: any, userId?: string, component?: string) => {
  return dataExposurePreventionManager.sanitizeData(response, userId, component, 'production')
}

export const sanitizeForLogging = (data: any, userId?: string, component?: string) => {
  return dataExposurePreventionManager.sanitizeData(data, userId, component, 'high_security')
}

export const createSecureStorage = <T>(data: T, sensitivity: DataSensitivityLevel, userId?: string) => {
  return dataExposurePreventionManager.createSecureContainer(data, sensitivity, userId)
}

export const accessSecureStorage = <T>(containerId: string, userId?: string, component?: string) => {
  return dataExposurePreventionManager.accessSecureContainer<T>(containerId, userId, component)
}

export default dataExposurePreventionManager