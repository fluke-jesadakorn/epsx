/**
 * Secure Form Validation System
 * 
 * SECURITY CRITICAL: This module provides tamper-proof form validation
 * with comprehensive input sanitization, integrity checking, and threat prevention.
 * 
 * Features:
 * - Multi-layer validation with sanitization integration
 * - Anti-tampering detection and prevention
 * - Context-aware validation rules
 * - Real-time security monitoring
 * - Comprehensive audit logging
 */

import { z } from 'zod'
import { comprehensiveInputSanitizer, type InputValidationResult, type SanitizationContext } from './input-sanitization'
import { permissionErrorAnalytics } from '@/lib/analytics/permission-error-analytics'

// Form validation result with security metadata
export interface SecureValidationResult<T = any> {
  success: boolean
  data?: T
  errors: ValidationError[]
  securityEvents: SecurityEvent[]
  integrityCheck: IntegrityCheckResult
  sanitizedData: Record<string, any>
  threatLevel: 'low' | 'medium' | 'high' | 'critical'
  recommendations: string[]
}

// Validation error with security context
export interface ValidationError {
  field: string
  message: string
  code: string
  severity: 'info' | 'warning' | 'error' | 'critical'
  securityImplications?: string[]
}

// Security event tracking
export interface SecurityEvent {
  type: 'sanitization_blocked' | 'validation_failed' | 'tampering_detected' | 'suspicious_pattern'
  field: string
  description: string
  originalValue: string
  sanitizedValue: string
  threatSignatures: string[]
  timestamp: number
  action: 'blocked' | 'sanitized' | 'flagged' | 'allowed'
}

// Form integrity check result
export interface IntegrityCheckResult {
  passed: boolean
  checksumValid: boolean
  fieldsModified: string[]
  suspiciousActivity: boolean
  tamperingAttempts: number
}

// Secure field configuration
export interface SecureFieldConfig {
  context: SanitizationContext
  required: boolean
  minLength?: number
  maxLength?: number
  pattern?: RegExp
  allowedValues?: string[]
  sanitizationLevel: 'strict' | 'moderate' | 'lenient'
  permissionRequired?: string
  antiTampering: boolean
}

// Form schema with security metadata
export interface SecureFormSchema {
  fields: Record<string, SecureFieldConfig>
  integrityToken?: string
  userId?: string
  component?: string
  securityLevel: 'standard' | 'elevated' | 'critical'
  enableAuditLogging: boolean
}

/**
 * Secure Form Validator Class
 * Provides enterprise-grade form validation with comprehensive security
 */
export class SecureFormValidator {
  private static instance: SecureFormValidator
  private formIntegrityTokens: Map<string, string>
  private fieldModificationHistory: Map<string, number>

  constructor() {
    this.formIntegrityTokens = new Map()
    this.fieldModificationHistory = new Map()
  }

  static getInstance(): SecureFormValidator {
    if (!SecureFormValidator.instance) {
      SecureFormValidator.instance = new SecureFormValidator()
    }
    return SecureFormValidator.instance
  }

  /**
   * Generate form integrity token for anti-tampering
   */
  public generateIntegrityToken(formId: string, schema: SecureFormSchema): string {
    const tokenData = {
      formId,
      fields: Object.keys(schema.fields),
      timestamp: Date.now(),
      userId: schema.userId,
      securityLevel: schema.securityLevel
    }

    const token = btoa(JSON.stringify(tokenData))
    this.formIntegrityTokens.set(formId, token)
    return token
  }

  /**
   * Validate form integrity and detect tampering
   */
  private validateFormIntegrity(
    formId: string, 
    submittedToken: string, 
    submittedFields: string[]
  ): IntegrityCheckResult {
    const storedToken = this.formIntegrityTokens.get(formId)
    
    if (!storedToken || !submittedToken) {
      return {
        passed: false,
        checksumValid: false,
        fieldsModified: submittedFields,
        suspiciousActivity: true,
        tamperingAttempts: 1
      }
    }

    try {
      const storedData = JSON.parse(atob(storedToken))
      const submittedData = JSON.parse(atob(submittedToken))

      const checksumValid = storedToken === submittedToken
      const fieldsModified = submittedFields.filter(field => 
        !storedData.fields.includes(field)
      )

      const suspiciousActivity = fieldsModified.length > 0 || !checksumValid
      const tamperingAttempts = suspiciousActivity ? 1 : 0

      return {
        passed: checksumValid && fieldsModified.length === 0,
        checksumValid,
        fieldsModified,
        suspiciousActivity,
        tamperingAttempts
      }

    } catch (error) {
      return {
        passed: false,
        checksumValid: false,
        fieldsModified: submittedFields,
        suspiciousActivity: true,
        tamperingAttempts: 1
      }
    }
  }

