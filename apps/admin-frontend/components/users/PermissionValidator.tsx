/**
 * Permission Validator Component
 * Advanced validation and safety checks for permission assignments
 */

'use client'

import React, { useState } from 'react'
import { AlertTriangle, Shield, CheckCircle, XCircle, Info, AlertCircle, Settings } from 'lucide-react'
import { Button } from '@epsx/ui'
import { Badge } from '@epsx/ui'
import { Alert, AlertDescription } from '@/components/ui/alert'
import { validatePermissionAssignment } from '@/lib/actions/unified-user-actions'
import { PermissionConflictResolver } from './PermissionConflictResolver'

interface ValidationResult {
  isValid: boolean
  conflicts: Array<{
    type: 'role_conflict' | 'permission_duplicate' | 'hierarchy_violation' | 'quota_exceeded'
    severity: 'error' | 'warning' | 'info'
    message: string
    details?: string
    suggestion?: string
  }>
  warnings: string[]
}

interface PermissionValidatorProps {
  userId: string
  resource: string
  action: string
  onValidationComplete?: (result: ValidationResult) => void
  autoValidate?: boolean
  className?: string
  showAdvancedResolver?: boolean
  proposedChanges?: {
    addPermissions?: string[]
    removePermissions?: string[]
    addRoles?: string[]
    removeRoles?: string[]
    addProfiles?: string[]
    removeProfiles?: string[]
  }
}

const CONFLICT_ICONS = {
  role_conflict: AlertTriangle,
  permission_duplicate: Info,
  hierarchy_violation: XCircle,
  quota_exceeded: AlertCircle,
}

const SEVERITY_COLORS = {
  error: 'text-red-600 bg-red-50 border-red-200',
  warning: 'text-yellow-600 bg-yellow-50 border-yellow-200',
  info: 'text-blue-600 bg-blue-50 border-blue-200',
}

