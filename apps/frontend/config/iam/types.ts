// ============================================================================
// SIMPLE ROLE TYPES - MATCHES BACKEND EXACTLY
// ============================================================================
// Simple types that align with the backend role system

export interface SimpleRole {
  id: string;
  name: string;
  description: string;
  color: string;
  icon: string;
  features: string[];
  isSystem: boolean;
  isActive: boolean;
}

export interface SimpleFeature {
  id: string;
  name: string;
  description: string;
  category: string;
  isSystem: boolean;
}

// Legacy interface for backward compatibility
export interface RoleDocument extends SimpleRole {
  permissions: string[]; // Maps to features for compatibility
}

// Legacy interface for backward compatibility  
export interface PermissionDocument extends SimpleFeature {
  action: string;
  resource: string;
  scope: string;
  tags: string[];
}

// Simple user claims that match backend
export interface SimpleUserProfile {
  firebase_uid: string;
  email: string;
  role: 'admin' | 'user' | 'guest';
  display_name?: string;
  name?: string;
  avatar_url?: string;
  is_active: boolean;
  last_login_at?: string;
}

// Feature access result
export interface FeatureAccessResult {
  hasAccess: boolean;
  role: string;
  feature: string;
  reason?: string;
}

// Role comparison result
export interface RoleComparisonResult {
  canAccess: boolean;
  userRole: string;
  requiredRole: string;
  hierarchy: number;
}