  /**
   * Main secure validation entry point
   */
  public async validateForm<T>(
    formData: Record<string, any>,
    schema: SecureFormSchema,
    formId?: string
  ): Promise<SecureValidationResult<T>> {
    const startTime = Date.now()
    const errors: ValidationError[] = []
    const securityEvents: SecurityEvent[] = []
    const sanitizedData: Record<string, any> = {}
    let threatLevel: 'low' | 'medium' | 'high' | 'critical' = 'low'

    try {
      // Step 1: Form integrity check (anti-tampering)
      let integrityCheck: IntegrityCheckResult = {
        passed: true,
        checksumValid: true,
        fieldsModified: [],
        suspiciousActivity: false,
        tamperingAttempts: 0
      }

      if (formId && schema.integrityToken) {
        integrityCheck = this.validateFormIntegrity(
          formId,
          schema.integrityToken,
          Object.keys(formData)
        )

        if (!integrityCheck.passed) {
          securityEvents.push({
            type: 'tampering_detected',
            field: 'form_integrity',
            description: 'Form tampering detected - fields modified or token invalid',
            originalValue: 'form_token',
            sanitizedValue: 'blocked',
            threatSignatures: ['form_tampering', 'token_manipulation'],
            timestamp: Date.now(),
            action: 'blocked'
          })
          threatLevel = 'critical'
        }
      }

      // Step 2: Field-by-field validation and sanitization
      for (const [fieldName, fieldConfig] of Object.entries(schema.fields)) {
        const fieldValue = formData[fieldName]
        
        try {
          const fieldResult = await this.validateField(
            fieldName,
            fieldValue,
            fieldConfig,
            schema
          )

          if (!fieldResult.success) {
            errors.push(...fieldResult.errors)
            securityEvents.push(...fieldResult.securityEvents)
            
            // Update threat level based on field validation
            if (fieldResult.threatLevel === 'critical' || threatLevel === 'low') {
              threatLevel = fieldResult.threatLevel
            }
          } else {
            sanitizedData[fieldName] = fieldResult.sanitizedValue
          }

        } catch (error) {
          errors.push({
            field: fieldName,
            message: `Field validation failed: ${error.message}`,
            code: 'VALIDATION_ERROR',
            severity: 'error',
            securityImplications: ['Potential security bypass attempt']
          })
          
          securityEvents.push({
            type: 'validation_failed',
            field: fieldName,
            description: `Field validation threw exception: ${error.message}`,
            originalValue: String(fieldValue),
            sanitizedValue: '',
            threatSignatures: ['validation_exception'],
            timestamp: Date.now(),
            action: 'blocked'
          })
          
          threatLevel = 'high'
        }
      }

      // Step 3: Cross-field validation
      const crossFieldResults = await this.validateCrossFieldRules(sanitizedData, schema)
      errors.push(...crossFieldResults.errors)
      securityEvents.push(...crossFieldResults.securityEvents)

      // Step 4: Generate recommendations
      const recommendations = this.generateSecurityRecommendations(
        errors,
        securityEvents,
        integrityCheck,
        threatLevel
      )

      // Step 5: Security logging
      if (schema.enableAuditLogging) {
        await this.logSecurityAuditEvent({
          userId: schema.userId,
          component: schema.component,
          formId,
          threatLevel,
          securityEvents: securityEvents.length,
          validationErrors: errors.length,
          integrityPassed: integrityCheck.passed,
          processingTime: Date.now() - startTime
        })
      }

      const success = errors.length === 0 && 
                     integrityCheck.passed && 
                     threatLevel !== 'critical'

      return {
        success,
        data: success ? sanitizedData as T : undefined,
        errors,
        securityEvents,
        integrityCheck,
        sanitizedData,
        threatLevel,
        recommendations
      }

    } catch (error) {
      console.error('Secure form validation failed:', error)
      
      return {
        success: false,
        errors: [{
          field: 'form',
          message: 'Form validation failed due to security error',
          code: 'SECURITY_ERROR',
          severity: 'critical',
          securityImplications: ['Potential security exploit attempt']
        }],
        securityEvents: [{
          type: 'validation_failed',
          field: 'form',
          description: `Form validation exception: ${error.message}`,
          originalValue: 'form_data',
          sanitizedValue: 'blocked',
          threatSignatures: ['validation_system_error'],
          timestamp: Date.now(),
          action: 'blocked'
        }],
        integrityCheck: {
          passed: false,
          checksumValid: false,
          fieldsModified: Object.keys(formData),
          suspiciousActivity: true,
          tamperingAttempts: 1
        },
        sanitizedData: {},
        threatLevel: 'critical',
        recommendations: ['Block form submission', 'Investigate security error', 'Review validation logs']
      }
    }
  }