export function PermissionValidator({
  userId,
  resource,
  action,
  onValidationComplete,
  autoValidate = false,
  className = '',
  showAdvancedResolver = false,
  proposedChanges
}: PermissionValidatorProps) {
  const [validationResult, setValidationResult] = useState<ValidationResult | null>(null)
  const [isValidating, setIsValidating] = useState(false)
  const [hasValidated, setHasValidated] = useState(false)
  const [showResolver, setShowResolver] = useState(false)

  const performValidation = async () => {
    if (!userId || !resource || !action) return

    try {
      setIsValidating(true)
      setHasValidated(false)

      const result = await validatePermissionAssignment({
        userId,
        resource,
        action
      })

      if (result.success) {
        // Transform backend response to our validation format
        const validationResult: ValidationResult = {
          isValid: result.data.conflicts.length === 0,
          conflicts: result.data.conflicts.map((conflict: any) => ({
            type: conflict.type || 'permission_duplicate',
            severity: conflict.severity || 'warning',
            message: conflict.message,
            details: conflict.details,
            suggestion: conflict.suggestion
          })),
          warnings: result.data.warnings || []
        }

        setValidationResult(validationResult)
        setHasValidated(true)
        onValidationComplete?.(validationResult)
      }
    } catch (error) {
      console.error('Validation error:', error)
      setValidationResult({
        isValid: false,
        conflicts: [{
          type: 'role_conflict',
          severity: 'error',
          message: 'Failed to validate permission assignment',
          details: 'An unexpected error occurred during validation'
        }],
        warnings: []
      })
      setHasValidated(true)
    } finally {
      setIsValidating(false)
    }
  }

  // Auto-validate when props change
  React.useEffect(() => {
    if (autoValidate && userId && resource && action) {
      performValidation()
    }
  }, [userId, resource, action, autoValidate])

  if (!userId || !resource || !action) {
    return null
  }

  const hasErrors = validationResult?.conflicts.some(c => c.severity === 'error') || false
  const hasWarnings = validationResult?.conflicts.some(c => c.severity === 'warning') || validationResult?.warnings.length > 0 || false

  return (
    <div className={`space-y-3 ${className}`}>
      {/* Validation Trigger */}
      {!autoValidate && (
        <Button
          onClick={performValidation}
          disabled={isValidating}
          variant="outline"
          size="sm"
          className="w-full"
        >
          {isValidating ? (
            <div className="animate-spin rounded-full h-4 w-4 border-b-2 border-primary mr-2"></div>
          ) : (
            <Shield className="h-4 w-4 mr-2" />
          )}
          Validate Permission Assignment
        </Button>
      )}

      {/* Validation Status */}
      {hasValidated && validationResult && (
        <div className="space-y-2">
          <div className="flex items-center gap-2">
            {validationResult.isValid ? (
              <CheckCircle className="h-5 w-5 text-green-600" />
            ) : hasErrors ? (
              <XCircle className="h-5 w-5 text-red-600" />
            ) : (
              <AlertTriangle className="h-5 w-5 text-yellow-600" />
            )}
            
            <span className="text-sm font-medium">
              {validationResult.isValid ? 'Validation Passed' : 
               hasErrors ? 'Validation Failed' : 'Validation Warnings'}
            </span>
            
            <Badge 
              variant={validationResult.isValid ? "default" : hasErrors ? "destructive" : "secondary"}
              size="sm"
            >
              {validationResult.isValid ? 'Safe' : hasErrors ? 'Blocked' : 'Caution'}
            </Badge>
          </div>

          {/* Permission Details */}
          <div className="text-xs text-muted-foreground bg-accent/20 p-2 rounded border font-mono">
            Validating: <span className="font-semibold">{resource}:{action}</span> for user {userId}
          </div>
        </div>
      )}

      {/* Validation Results */}
      {validationResult && (
        <div className="space-y-2">
          {/* Conflicts */}
          {validationResult.conflicts.map((conflict, index) => {
            const IconComponent = CONFLICT_ICONS[conflict.type] || AlertTriangle
            const colorClass = SEVERITY_COLORS[conflict.severity]
            
            return (
              <Alert key={`conflict-${conflict.type}-${index}`} className={`text-sm ${colorClass}`}>
                <IconComponent className="h-4 w-4" />
                <AlertDescription>
                  <div className="space-y-1">
                    <div className="font-medium">{conflict.message}</div>
                    {conflict.details && (
                      <div className="text-xs opacity-75">{conflict.details}</div>
                    )}
                    {conflict.suggestion && (
                      <div className="text-xs bg-white/50 dark:bg-gray-700/50 p-2 rounded border dark:border-gray-600">
                        <strong>Suggestion:</strong> {conflict.suggestion}
                      </div>
                    )}
                  </div>
                </AlertDescription>
              </Alert>
            )
          })}

          {/* General Warnings */}
          {validationResult.warnings.map((warning, index) => (
            <Alert key={`warning-${warning.substring(0, 20)}-${index}`} className="text-sm text-yellow-600 bg-yellow-50 border-yellow-200">
              <Info className="h-4 w-4" />
              <AlertDescription>{warning}</AlertDescription>
            </Alert>
          ))}

          {/* Safety Recommendations */}
          {hasWarnings && (
            <div className="bg-blue-50 border border-blue-200 p-3 rounded text-sm text-blue-800">
              <div className="font-medium mb-2">Safety Recommendations:</div>
              <ul className="text-xs space-y-1 list-disc list-inside">
                <li>Review existing user permissions before granting additional access</li>
                <li>Consider using temporary permissions for short-term access needs</li>
                <li>Document the reason for this permission assignment</li>
                <li>Set up periodic reviews of user permissions</li>
              </ul>
            </div>
          )}

          {/* Validation Summary */}
          <div className="flex items-center justify-between text-xs text-muted-foreground pt-2 border-t">
            <span>
              {validationResult.conflicts.length} conflict(s), {validationResult.warnings.length} warning(s)
            </span>
            <div className="flex items-center gap-2">
              {(showAdvancedResolver || hasWarnings || hasErrors) && (
                <Button
                  onClick={() => setShowResolver(!showResolver)}
                  variant="ghost"
                  size="sm"
                >
                  <Settings className="h-4 w-4 mr-1" />
                  {showResolver ? 'Hide' : 'Advanced'} Resolver
                </Button>
              )}
              <Button
                onClick={performValidation}
                disabled={isValidating}
                variant="ghost"
                size="sm"
              >
                Re-validate
              </Button>
            </div>
          </div>
        </div>
      )}

      {/* Advanced Conflict Resolver */}
      {showResolver && (
        <div className="mt-6 pt-6 border-t">
          <PermissionConflictResolver
            userId={userId}
            proposedChanges={proposedChanges}
            onValidationComplete={(result) => {
              // Update our validation result with advanced resolver results
              if (result.conflicts.length === 0 && result.securityRisks.length === 0) {
                setValidationResult(prev => prev ? { ...prev, isValid: true } : null);
              }
            }}
            autoValidate={false}
          />
        </div>
      )}
    </div>
  )
}

