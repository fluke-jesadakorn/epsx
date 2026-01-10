/**
 * Secure Permission Context Guards
 * Runtime security validation for permission contexts and state management
 */

import React, { createContext, useContext, useEffect, useState } from 'react';

import { logInvalidPermission, logPermissionDenied } from '@/lib/analytics/permission-error-analytics';

export interface SecurityContext {
  userId?: string;
  sessionId?: string;
  permissions: string[];
  lastValidated: string;
  securityScore: number;
  threats: SecurityThreat[];
}

export interface SecurityThreat {
  id: string;
  type: 'permission_escalation' | 'session_hijack' | 'invalid_access' | 'rate_limit_exceeded';
  severity: 'low' | 'medium' | 'high' | 'critical';
  description: string;
  timestamp: string;
  metadata?: Record<string, any>;
}

const SecurityContextState = createContext<SecurityContext | null>(null);

/**
 *
 * @param root0
 * @param root0.children
 * @param root0.initialContext
 */
export function SecurityContextProvider({
  children,
  initialContext
}: {
  children: React.ReactNode;
  initialContext?: Partial<SecurityContext>;
}): React.ReactElement {
  const [context] = useState<SecurityContext>({
    permissions: [],
    lastValidated: new Date().toISOString(),
    securityScore: 100,
    threats: [],
    ...initialContext
  });

  // Validate security context periodically
  useEffect(() => {
    const interval = setInterval(() => {
      validateSecurityContext(context);
    }, 30000); // Every 30 seconds

    return () => clearInterval(interval);
  }, [context]);

  return (
    <SecurityContextState.Provider value={context}>
      {children}
    </SecurityContextState.Provider>
  );
}

/**
 *
 */
export function useSecurityContext(): SecurityContext {
  const context = useContext(SecurityContextState);
  if (!context) {
    throw new Error('useSecurityContext must be used within SecurityContextProvider');
  }
  return context;
}

/**
 *
 * @param requiredPermission
 * @param userPermissions
 * @param userId
 */
export function validatePermissionRequest(
  requiredPermission: string,
  userPermissions: string[],
  userId?: string
): boolean {
  // Check for direct permission match
  if (userPermissions.includes(requiredPermission)) {
    return true;
  }

  // Check for wildcard permissions
  const permissionParts = requiredPermission.split(':');
  const wildcardPermissions = [
    `${permissionParts[0]}:*:*`,
    `${permissionParts[0]}:${permissionParts[1]}:*`,
    '*:*:*'
  ];

  const hasWildcardPermission = wildcardPermissions.some(wildcard =>
    userPermissions.includes(wildcard)
  );

  if (hasWildcardPermission) {
    return true;
  }

  // Log permission denial
  logPermissionDenied(requiredPermission, userPermissions, userId);
  return false;
}

/**
 * Detect permission escalation threats 
 * @param permissions 
 */
function detectPermissionEscalation(permissions: string[]): { threats: SecurityThreat[], scoreReduction: number } {
  const threats: SecurityThreat[] = [];
  let scoreReduction = 0;

  const adminPermissions = permissions.filter(p => p.startsWith('admin:'));
  if (adminPermissions.length > 0) {
    // Validate admin permissions are legitimate
    const suspiciousAdminPerms = adminPermissions.filter(p =>
      !['admin:*:*', 'admin:users:view', 'admin:analytics:view'].includes(p)
    );

    if (suspiciousAdminPerms.length > 0) {
      threats.push({
        id: crypto.randomUUID(),
        type: 'permission_escalation',
        severity: 'high',
        description: `Suspicious admin permissions detected: ${suspiciousAdminPerms.join(', ')}`,
        timestamp: new Date().toISOString(),
        metadata: { permissions: suspiciousAdminPerms }
      });
      scoreReduction = 20;
    }
  }

  return { threats, scoreReduction };
}

/**
 * Validate session age 
 * @param lastValidatedStr 
 */
function validateSessionAge(lastValidatedStr: string): { threats: SecurityThreat[], scoreReduction: number } {
  const threats: SecurityThreat[] = [];
  let scoreReduction = 0;

  const lastValidated = new Date(lastValidatedStr);
  const sessionAge = Date.now() - lastValidated.getTime();
  const maxSessionAge = 24 * 60 * 60 * 1000; // 24 hours

  if (sessionAge > maxSessionAge) {
    threats.push({
      id: crypto.randomUUID(),
      type: 'session_hijack',
      severity: 'medium',
      description: 'Session has exceeded maximum age limit',
      timestamp: new Date().toISOString(),
      metadata: { sessionAge, maxSessionAge }
    });
    scoreReduction = 15;
  }

  return { threats, scoreReduction };
}

/**
 * Detect suspicious permission combinations 
 * @param permissions 
 */
function detectSuspiciousCombinations(permissions: string[]): { threats: SecurityThreat[], scoreReduction: number } {
  const threats: SecurityThreat[] = [];
  let scoreReduction = 0;

  const hasWeb3Perms = permissions.some(p => p.includes('web3'));
  const hasAdminPerms = permissions.some(p => p.startsWith('admin:'));
  const hasUserMgmtPerms = permissions.some(p => p.includes('users:manage'));

  if (hasWeb3Perms && hasAdminPerms && hasUserMgmtPerms) {
    threats.push({
      id: crypto.randomUUID(),
      type: 'invalid_access',
      severity: 'medium',
      description: 'User has unusually broad permission set',
      timestamp: new Date().toISOString(),
      metadata: {
        permissionTypes: ['web3', 'admin', 'user_management'],
        totalPermissions: permissions.length
      }
    });
    scoreReduction = 10;
  }

  return { threats, scoreReduction };
}

