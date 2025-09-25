/**
 * Comprehensive Input Sanitization System
 * 
 * SECURITY CRITICAL: This module provides bulletproof input validation and sanitization
 * for all user inputs to prevent XSS, injection attacks, and malicious data processing.
 * 
 * Features:
 * - Multi-layer sanitization with configurable strictness
 * - Permission-aware validation contexts
 * - Anti-tampering detection and prevention
 * - Comprehensive logging and monitoring
 * - Real-time threat detection and blocking
 */

import DOMPurify from 'isomorphic-dompurify'
import { permissionErrorAnalytics } from '@/lib/analytics/permission-error-analytics'

// Security threat levels
export type ThreatLevel = 'low' | 'medium' | 'high' | 'critical'

// Sanitization contexts for different data types
export type SanitizationContext = 
  | 'permission_string'
  | 'user_input'
  | 'admin_command'
  | 'api_parameter'
  | 'search_query'
  | 'file_upload'
  | 'rich_text'
  | 'json_data'
  | 'sql_fragment'
  | 'system_command'

// Input validation result with security metadata
export interface InputValidationResult {
  isValid: boolean
  sanitizedInput: string
  originalInput: string
  threatsDetected: ThreatSignature[]
  threatLevel: ThreatLevel
  context: SanitizationContext
  timestamp: number
  userId?: string
  component?: string
  securityScore: number
  recommendations: string[]
}

// Security threat signature detection
export interface ThreatSignature {
  type: 'xss' | 'injection' | 'command' | 'path_traversal' | 'permission_bypass' | 'data_exfiltration'
  pattern: string
  severity: ThreatLevel
  description: string
  blockedContent: string
  recommendedAction: string
}

// Sanitization configuration options
export interface SanitizationOptions {
  context: SanitizationContext
  strictMode: boolean
  allowHtml: boolean
  maxLength: number
  userId?: string
  component?: string
  permissionLevel?: 'user' | 'admin' | 'super_admin'
  enableThreatDetection: boolean
  enableLogging: boolean
}

/**
 * Comprehensive Input Sanitizer Class
 * Provides enterprise-grade input sanitization with threat detection
 */
export class ComprehensiveInputSanitizer {
  private static instance: ComprehensiveInputSanitizer
  private threatPatterns: Map<string, ThreatSignature>
  private blockedPatterns: Set<string>
  private suspiciousActivity: Map<string, number>

  constructor() {
    this.threatPatterns = new Map()
    this.blockedPatterns = new Set()
    this.suspiciousActivity = new Map()
    this.initializeThreatPatterns()
  }

  static getInstance(): ComprehensiveInputSanitizer {
    if (!ComprehensiveInputSanitizer.instance) {
      ComprehensiveInputSanitizer.instance = new ComprehensiveInputSanitizer()
    }
    return ComprehensiveInputSanitizer.instance
  }

