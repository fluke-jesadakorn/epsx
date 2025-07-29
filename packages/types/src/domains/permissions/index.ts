// Re-export all permission types from the existing file
export * from '../../permission_profile';

// Additional consolidated permission types
export interface Permission {
  id: string;
  name: string;
  description: string;
  resource: string;
  action: string;
  scope?: string;
}

export interface Role {
  id: string;
  name: string;
  description: string;
  permissions: Permission[];
  isSystem: boolean;
  level: number;
}

export interface PermissionCheckRequest {
  userId: string;
  permission: string;
  resource?: string;
  context?: Record<string, any>;
}

export interface PermissionCheckResponse {
  allowed: boolean;
  reason?: string;
  context?: Record<string, any>;
}

export interface UserPermissionStatus {
  userId: string;
  permissions: Permission[];
  roles: Role[];
  effectivePermissions: string[];
  lastUpdated: Date;
}