/**
 *
 * @param context
 */
export function validateSecurityContext(context: SecurityContext): SecurityContext {
  let threats: SecurityThreat[] = [...context.threats];
  let securityScore = 100;

  // 1. Check for permission escalation
  const escalation = detectPermissionEscalation(context.permissions);
  threats = [...threats, ...escalation.threats];
  securityScore -= escalation.scoreReduction;

  // 2. Check session age
  const session = validateSessionAge(context.lastValidated);
  threats = [...threats, ...session.threats];
  securityScore -= session.scoreReduction;

  // 3. Check for suspicious permission combinations
  const combinations = detectSuspiciousCombinations(context.permissions);
  threats = [...threats, ...combinations.threats];
  securityScore -= combinations.scoreReduction;

  // Remove old threats (older than 1 hour)
  const oneHourAgo = new Date(Date.now() - 60 * 60 * 1000);
  const recentThreats = threats.filter(threat =>
    new Date(threat.timestamp) > oneHourAgo
  );

  return {
    ...context,
    threats: recentThreats,
    securityScore: Math.max(0, securityScore),
    lastValidated: new Date().toISOString()
  };
}

/**
 *
 * @param permission
 */
export function checkPermissionSafety(permission: string): {
  isSafe: boolean;
  risks: string[];
  recommendations: string[];
} {
  const risks: string[] = [];
  const recommendations: string[] = [];

  // Check for dangerous wildcards
  if (permission === '*:*:*') {
    risks.push('Global wildcard permission grants unlimited access');
    recommendations.push('Use specific permissions instead of global wildcards');
  }

  // Check for admin permissions
  if (permission.startsWith('admin:')) {
    if (permission === 'admin:*:*') {
      risks.push('Full admin access can modify system settings');
      recommendations.push('Limit admin permissions to specific resources');
    }

    if (permission.includes('users:delete')) {
      risks.push('User deletion permission cannot be undone');
      recommendations.push('Use user deactivation instead of deletion');
    }
  }

  // Check for dangerous actions
  const dangerousActions = ['delete', 'destroy', 'purge', 'admin'];
  const action = permission.split(':')[2];

  if (action && dangerousActions.includes(action)) {
    risks.push(`Action "${action}" can cause irreversible changes`);
    recommendations.push('Ensure proper audit logging is enabled');
  }

  // Check for embedded timestamps
  const parts = permission.split(':');
  if (parts.length === 4 && !isNaN(parseInt(parts[3] || '0'))) {
    const expiryTimestamp = parseInt(parts[3] || '0');
    const expiryDate = new Date(expiryTimestamp * 1000);
    const now = new Date();

    if (expiryDate < now) {
      risks.push('Permission has already expired');
      recommendations.push('Remove expired permissions from user accounts');
    } else if (expiryDate.getTime() - now.getTime() > 365 * 24 * 60 * 60 * 1000) {
      risks.push('Permission expires more than 1 year in the future');
      recommendations.push('Consider shorter expiry times for security');
    }
  }

  return {
    isSafe: risks.length === 0,
    risks,
    recommendations
  };
}

/**
 *
 * @param input
 */
export function sanitizePermissionInput(input: string): string {
  // Remove dangerous characters and normalize
  return input
    .replace(/[^\w:.-]/g, '')
    .toLowerCase()
    .trim();
}

/**
 *
 * @param permission
 */
export function validatePermissionFormat(permission: string): {
  isValid: boolean;
  errors: string[];
} {
  const errors: string[] = [];

  if (!permission || permission.trim().length === 0) {
    errors.push('Permission cannot be empty');
    return { isValid: false, errors };
  }

  const parts = permission.split(':');

  if (parts.length < 3 || parts.length > 4) {
    errors.push('Permission must have format "platform:resource:action" or "platform:resource:action:timestamp"');
  }

  // Validate platform
  const validPlatforms = ['admin', 'epsx', 'epsx-pay', 'epsx-token', '*'];
  if (parts[0] && !validPlatforms.includes(parts[0])) {
    errors.push(`Invalid platform "${parts[0]}". Must be one of: ${validPlatforms.join(', ')}`);
  }

  // Validate resource
  if (parts[1] && parts[1].length === 0) {
    errors.push('Resource cannot be empty');
  }

  // Validate action
  if (parts[2] && parts[2].length === 0) {
    errors.push('Action cannot be empty');
  }

  // Validate timestamp if present
  if (parts[3]) {
    const timestamp = parseInt(parts[3] || '0');
    if (isNaN(timestamp) || timestamp < 0) {
      errors.push('Timestamp must be a valid Unix timestamp');
    }
  }

  return {
    isValid: errors.length === 0,
    errors
  };
}

// Rate limiting for permission checks
const permissionCheckLimiter = new Map<string, { count: number; resetTime: number }>();

/**
 *
 * @param userId
 * @param limit
 */
export function rateLimitPermissionCheck(userId: string, limit = 100): boolean {
  const now = Date.now();
  const windowSize = 60 * 1000; // 1 minute window

  const userLimiter = permissionCheckLimiter.get(userId);

  if (!userLimiter || now > userLimiter.resetTime) {
    permissionCheckLimiter.set(userId, { count: 1, resetTime: now + windowSize });
    return true;
  }

  if (userLimiter.count >= limit) {
    logInvalidPermission('rate_limit_exceeded', `User ${userId} exceeded permission check rate limit`, userId);
    return false;
  }

  userLimiter.count++;
  return true;
}