/**
 * Batch Permission Validator
 * Validates multiple permission assignments at once
 */
export function BatchPermissionValidator({
  userId,
  permissions,
  onValidationComplete,
  className = ''
}: {
  userId: string
  permissions: Array<{ resource: string; action: string }>
  onValidationComplete?: (results: Array<{ permission: { resource: string; action: string }, result: ValidationResult }>) => void
  className?: string
}) {
  const [validationResults, setValidationResults] = useState<Array<{ permission: { resource: string; action: string }, result: ValidationResult }>>([])
  const [isValidating, setIsValidating] = useState(false)

  const validateAll = async () => {
    if (permissions.length === 0) return

    try {
      setIsValidating(true)
      const results = []

      for (const permission of permissions) {
        const result = await validatePermissionAssignment({
          userId,
          resource: permission.resource,
          action: permission.action
        })

        if (result.success) {
          results.push({
            permission,
            result: {
              isValid: result.data.conflicts.length === 0,
              conflicts: result.data.conflicts.map((conflict: any) => ({
                type: conflict.type || 'permission_duplicate',
                severity: conflict.severity || 'warning',
                message: conflict.message,
                details: conflict.details,
                suggestion: conflict.suggestion
              })),
              warnings: result.data.warnings || []
            }
          })
        }
      }

      setValidationResults(results)
      onValidationComplete?.(results)
    } catch (error) {
      console.error('Batch validation error:', error)
    } finally {
      setIsValidating(false)
    }
  }

  const totalConflicts = validationResults.reduce((sum, r) => sum + r.result.conflicts.length, 0)
  const totalWarnings = validationResults.reduce((sum, r) => sum + r.result.warnings.length, 0)
  const hasErrors = validationResults.some(r => r.result.conflicts.some(c => c.severity === 'error'))

  return (
    <div className={`space-y-4 ${className}`}>
      <div className="flex items-center justify-between">
        <h4 className="font-medium">Batch Permission Validation</h4>
        <Button
          onClick={validateAll}
          disabled={isValidating || permissions.length === 0}
          variant="outline"
          size="sm"
        >
          {isValidating ? (
            <div className="animate-spin rounded-full h-4 w-4 border-b-2 border-primary mr-2"></div>
          ) : (
            <Shield className="h-4 w-4 mr-2" />
          )}
          Validate {permissions.length} Permission(s)
        </Button>
      </div>

      {validationResults.length > 0 && (
        <div className="space-y-3">
          {/* Summary */}
          <div className="grid grid-cols-3 gap-2 text-center">
            <div className="p-2 border rounded bg-card">
              <div className="text-lg font-bold text-green-600">
                {validationResults.filter(r => r.result.isValid).length}
              </div>
              <div className="text-xs text-muted-foreground">Valid</div>
            </div>
            <div className="p-2 border rounded bg-card">
              <div className="text-lg font-bold text-yellow-600">{totalConflicts}</div>
              <div className="text-xs text-muted-foreground">Conflicts</div>
            </div>
            <div className="p-2 border rounded bg-card">
              <div className="text-lg font-bold text-blue-600">{totalWarnings}</div>
              <div className="text-xs text-muted-foreground">Warnings</div>
            </div>
          </div>

          {/* Individual Results */}
          <div className="space-y-2 max-h-64 overflow-y-auto">
            {validationResults.map((result, index) => (
              <div key={`${result.permission.resource}-${result.permission.action}-${index}`} className="p-3 border rounded bg-card">
                <div className="flex items-center justify-between mb-2">
                  <span className="font-mono text-sm">
                    {result.permission.resource}:{result.permission.action}
                  </span>
                  <Badge 
                    variant={result.result.isValid ? "default" : hasErrors ? "destructive" : "secondary"}
                    size="sm"
                  >
                    {result.result.isValid ? 'Valid' : 'Issues'}
                  </Badge>
                </div>
                
                {result.result.conflicts.length > 0 && (
                  <div className="space-y-1">
                    {result.result.conflicts.map((conflict, cIndex) => (
                      <div key={cIndex} className={`text-xs p-2 rounded ${SEVERITY_COLORS[conflict.severity]}`}>
                        {conflict.message}
                      </div>
                    ))}
                  </div>
                )}
              </div>
            ))}
          </div>
        </div>
      )}
    </div>
  )
}