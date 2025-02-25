import { useContext } from 'react';
import type { Permission, Role } from '@/constants/roles';
import { ROLES, hasPermission } from '@/constants/roles';
import { useAuth as useAuthContext } from '@/contexts/AuthContext';
import type { ComponentType, FC, ReactElement } from 'react';
import React from 'react';

export const useAuth = () => {
  const context = useAuthContext();

  if (!context) {
    throw new Error('useAuth must be used within an AuthProvider');
  }

  const { user } = context;

  return {
    ...context,
    // Role checking helpers
    isAdmin: user?.role === ROLES.ADMIN,
    isPremium: user?.role === ROLES.PREMIUM || user?.role === ROLES.ADMIN,

    // Permission checking helpers
    hasPermission: (permission: Permission) => {
      if (!user?.role) return false;
      return hasPermission(user.role, permission);
    },

    // Combined permission checking
    checkPermissions: (permissions: Permission[]) => {
      if (!user?.role) return false;
      return permissions.every(permission => hasPermission(user.role!, permission));
    },

    // Get all user permissions
    getPermissions: () => user?.permissions || [],

    // Get user role
    getRole: (): Role | undefined => user?.role,
  };
};

interface GuardProps {
  children: ReactElement;
}

// HOC for protecting components with permissions
export function withPermission<P extends object>(
  WrappedComponent: ComponentType<P>,
  requiredPermission: Permission
): ComponentType<P> {
  return function PermissionGuard(props: P) {
    const { hasPermission: checkPermission } = useAuth();
    
    if (!checkPermission(requiredPermission)) {
      return null; // Or return an unauthorized component
    }
    
    return React.createElement(WrappedComponent, props);
  };
}

// HOC for protecting components with roles
export function withRole<P extends object>(
  WrappedComponent: ComponentType<P>,
  requiredRole: Role
): ComponentType<P> {
  return function RoleGuard(props: P) {
    const { user } = useAuthContext();
    
    if (user?.role !== requiredRole) {
      return null; // Or return an unauthorized component
    }
    
    return React.createElement(WrappedComponent, props);
  };
}

// Component wrappers for JSX usage
export function PermissionGuard({ children, permission }: GuardProps & { permission: Permission }) {
  const { hasPermission: checkPermission } = useAuth();
  
  if (!checkPermission(permission)) {
    return null;
  }
  
  return children;
}

export function RoleGuard({ children, role }: GuardProps & { role: Role }) {
  const { user } = useAuthContext();
  
  if (user?.role !== role) {
    return null;
  }
  
  return children;
}