  /**
   * Validate individual field with security checks
   */
  private async validateField(
    fieldName: string,
    fieldValue: any,
    config: SecureFieldConfig,
    schema: SecureFormSchema
  ): Promise<{
    success: boolean
    sanitizedValue: any
    errors: ValidationError[]
    securityEvents: SecurityEvent[]
    threatLevel: 'low' | 'medium' | 'high' | 'critical'
  }> {
    const errors: ValidationError[] = []
    const securityEvents: SecurityEvent[] = []
    let threatLevel: 'low' | 'medium' | 'high' | 'critical' = 'low'

    // Convert to string for sanitization
    const stringValue = String(fieldValue || '')

    // Step 1: Required field validation
    if (config.required && (!fieldValue || stringValue.trim() === '')) {
      errors.push({
        field: fieldName,
        message: 'This field is required',
        code: 'REQUIRED_FIELD',
        severity: 'error'
      })
    }

    // Step 2: Length validation
    if (config.minLength && stringValue.length < config.minLength) {
      errors.push({
        field: fieldName,
        message: `Minimum length is ${config.minLength} characters`,
        code: 'MIN_LENGTH',
        severity: 'error'
      })
    }

    if (config.maxLength && stringValue.length > config.maxLength) {
      errors.push({
        field: fieldName,
        message: `Maximum length is ${config.maxLength} characters`,
        code: 'MAX_LENGTH',
        severity: 'error',
        securityImplications: ['Potential buffer overflow attempt']
      })
      threatLevel = 'medium'
    }

    // Step 3: Pattern validation
    if (config.pattern && !config.pattern.test(stringValue)) {
      errors.push({
        field: fieldName,
        message: 'Invalid format',
        code: 'INVALID_FORMAT',
        severity: 'error'
      })
    }

    // Step 4: Allowed values validation
    if (config.allowedValues && !config.allowedValues.includes(stringValue)) {
      errors.push({
        field: fieldName,
        message: 'Invalid value provided',
        code: 'INVALID_VALUE',
        severity: 'error',
        securityImplications: ['Potential injection attempt']
      })
      threatLevel = 'medium'
    }

    // Step 5: Comprehensive sanitization
    const sanitizationOptions = {
      context: config.context,
      strictMode: config.sanitizationLevel === 'strict',
      allowHtml: config.sanitizationLevel === 'lenient',
      maxLength: config.maxLength || 10000,
      userId: schema.userId,
      component: schema.component,
      enableThreatDetection: true,
      enableLogging: schema.enableAuditLogging
    }

    const sanitizationResult = comprehensiveInputSanitizer.sanitizeInput(
      stringValue,
      sanitizationOptions
    )

    // Handle sanitization results
    if (!sanitizationResult.isValid) {
      errors.push({
        field: fieldName,
        message: 'Input contains security threats and was blocked',
        code: 'SECURITY_THREAT',
        severity: 'critical',
        securityImplications: sanitizationResult.threatsDetected.map(t => t.description)
      })

      securityEvents.push({
        type: 'sanitization_blocked',
        field: fieldName,
        description: `Input blocked due to ${sanitizationResult.threatsDetected.length} security threats`,
        originalValue: stringValue,
        sanitizedValue: '',
        threatSignatures: sanitizationResult.threatsDetected.map(t => t.type),
        timestamp: Date.now(),
        action: 'blocked'
      })

      threatLevel = sanitizationResult.threatLevel
    }

    // Step 6: Anti-tampering check
    if (config.antiTampering) {
      const tamperingResult = this.detectFieldTampering(fieldName, stringValue, schema.userId)
      if (tamperingResult.detected) {
        securityEvents.push({
          type: 'tampering_detected',
          field: fieldName,
          description: `Field tampering detected: ${tamperingResult.reason}`,
          originalValue: stringValue,
          sanitizedValue: sanitizationResult.sanitizedInput,
          threatSignatures: ['field_tampering'],
          timestamp: Date.now(),
          action: 'flagged'
        })
        threatLevel = 'high'
      }
    }

    const success = errors.length === 0 && sanitizationResult.isValid
    
    return {
      success,
      sanitizedValue: success ? sanitizationResult.sanitizedInput : '',
      errors,
      securityEvents,
      threatLevel
    }
  }