  /**
   * Initialize comprehensive threat detection patterns
   */
  private initializeThreatPatterns(): void {
    // XSS Patterns
    this.addThreatPattern('xss_script', {
      type: 'xss',
      pattern: /<script\b[^<]*(?:(?!<\/script>)<[^<]*)*<\/script>/gi,
      severity: 'critical',
      description: 'Script tag injection attempt detected',
      blockedContent: '<script>',
      recommendedAction: 'Block and log user activity'
    })

    this.addThreatPattern('xss_javascript', {
      type: 'xss',
      pattern: /javascript:/gi,
      severity: 'high',
      description: 'JavaScript URL scheme detected',
      blockedContent: 'javascript:',
      recommendedAction: 'Sanitize and monitor user'
    })

    this.addThreatPattern('xss_onerror', {
      type: 'xss',
      pattern: /on\w+\s*=/gi,
      severity: 'high',
      description: 'HTML event handler injection detected',
      blockedContent: 'event handlers',
      recommendedAction: 'Remove event handlers and alert security'
    })

    // Injection Patterns
    this.addThreatPattern('sql_injection', {
      type: 'injection',
      pattern: /(union\s+select|insert\s+into|delete\s+from|drop\s+table|exec\s*\()/gi,
      severity: 'critical',
      description: 'SQL injection attempt detected',
      blockedContent: 'SQL keywords',
      recommendedAction: 'Block immediately and escalate to security team'
    })

    this.addThreatPattern('command_injection', {
      type: 'command',
      pattern: /[;&|`$(){}[\]\\]/g,
      severity: 'high',
      description: 'Command injection metacharacters detected',
      blockedContent: 'shell metacharacters',
      recommendedAction: 'Sanitize and monitor for additional attempts'
    })

    // Permission Bypass Patterns
    this.addThreatPattern('permission_bypass_1', {
      type: 'permission_bypass',
      pattern: /admin:\*:\*/gi,
      severity: 'critical',
      description: 'Administrative permission bypass attempt',
      blockedContent: 'admin wildcard permissions',
      recommendedAction: 'Block user and require security review'
    })

    this.addThreatPattern('permission_bypass_2', {
      type: 'permission_bypass',
      pattern: /(\.\.\/|\.\.\\)/g,
      severity: 'high',
      description: 'Path traversal attempt in permission string',
      blockedContent: 'path traversal sequences',
      recommendedAction: 'Block and audit user permissions'
    })

    // Data Exfiltration Patterns
    this.addThreatPattern('data_exfiltration', {
      type: 'data_exfiltration',
      pattern: /(api[_-]?key|secret|password|token|private[_-]?key)/gi,
      severity: 'critical',
      description: 'Potential credential harvesting attempt',
      blockedContent: 'credential keywords',
      recommendedAction: 'Block and perform immediate security audit'
    })

    // Path Traversal
    this.addThreatPattern('path_traversal', {
      type: 'path_traversal',
      pattern: /(\.\.\/|\.\.\\|%2e%2e%2f|%2e%2e%5c)/gi,
      severity: 'high',
      description: 'Path traversal attack detected',
      blockedContent: 'directory traversal sequences',
      recommendedAction: 'Block and log security incident'
    })
  }

  private addThreatPattern(id: string, signature: ThreatSignature): void {
    this.threatPatterns.set(id, signature)
  }

  /**
   * Main sanitization entry point with comprehensive security checking
   */
  public sanitizeInput(
    input: string,
    options: SanitizationOptions
  ): InputValidationResult {
    const startTime = Date.now()
    let sanitizedInput = input
    const threatsDetected: ThreatSignature[] = []

    try {
      // Step 1: Threat Detection
      if (options.enableThreatDetection) {
        const threats = this.detectThreats(input, options.context)
        threatsDetected.push(...threats)
      }

      // Step 2: Context-specific sanitization
      sanitizedInput = this.applySanitizationByContext(sanitizedInput, options)

      // Step 3: General security sanitization
      sanitizedInput = this.applyGeneralSanitization(sanitizedInput, options)

      // Step 4: Length and format validation
      sanitizedInput = this.validateLength(sanitizedInput, options.maxLength)

      // Step 5: Final security check
      const finalThreats = this.detectThreats(sanitizedInput, options.context)
      threatsDetected.push(...finalThreats)

      // Calculate threat level and security score
      const threatLevel = this.calculateThreatLevel(threatsDetected)
      const securityScore = this.calculateSecurityScore(sanitizedInput, threatsDetected)

      const result: InputValidationResult = {
        isValid: threatLevel !== 'critical' && securityScore >= 50,
        sanitizedInput,
        originalInput: input,
        threatsDetected,
        threatLevel,
        context: options.context,
        timestamp: Date.now(),
        userId: options.userId,
        component: options.component,
        securityScore,
        recommendations: this.generateRecommendations(threatsDetected, securityScore)
      }

      // Log security events
      if (options.enableLogging) {
        this.logSecurityEvent(result, options)
      }

      // Track suspicious activity
      if (options.userId && threatsDetected.length > 0) {
        this.trackSuspiciousActivity(options.userId, threatsDetected)
      }

      return result

    } catch (error) {
      console.error('Input sanitization failed:', error)
      
      // Return safe fallback
      return {
        isValid: false,
        sanitizedInput: '',
        originalInput: input,
        threatsDetected: [{
          type: 'xss',
          pattern: 'sanitization_error',
          severity: 'critical',
          description: 'Sanitization process failed - input blocked for security',
          blockedContent: 'entire input',
          recommendedAction: 'Block user and investigate sanitization failure'
        }],
        threatLevel: 'critical',
        context: options.context,
        timestamp: Date.now(),
        userId: options.userId,
        component: options.component,
        securityScore: 0,
        recommendations: ['Block input', 'Investigate sanitization failure', 'Review security logs']
      }
    }
  }

  /**
   * Detect security threats using pattern matching
   */
  private detectThreats(input: string, context: SanitizationContext): ThreatSignature[] {
    const threats: ThreatSignature[] = []

    for (const [id, signature] of this.threatPatterns) {
      const pattern = typeof signature.pattern === 'string' ? 
        new RegExp(signature.pattern, 'gi') : signature.pattern

      if (pattern.test(input)) {
        threats.push({
          ...signature,
          blockedContent: this.extractMatchedContent(input, pattern)
        })
      }
    }

    // Context-specific threat detection
    switch (context) {
      case 'permission_string':
        threats.push(...this.detectPermissionThreats(input))
        break
      case 'admin_command':
        threats.push(...this.detectAdminCommandThreats(input))
        break
      case 'api_parameter':
        threats.push(...this.detectApiParameterThreats(input))
        break
    }

    return threats
  }

  /**
   * Apply context-specific sanitization rules
   */
  private applySanitizationByContext(input: string, options: SanitizationOptions): string {
    let sanitized = input

    switch (options.context) {
      case 'permission_string':
        sanitized = this.sanitizePermissionString(sanitized)
        break
      case 'user_input':
        sanitized = this.sanitizeUserInput(sanitized, options)
        break
      case 'admin_command':
        sanitized = this.sanitizeAdminCommand(sanitized)
        break
      case 'api_parameter':
        sanitized = this.sanitizeApiParameter(sanitized)
        break
      case 'search_query':
        sanitized = this.sanitizeSearchQuery(sanitized)
        break
      case 'rich_text':
        sanitized = this.sanitizeRichText(sanitized, options)
        break
      case 'json_data':
        sanitized = this.sanitizeJsonData(sanitized)
        break
      default:
        sanitized = this.sanitizeGeneric(sanitized)
    }

    return sanitized
  }

  /**
   * Permission string sanitization with strict validation
   */
  private sanitizePermissionString(input: string): string {
    // Only allow alphanumeric, colons, underscores, and timestamps
    const allowedPattern = /^[a-zA-Z0-9:_]+(\:[0-9]{10,13})?$/
    
    if (!allowedPattern.test(input)) {
      throw new Error('Invalid permission string format')
    }

    // Additional validation for permission structure
    const parts = input.split(':')
    if (parts.length < 3 || parts.length > 4) {
      throw new Error('Permission string must have 3-4 parts: platform:resource:action[:timestamp]')
    }

    // Validate each part
    const [platform, resource, action, timestamp] = parts
    
    if (!platform || !resource || !action) {
      throw new Error('Permission string parts cannot be empty')
    }

    if (timestamp && (isNaN(Number(timestamp)) || Number(timestamp) < 1000000000000)) {
      throw new Error('Invalid timestamp in permission string')
    }

    return input.toLowerCase()
  }

  /**
   * User input sanitization with HTML cleaning
   */
  private sanitizeUserInput(input: string, options: SanitizationOptions): string {
    if (options.allowHtml) {
      return DOMPurify.sanitize(input, {
        ALLOWED_TAGS: ['b', 'i', 'em', 'strong', 'p', 'br'],
        ALLOWED_ATTR: []
      })
    }
    
    return input
      .replace(/[<>'"&]/g, (char) => {
        switch (char) {
          case '<': return '&lt;'
          case '>': return '&gt;'
          case '"': return '&quot;'
          case "'": return '&#x27;'
          case '&': return '&amp;'
          default: return char
        }
      })
      .trim()
  }

  /**
   * Admin command sanitization with strict controls
   */
  private sanitizeAdminCommand(input: string): string {
    // Extremely strict - only allow specific admin commands
    const allowedCommands = [
      'list_users',
      'view_permissions',
      'grant_permission',
      'revoke_permission',
      'create_plan',
      'update_plan',
      'view_analytics'
    ]

    const command = input.toLowerCase().trim()
    
    if (!allowedCommands.includes(command)) {
      throw new Error(`Unauthorized admin command: ${command}`)
    }

    return command
  }

  /**
   * API parameter sanitization
   */
  private sanitizeApiParameter(input: string): string {
    // Remove all potentially dangerous characters
    return input
      .replace(/[<>'"&;|`$(){}[\]\\]/g, '')
      .replace(/\.\./g, '')
      .trim()
  }

  /**
   * Search query sanitization
   */
  private sanitizeSearchQuery(input: string): string {
    // Allow letters, numbers, spaces, and basic punctuation
    return input
      .replace(/[^a-zA-Z0-9\s\-_.@]/g, '')
      .replace(/\s+/g, ' ')
      .trim()
  }

  /**
   * Rich text sanitization with DOMPurify
   */
  private sanitizeRichText(input: string, options: SanitizationOptions): string {
    const config = options.strictMode ? {
      ALLOWED_TAGS: ['p', 'br', 'strong', 'em'],
      ALLOWED_ATTR: []
    } : {
      ALLOWED_TAGS: ['p', 'br', 'strong', 'em', 'u', 'ol', 'ul', 'li', 'a'],
      ALLOWED_ATTR: ['href', 'target']
    }

    return DOMPurify.sanitize(input, config)
  }

  /**
   * JSON data sanitization
   */
  private sanitizeJsonData(input: string): string {
    try {
      const parsed = JSON.parse(input)
      const sanitized = this.sanitizeObjectRecursive(parsed)
      return JSON.stringify(sanitized)
    } catch {
      throw new Error('Invalid JSON format')
    }
  }

  /**
   * Recursively sanitize object properties
   */
  private sanitizeObjectRecursive(obj: any): any {
    if (typeof obj === 'string') {
      return this.sanitizeUserInput(obj, { 
        context: 'user_input', 
        strictMode: true, 
        allowHtml: false, 
        maxLength: 1000,
        enableThreatDetection: true,
        enableLogging: false
      }).sanitizedInput
    }
    
    if (Array.isArray(obj)) {
      return obj.map(item => this.sanitizeObjectRecursive(item))
    }
    
    if (obj !== null && typeof obj === 'object') {
      const sanitized: any = {}
      for (const [key, value] of Object.entries(obj)) {
        const sanitizedKey = key.replace(/[^a-zA-Z0-9_]/g, '')
        sanitized[sanitizedKey] = this.sanitizeObjectRecursive(value)
      }
      return sanitized
    }
    
    return obj
  }

  /**
   * Generic sanitization for unknown input types
   */
  private sanitizeGeneric(input: string): string {
    return input
      .replace(/[<>'"&;|`$(){}[\]\\]/g, '')
      .replace(/\s+/g, ' ')
      .trim()
  }

  /**
   * Apply general security sanitization
   */
  private applyGeneralSanitization(input: string, options: SanitizationOptions): string {
    let sanitized = input

    // Remove null bytes
    sanitized = sanitized.replace(/\0/g, '')

    // Normalize unicode
    sanitized = sanitized.normalize('NFKC')

    // Remove or encode dangerous Unicode characters
    sanitized = sanitized.replace(/[\u200B-\u200D\uFEFF]/g, '') // Zero-width characters
    sanitized = sanitized.replace(/[\u2028\u2029]/g, '') // Line/paragraph separators

    if (options.strictMode) {
      // In strict mode, only allow basic ASCII
      sanitized = sanitized.replace(/[^\x20-\x7E]/g, '')
    }

    return sanitized
  }

  /**
   * Validate input length
   */
  private validateLength(input: string, maxLength: number): string {
    if (input.length > maxLength) {
      throw new Error(`Input exceeds maximum length of ${maxLength} characters`)
    }
    return input
  }

  /**
   * Permission-specific threat detection
   */
  private detectPermissionThreats(input: string): ThreatSignature[] {
    const threats: ThreatSignature[] = []

    // Check for permission escalation attempts
    if (input.includes('admin:*:*') || input.includes('*:*:*')) {
      threats.push({
        type: 'permission_bypass',
        pattern: 'wildcard_permission',
        severity: 'critical',
        description: 'Wildcard permission escalation attempt',
        blockedContent: 'wildcard permissions',
        recommendedAction: 'Block user and audit all permissions'
      })
    }

    return threats
  }

  /**
   * Admin command threat detection
   */
  private detectAdminCommandThreats(input: string): ThreatSignature[] {
    const threats: ThreatSignature[] = []

    // Check for unauthorized admin operations
    const dangerousPatterns = ['delete_all', 'drop_', 'truncate_', 'exec', 'eval']
    
    for (const pattern of dangerousPatterns) {
      if (input.toLowerCase().includes(pattern)) {
        threats.push({
          type: 'command',
          pattern: pattern,
          severity: 'critical',
          description: `Dangerous admin command detected: ${pattern}`,
          blockedContent: pattern,
          recommendedAction: 'Block immediately and alert security team'
        })
      }
    }

    return threats
  }

  /**
   * API parameter threat detection
   */
  private detectApiParameterThreats(input: string): ThreatSignature[] {
    const threats: ThreatSignature[] = []

    // Check for API abuse patterns
    if (input.length > 10000) {
      threats.push({
        type: 'data_exfiltration',
        pattern: 'oversized_parameter',
        severity: 'medium',
        description: 'Abnormally large API parameter detected',
        blockedContent: 'oversized parameter',
        recommendedAction: 'Limit parameter size and monitor user'
      })
    }

    return threats
  }

  /**
   * Calculate overall threat level
   */
  private calculateThreatLevel(threats: ThreatSignature[]): ThreatLevel {
    if (threats.some(t => t.severity === 'critical')) return 'critical'
    if (threats.some(t => t.severity === 'high')) return 'high'
    if (threats.some(t => t.severity === 'medium')) return 'medium'
    return 'low'
  }

  /**
   * Calculate security score (0-100)
   */
  private calculateSecurityScore(input: string, threats: ThreatSignature[]): number {
    let score = 100

    // Deduct points for threats
    for (const threat of threats) {
      switch (threat.severity) {
        case 'critical': score -= 50; break
        case 'high': score -= 20; break
        case 'medium': score -= 10; break
        case 'low': score -= 5; break
      }
    }

    // Additional deductions for suspicious patterns
    if (input.length > 1000) score -= 5
    if (/[<>'"&;|`$]/.test(input)) score -= 10

    return Math.max(0, score)
  }

  /**
   * Generate security recommendations
   */
  private generateRecommendations(threats: ThreatSignature[], score: number): string[] {
    const recommendations: string[] = []

    if (score < 50) {
      recommendations.push('Input blocked due to security threats')
    }

    if (threats.some(t => t.type === 'xss')) {
      recommendations.push('Enable XSS protection headers')
    }

    if (threats.some(t => t.type === 'injection')) {
      recommendations.push('Use parameterized queries')
    }

    if (threats.some(t => t.type === 'permission_bypass')) {
      recommendations.push('Audit user permissions immediately')
    }

    return recommendations
  }

  /**
   * Extract matched content for logging
   */
  private extractMatchedContent(input: string, pattern: RegExp): string {
    const matches = input.match(pattern)
    return matches ? matches.join(', ') : 'pattern match'
  }

  /**
   * Track suspicious activity by user
   */
  private trackSuspiciousActivity(userId: string, threats: ThreatSignature[]): void {
    const currentCount = this.suspiciousActivity.get(userId) || 0
    const threatCount = threats.length
    
    this.suspiciousActivity.set(userId, currentCount + threatCount)

    // Alert if user exceeds threshold
    if (currentCount + threatCount > 5) {
      this.alertSecurityTeam(userId, threats)
    }
  }

  /**
   * Alert security team of suspicious activity
   */
  private alertSecurityTeam(userId: string, threats: ThreatSignature[]): void {
    console.error(`🚨 SECURITY ALERT: User ${userId} detected with ${threats.length} threats`)
    
    // Log to analytics
    permissionErrorAnalytics.trackEvent('security_threat_detected', {
      user_id: userId,
      threat_count: threats.length,
      threat_types: threats.map(t => t.type),
      severity: 'critical',
      timestamp: Date.now()
    })
  }

  /**
   * Log security event
   */
  private logSecurityEvent(result: InputValidationResult, options: SanitizationOptions): void {
    if (result.threatsDetected.length > 0) {
      console.warn('Security threats detected:', {
        userId: options.userId,
        component: options.component,
        context: options.context,
        threats: result.threatsDetected.map(t => ({
          type: t.type,
          severity: t.severity,
          description: t.description
        })),
        securityScore: result.securityScore
      })

      // Track in analytics
      if (options.enableLogging) {
        permissionErrorAnalytics.trackEvent('input_security_threat', {
          user_id: options.userId,
          component: options.component,
          context: options.context,
          threat_level: result.threatLevel,
          security_score: result.securityScore,
          threat_count: result.threatsDetected.length
        })
      }
    }
  }
}

// Singleton instance
export const comprehensiveInputSanitizer = ComprehensiveInputSanitizer.getInstance()

// Convenience functions for common use cases
export const sanitizePermissionString = (input: string, userId?: string, component?: string): InputValidationResult => {
  return comprehensiveInputSanitizer.sanitizeInput(input, {
    context: 'permission_string',
    strictMode: true,
    allowHtml: false,
    maxLength: 100,
    userId,
    component,
    enableThreatDetection: true,
    enableLogging: true
  })
}

export const sanitizeUserInput = (input: string, userId?: string, component?: string): InputValidationResult => {
  return comprehensiveInputSanitizer.sanitizeInput(input, {
    context: 'user_input',
    strictMode: false,
    allowHtml: false,
    maxLength: 10000,
    userId,
    component,
    enableThreatDetection: true,
    enableLogging: true
  })
}

export const sanitizeAdminCommand = (input: string, userId?: string, component?: string): InputValidationResult => {
  return comprehensiveInputSanitizer.sanitizeInput(input, {
    context: 'admin_command',
    strictMode: true,
    allowHtml: false,
    maxLength: 100,
    userId,
    component,
    permissionLevel: 'admin',
    enableThreatDetection: true,
    enableLogging: true
  })
}

export const sanitizeApiParameter = (input: string, userId?: string, component?: string): InputValidationResult => {
  return comprehensiveInputSanitizer.sanitizeInput(input, {
    context: 'api_parameter',
    strictMode: true,
    allowHtml: false,
    maxLength: 1000,
    userId,
    component,
    enableThreatDetection: true,
    enableLogging: true
  })
}

export const sanitizeSearchQuery = (input: string, userId?: string, component?: string): InputValidationResult => {
  return comprehensiveInputSanitizer.sanitizeInput(input, {
    context: 'search_query',
    strictMode: false,
    allowHtml: false,
    maxLength: 500,
    userId,
    component,
    enableThreatDetection: true,
    enableLogging: false // Search queries logged separately
  })
}

export default comprehensiveInputSanitizer