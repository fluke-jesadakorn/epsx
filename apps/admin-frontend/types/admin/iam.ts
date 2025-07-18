// IAM Types for AWS-style permission system

export interface User {
  id: string;
  email: string;
  name?: string;
  displayName?: string;
  emailVerified: boolean;
  disabled: boolean;
  roles: string[];
  groups: string[];
  attachedPolicies: string[];
  status: 'active' | 'inactive' | 'disabled';
  lastActivity?: string;
  createdAt: string;
  updatedAt: string;
}

export interface Role {
  id: string;
  name: string;
  description: string;
  attachedPolicies: string[];
  trustPolicy?: PolicyDocument;
  maxSessionDuration?: number;
  path?: string;
  createdAt: string;
  updatedAt: string;
}

export interface Policy {
  id: string;
  name: string;
  description: string;
  policyDocument: PolicyDocument;
  arn?: string;
  path?: string;
  isAttachable: boolean;
  attachmentCount: number;
  createdAt: string;
  updatedAt: string;
}

export interface Group {
  id: string;
  name: string;
  description: string;
  memberCount: number;
  attachedPolicies: string[];
  path?: string;
  createdAt: string;
  updatedAt: string;
}

export interface PolicyDocument {
  Version: '2012-10-17';
  Statement: PolicyStatement[];
}

export interface PolicyStatement {
  Sid?: string;
  Effect: 'Allow' | 'Deny';
  Action: string | string[];
  Resource?: string | string[];
  Condition?: Record<string, any>;
  Principal?: string | string[] | { [key: string]: string | string[] };
}

export interface Permission {
  id: string;
  service: string;
  action: string;
  resource: string;
  effect: 'Allow' | 'Deny';
  conditions?: Record<string, any>;
}

export interface UserPermission {
  userId: string;
  permissionId: string;
  grantedBy: string;
  grantedAt: string;
  expiresAt?: string;
  source: 'direct' | 'role' | 'group' | 'policy';
  sourceId: string;
}

export interface AuditLog {
  id: string;
  userId: string;
  action: string;
  resource: string;
  effect: 'Allow' | 'Deny';
  timestamp: string;
  ipAddress?: string;
  userAgent?: string;
  details?: Record<string, any>;
}

export interface IAMStats {
  totalUsers: number;
  activeUsers: number;
  totalRoles: number;
  totalPolicies: number;
  totalGroups: number;
  totalPermissions: number;
}

// Form types for modals
export interface UserFormData {
  email: string;
  name?: string;
  displayName?: string;
  roles: string[];
  groups: string[];
  attachedPolicies: string[];
  status: 'active' | 'inactive' | 'disabled';
}

export interface RoleFormData {
  name: string;
  description: string;
  attachedPolicies: string[];
  trustPolicy?: PolicyDocument;
  maxSessionDuration?: number;
  path?: string;
}

export interface PolicyFormData {
  name: string;
  description: string;
  policyDocument: PolicyDocument;
  path?: string;
}

export interface GroupFormData {
  name: string;
  description: string;
  attachedPolicies: string[];
  path?: string;
}

// Policy templates
export const POLICY_TEMPLATES = {
  Bronze: {
    Version: '2012-10-17' as const,
    Statement: [
      {
        Effect: 'Allow' as const,
        Action: ['dashboard:read', 'profile:read', 'profile:update'],
        Resource: ['user:self']
      }
    ]
  },
  Silver: {
    Version: '2012-10-17' as const,
    Statement: [
      {
        Effect: 'Allow' as const,
        Action: ['dashboard:read', 'profile:read', 'profile:update', 'analytics:read'],
        Resource: ['user:self', 'analytics:basic']
      }
    ]
  },
  Gold: {
    Version: '2012-10-17' as const,
    Statement: [
      {
        Effect: 'Allow' as const,
        Action: ['dashboard:read', 'profile:read', 'profile:update', 'analytics:read', 'trading:read'],
        Resource: ['user:self', 'analytics:*', 'trading:*']
      }
    ]
  },
  Platinum: {
    Version: '2012-10-17' as const,
    Statement: [
      {
        Effect: 'Allow' as const,
        Action: ['dashboard:*', 'profile:*', 'analytics:*', 'trading:*', 'api:read'],
        Resource: ['user:self', 'analytics:*', 'trading:*', 'api:*']
      }
    ]
  },
  Admin: {
    Version: '2012-10-17' as const,
    Statement: [
      {
        Effect: 'Allow' as const,
        Action: ['*'],
        Resource: ['*']
      }
    ]
  }
};