  /**
   * Detect field tampering attempts
   */
  private detectFieldTampering(
    fieldName: string, 
    value: string, 
    userId?: string
  ): { detected: boolean; reason: string } {
    const fieldKey = `${userId}_${fieldName}`
    const currentCount = this.fieldModificationHistory.get(fieldKey) || 0
    
    // Track field modifications
    this.fieldModificationHistory.set(fieldKey, currentCount + 1)

    // Detect suspicious patterns
    if (currentCount > 10) {
      return { detected: true, reason: 'Excessive field modifications' }
    }

    if (value.length > 10000) {
      return { detected: true, reason: 'Abnormally large field value' }
    }

    if (/(\$\{|`|eval\(|Function\()/i.test(value)) {
      return { detected: true, reason: 'JavaScript injection pattern detected' }
    }

    return { detected: false, reason: '' }
  }

  /**
   * Cross-field validation rules
   */
  private async validateCrossFieldRules(
    data: Record<string, any>,
    schema: SecureFormSchema
  ): Promise<{
    errors: ValidationError[]
    securityEvents: SecurityEvent[]
  }> {
    const errors: ValidationError[] = []
    const securityEvents: SecurityEvent[] = []

    // Password confirmation validation
    if (data.password && data.confirmPassword && data.password !== data.confirmPassword) {
      errors.push({
        field: 'confirmPassword',
        message: 'Passwords do not match',
        code: 'PASSWORD_MISMATCH',
        severity: 'error'
      })
    }

    // Email format validation for user registration
    if (data.email && !/^[^\s@]+@[^\s@]+\.[^\s@]+$/.test(data.email)) {
      errors.push({
        field: 'email',
        message: 'Invalid email format',
        code: 'INVALID_EMAIL',
        severity: 'error'
      })
    }

    // Permission string validation
    if (data.permission && typeof data.permission === 'string') {
      const parts = data.permission.split(':')
      if (parts.length < 3) {
        errors.push({
          field: 'permission',
          message: 'Permission must have format platform:resource:action',
          code: 'INVALID_PERMISSION_FORMAT',
          severity: 'error',
          securityImplications: ['Malformed permission could bypass validation']
        })
      }
    }

    // Detect privilege escalation attempts
    if (data.role && ['super_admin', 'system_admin'].includes(data.role)) {
      securityEvents.push({
        type: 'suspicious_pattern',
        field: 'role',
        description: 'Attempt to assign high-privilege role detected',
        originalValue: data.role,
        sanitizedValue: data.role,
        threatSignatures: ['privilege_escalation'],
        timestamp: Date.now(),
        action: 'flagged'
      })
    }

    return { errors, securityEvents }
  }

  /**
   * Generate security recommendations
   */
  private generateSecurityRecommendations(
    errors: ValidationError[],
    securityEvents: SecurityEvent[],
    integrityCheck: IntegrityCheckResult,
    threatLevel: string
  ): string[] {
    const recommendations: string[] = []

    if (threatLevel === 'critical') {
      recommendations.push('Block form submission immediately')
      recommendations.push('Investigate user account for compromise')
      recommendations.push('Review security logs for related incidents')
    }

    if (!integrityCheck.passed) {
      recommendations.push('Regenerate form integrity token')
      recommendations.push('Implement additional anti-tampering measures')
    }

    if (securityEvents.some(e => e.type === 'sanitization_blocked')) {
      recommendations.push('Enable enhanced input monitoring')
      recommendations.push('Consider additional user verification')
    }

    if (errors.some(e => e.severity === 'critical')) {
      recommendations.push('Implement stricter validation rules')
      recommendations.push('Enable real-time threat detection')
    }

    if (securityEvents.length > 5) {
      recommendations.push('Flag user for security review')
      recommendations.push('Implement rate limiting')
    }

    return recommendations
  }

  /**
   * Log security audit event
   */
  private async logSecurityAuditEvent(auditData: {
    userId?: string
    component?: string
    formId?: string
    threatLevel: string
    securityEvents: number
    validationErrors: number
    integrityPassed: boolean
    processingTime: number
  }): Promise<void> {
    try {
      permissionErrorAnalytics.trackEvent('secure_form_validation_audit', {
        user_id: auditData.userId,
        component: auditData.component,
        form_id: auditData.formId,
        threat_level: auditData.threatLevel,
        security_events: auditData.securityEvents,
        validation_errors: auditData.validationErrors,
        integrity_passed: auditData.integrityPassed,
        processing_time_ms: auditData.processingTime,
        timestamp: Date.now()
      })
    } catch (error) {
      console.error('Failed to log security audit event:', error)
    }
  }
}

// Singleton instance
export const secureFormValidator = SecureFormValidator.getInstance()

// Pre-configured validation schemas for common forms
export const CommonSecureSchemas = {
  // User registration form
  userRegistration: {
    fields: {
      email: {
        context: 'user_input' as SanitizationContext,
        required: true,
        maxLength: 255,
        pattern: /^[^\s@]+@[^\s@]+\.[^\s@]+$/,
        sanitizationLevel: 'strict' as const,
        antiTampering: true
      },
      password: {
        context: 'user_input' as SanitizationContext,
        required: true,
        minLength: 8,
        maxLength: 128,
        sanitizationLevel: 'strict' as const,
        antiTampering: true
      },
      confirmPassword: {
        context: 'user_input' as SanitizationContext,
        required: true,
        sanitizationLevel: 'strict' as const,
        antiTampering: true
      }
    },
    securityLevel: 'elevated' as const,
    enableAuditLogging: true
  },

  // Permission assignment form
  permissionAssignment: {
    fields: {
      permission: {
        context: 'permission_string' as SanitizationContext,
        required: true,
        maxLength: 100,
        pattern: /^[a-zA-Z0-9:_]+(\:[0-9]{10,13})?$/,
        sanitizationLevel: 'strict' as const,
        antiTampering: true
      },
      userId: {
        context: 'user_input' as SanitizationContext,
        required: true,
        maxLength: 50,
        sanitizationLevel: 'strict' as const,
        antiTampering: true
      }
    },
    securityLevel: 'critical' as const,
    enableAuditLogging: true
  },

  // Plan creation form
  planCreation: {
    fields: {
      name: {
        context: 'user_input' as SanitizationContext,
        required: true,
        maxLength: 100,
        sanitizationLevel: 'moderate' as const,
        antiTampering: true
      },
      description: {
        context: 'user_input' as SanitizationContext,
        required: false,
        maxLength: 1000,
        sanitizationLevel: 'moderate' as const,
        antiTampering: false
      },
      price: {
        context: 'api_parameter' as SanitizationContext,
        required: true,
        pattern: /^\d+(\.\d{2})?$/,
        sanitizationLevel: 'strict' as const,
        antiTampering: true
      }
    },
    securityLevel: 'elevated' as const,
    enableAuditLogging: true
  },

  // Search form
  search: {
    fields: {
      query: {
        context: 'search_query' as SanitizationContext,
        required: true,
        maxLength: 500,
        sanitizationLevel: 'moderate' as const,
        antiTampering: false
      }
    },
    securityLevel: 'standard' as const,
    enableAuditLogging: false
  }
}

// Convenience validation functions
export const validateUserRegistrationForm = async (
  formData: { email: string; password: string; confirmPassword: string },
  userId?: string,
  component?: string
): Promise<SecureValidationResult<{ email: string; password: string }>> => {
  return secureFormValidator.validateForm(formData, {
    ...CommonSecureSchemas.userRegistration,
    userId,
    component
  })
}

export const validatePermissionAssignmentForm = async (
  formData: { permission: string; userId: string },
  adminUserId?: string,
  component?: string
): Promise<SecureValidationResult<{ permission: string; userId: string }>> => {
  return secureFormValidator.validateForm(formData, {
    ...CommonSecureSchemas.permissionAssignment,
    userId: adminUserId,
    component
  })
}

export const validatePlanCreationForm = async (
  formData: { name: string; description?: string; price: string },
  userId?: string,
  component?: string
): Promise<SecureValidationResult<{ name: string; description?: string; price: number }>> => {
  return secureFormValidator.validateForm(formData, {
    ...CommonSecureSchemas.planCreation,
    userId,
    component
  })
}

export default secureFormValidator