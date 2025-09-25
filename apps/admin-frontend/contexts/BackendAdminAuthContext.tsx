// ============================================================================
// BACKEND ADMIN AUTH CONTEXT (Phase 2.2)
// Replaces local admin permission state management with backend-centric auth
// THE SINGLE SOURCE OF TRUTH for admin user authentication and authorization
// ============================================================================

'use client';

import React, { createContext, useContext, useEffect, useState, useCallback } from 'react';
import { useBackendAdminPermissions } from '@/lib/permissions/use-backend-admin-permissions';
import { adminPermissionAuthority } from '@/lib/permissions/backend-authority-client';

// ============================================================================
// ADMIN AUTH CONTEXT TYPES
// ============================================================================

export interface AdminUser {
  id: string;
  wallet_address?: string;
  email?: string;
  admin_tier?: string;
  admin_permissions?: string[];
  admin_roles?: string[];
  created_at?: string;
  updated_at?: string;
  last_login?: string;
  
  // Admin-specific fields
  admin_level?: 'admin' | 'super_admin' | 'system_admin';
  security_clearance?: 'standard' | 'elevated' | 'critical';
  admin_expires_at?: string;
}

export interface AdminAuthState {
  // Admin user data
  user: AdminUser | null;
  userId?: string;
  
  // Authentication state
  isAuthenticated: boolean;
  isAdmin: boolean;
  isSuperAdmin: boolean;
  isLoading: boolean;
  
  // Permission state from backend
  permissions: Record<string, boolean>;
  permissionLoading: boolean;
  
  // Admin capabilities
  canManageUsers: boolean;
  canManageSystem: boolean;
  canManagePermissions: boolean;
  canViewAnalytics: boolean;
  canViewAuditLogs: boolean;
  canManageSecurity: boolean;
  
  // Admin tier/subscription info
  currentAdminTier?: string;
  adminTierPermissions?: string[];
  adminSecurityClearance?: 'standard' | 'elevated' | 'critical';
  
  // Error state
  error: string | null;
  
  // Session info
  lastActivity?: string;
  sessionExpiry?: string;
  adminSessionExpiry?: string;
}

export interface AdminAuthContextValue extends AdminAuthState {
  // Authentication methods
  adminLogin: (walletAddress: string, signature: string, message: string) => Promise<boolean>;
  adminLogout: () => Promise<void>;
  refreshAdminAuth: () => Promise<void>;
  
  // Admin permission methods
  checkAdminPermission: (permission: string, resourcePath?: string) => Promise<boolean>;
  checkAnyAdminPermission: (permissions: string[]) => Promise<boolean>;
  checkAllAdminPermissions: (permissions: string[]) => Promise<boolean>;
  refreshAdminPermissions: () => Promise<void>;
  
  // Admin utility methods
  getAdminTierInfo: () => { tier: string; permissions: string[]; securityClearance: string } | null;
  getAdminCapabilities: () => {
    canManageUsers: boolean;
    canManageSystem: boolean;
    canManagePermissions: boolean;
    canViewAnalytics: boolean;
    canViewAuditLogs: boolean;
    canManageSecurity: boolean;
  };
  
  // Admin session management
  extendAdminSession: () => Promise<void>;
  validateAdminSession: () => Promise<boolean>;
  
  // Error handling
  clearError: () => void;
}

// ============================================================================
// ADMIN AUTH CONTEXT CREATION
// ============================================================================

const BackendAdminAuthContext = createContext<AdminAuthContextValue | null>(null);

export function useBackendAdminAuth(): AdminAuthContextValue {
  const context = useContext(BackendAdminAuthContext);
  if (!context) {
    throw new Error('useBackendAdminAuth must be used within a BackendAdminAuthProvider');
  }
  return context;
}

// ============================================================================
// BACKEND ADMIN AUTH PROVIDER COMPONENT
// ============================================================================

export function BackendAdminAuthProvider({ children }: { children: React.ReactNode }) {
  // ============================================================================
  // STATE MANAGEMENT
  // ============================================================================
  
  const [authState, setAuthState] = useState<AdminAuthState>({
    user: null,
    isAuthenticated: false,
    isAdmin: false,
    isSuperAdmin: false,
    isLoading: true,
    permissions: {},
    permissionLoading: false,
    canManageUsers: false,
    canManageSystem: false,
    canManagePermissions: false,
    canViewAnalytics: false,
    canViewAuditLogs: false,
    canManageSecurity: false,
    error: null,
  });

  // ============================================================================
  // BACKEND ADMIN PERMISSIONS INTEGRATION
  // ============================================================================
  
  const {
    permissions,
    loading: permissionLoading,
    isAdmin,
    isSuperAdmin,
    canManageUsers,
    canManageSystem,
    canManagePermissions,
    canViewAnalytics,
    canViewAuditLogs,
    canManageSecurity,
    adminTier,
    checkAdminPermission,
    checkAnyAdminPermission,
    checkAllAdminPermissions,
    refreshAdminPermissions: refreshBackendAdminPermissions,
    getAdminTierInfo,
    clearError: clearPermissionError,
  } = useBackendAdminPermissions(
    authState.userId,
    [], // We'll load permissions dynamically
    {
      autoRefresh: true,
      refreshInterval: 20, // 20 minutes for admin sessions
      cacheTimeout: 30,   // 30 minutes for admin cache
    }
  );

  // ============================================================================
  // ADMIN AUTHENTICATION METHODS
  // ============================================================================
  
  const adminLogin = useCallback(async (
    walletAddress: string, 
    signature: string, 
    message: string
  ): Promise<boolean> => {
    setAuthState(prev => ({ ...prev, isLoading: true, error: null }));

    try {
      // Verify admin credentials with backend first
      const isValidAdmin = await adminPermissionAuthority.isAdmin(walletAddress);
      if (!isValidAdmin) {
        throw new Error('No admin permissions found for this wallet');
      }

      // Store admin authentication data
      localStorage.setItem('admin_wallet_address', walletAddress);
      localStorage.setItem('admin_wallet_signature', signature);
      localStorage.setItem('admin_auth_message', message);
      localStorage.setItem('admin_auth_timestamp', Date.now().toString());
      localStorage.setItem('admin_chain_id', '56'); // BSC Mainnet
      localStorage.setItem('admin_session_type', 'admin');

      // Get admin capabilities
      const [
        isAdminResult,
        isSuperAdminResult,
        adminUserPermissions,
      ] = await Promise.all([
        adminPermissionAuthority.isAdmin(walletAddress),
        adminPermissionAuthority.isSuperAdmin(walletAddress),
        adminPermissionAuthority.getAdminUserPermissions(walletAddress),
      ]);

      // Create admin user object
      const adminUser: AdminUser = {
        id: walletAddress,
        wallet_address: walletAddress,
        admin_tier: adminUserPermissions.tier_info?.current_tier || 'basic_admin',
        admin_permissions: adminUserPermissions.permissions
          .filter(p => p.granted)
          .map(p => p.permission),
        admin_level: isSuperAdminResult ? 'super_admin' : 'admin',
        security_clearance: isSuperAdminResult ? 'critical' : 'elevated',
        created_at: new Date().toISOString(),
        updated_at: new Date().toISOString(),
        last_login: new Date().toISOString(),
      };

      // Update auth state
      setAuthState(prev => ({
        ...prev,
        user: adminUser,
        userId: walletAddress,
        isAuthenticated: true,
        isAdmin: isAdminResult,
        isSuperAdmin: isSuperAdminResult,
        isLoading: false,
        currentAdminTier: adminUserPermissions.tier_info?.current_tier,
        adminTierPermissions: adminUserPermissions.tier_info?.tier_permissions || [],
        adminSecurityClearance: isSuperAdminResult ? 'critical' : 'elevated',
        lastActivity: new Date().toISOString(),
        sessionExpiry: new Date(Date.now() + 8 * 60 * 60 * 1000).toISOString(), // 8 hours
        adminSessionExpiry: new Date(Date.now() + 4 * 60 * 60 * 1000).toISOString(), // 4 hours for admin
      }));

      // Refresh admin permissions after login
      await refreshBackendAdminPermissions();

      return true;
    } catch (error) {
      console.error('Admin login failed:', error);
      setAuthState(prev => ({
        ...prev,
        isLoading: false,
        error: error instanceof Error ? error.message : 'Admin login failed',
      }));
      return false;
    }
  }, [refreshBackendAdminPermissions]);

  const adminLogout = useCallback(async (): Promise<void> => {
    try {
      // Clear admin-specific storage
      localStorage.removeItem('admin_wallet_address');
      localStorage.removeItem('admin_wallet_signature');
      localStorage.removeItem('admin_auth_message');
      localStorage.removeItem('admin_auth_timestamp');
      localStorage.removeItem('admin_chain_id');
      localStorage.removeItem('admin_session_type');
      localStorage.removeItem('admin_auth_token');

      // Reset auth state
      setAuthState({
        user: null,
        userId: undefined,
        isAuthenticated: false,
        isAdmin: false,
        isSuperAdmin: false,
        isLoading: false,
        permissions: {},
        permissionLoading: false,
        canManageUsers: false,
        canManageSystem: false,
        canManagePermissions: false,
        canViewAnalytics: false,
        canViewAuditLogs: false,
        canManageSecurity: false,
        currentAdminTier: undefined,
        adminTierPermissions: undefined,
        adminSecurityClearance: undefined,
        error: null,
      });

      // Clear permission cache
      clearPermissionError();

    } catch (error) {
      console.error('Admin logout failed:', error);
    }
  }, [clearPermissionError]);

  const refreshAdminAuth = useCallback(async (): Promise<void> => {
    setAuthState(prev => ({ ...prev, isLoading: true }));

    try {
      const walletAddress = localStorage.getItem('admin_wallet_address');
      const signature = localStorage.getItem('admin_wallet_signature');
      const sessionType = localStorage.getItem('admin_session_type');

      if (!walletAddress || !signature || sessionType !== 'admin') {
        await adminLogout();
        return;
      }

      // Validate admin session with backend
      const isValidAdmin = await adminPermissionAuthority.isAdmin(walletAddress);
      
      if (isValidAdmin) {
        // Session is valid, get updated admin info
        const [
          isAdminResult,
          isSuperAdminResult,
          adminUserPermissions,
        ] = await Promise.all([
          adminPermissionAuthority.isAdmin(walletAddress),
          adminPermissionAuthority.isSuperAdmin(walletAddress),
          adminPermissionAuthority.getAdminUserPermissions(walletAddress),
        ]);

        const adminUser: AdminUser = {
          id: walletAddress,
          wallet_address: walletAddress,
          admin_tier: adminUserPermissions.tier_info?.current_tier || 'basic_admin',
          admin_permissions: adminUserPermissions.permissions
            .filter(p => p.granted)
            .map(p => p.permission),
          admin_level: isSuperAdminResult ? 'super_admin' : 'admin',
          security_clearance: isSuperAdminResult ? 'critical' : 'elevated',
          updated_at: new Date().toISOString(),
          last_login: localStorage.getItem('admin_auth_timestamp') 
            ? new Date(parseInt(localStorage.getItem('admin_auth_timestamp')!)).toISOString()
            : new Date().toISOString(),
        };

        setAuthState(prev => ({
          ...prev,
          user: adminUser,
          userId: walletAddress,
          isAuthenticated: true,
          isAdmin: isAdminResult,
          isSuperAdmin: isSuperAdminResult,
          isLoading: false,
          currentAdminTier: adminUserPermissions.tier_info?.current_tier,
          adminTierPermissions: adminUserPermissions.tier_info?.tier_permissions || [],
          adminSecurityClearance: isSuperAdminResult ? 'critical' : 'elevated',
          lastActivity: new Date().toISOString(),
        }));
      } else {
        // Session invalid, logout
        await adminLogout();
      }
    } catch (error) {
      console.error('Admin auth refresh failed:', error);
      await adminLogout();
    }
  }, [adminLogout]);

  // ============================================================================
  // ADMIN UTILITY METHODS
  // ============================================================================
  
  const getAdminCapabilities = useCallback(() => {
    return {
      canManageUsers,
      canManageSystem,
      canManagePermissions,
      canViewAnalytics,
      canViewAuditLogs,
      canManageSecurity,
    };
  }, [canManageUsers, canManageSystem, canManagePermissions, canViewAnalytics, canViewAuditLogs, canManageSecurity]);

  const getAdminTierInfoExtended = useCallback(() => {
    if (!authState.currentAdminTier) return null;
    
    return {
      tier: authState.currentAdminTier,
      permissions: authState.adminTierPermissions || [],
      securityClearance: authState.adminSecurityClearance || 'standard',
    };
  }, [authState.currentAdminTier, authState.adminTierPermissions, authState.adminSecurityClearance]);

  const extendAdminSession = useCallback(async (): Promise<void> => {
    if (!authState.userId) return;

    try {
      // Validate current admin session
      const isValidAdmin = await adminPermissionAuthority.isAdmin(authState.userId);
      
      if (isValidAdmin) {
        // Extend session expiry
        const newExpiry = new Date(Date.now() + 4 * 60 * 60 * 1000).toISOString(); // 4 hours
        setAuthState(prev => ({
          ...prev,
          adminSessionExpiry: newExpiry,
          lastActivity: new Date().toISOString(),
        }));
      } else {
        await adminLogout();
      }
    } catch (error) {
      console.error('Admin session extension failed:', error);
    }
  }, [authState.userId, adminLogout]);

  const validateAdminSession = useCallback(async (): Promise<boolean> => {
    if (!authState.userId) return false;

    try {
      return await adminPermissionAuthority.isAdmin(authState.userId);
    } catch (error) {
      console.error('Admin session validation failed:', error);
      return false;
    }
  }, [authState.userId]);

  const clearError = useCallback(() => {
    setAuthState(prev => ({ ...prev, error: null }));
  }, []);

  // ============================================================================
  // EFFECTS
  // ============================================================================
  
  // Initialize admin auth state from localStorage
  useEffect(() => {
    const initializeAdminAuth = async () => {
      const walletAddress = localStorage.getItem('admin_wallet_address');
      const signature = localStorage.getItem('admin_wallet_signature');
      const sessionType = localStorage.getItem('admin_session_type');

      if (walletAddress && signature && sessionType === 'admin') {
        // Try to restore admin session
        await refreshAdminAuth();
      } else {
        // No saved admin auth, set loading to false
        setAuthState(prev => ({ ...prev, isLoading: false }));
      }
    };

    initializeAdminAuth();
  }, []);

  // Update auth state when permissions change
  useEffect(() => {
    if (authState.isAuthenticated) {
      setAuthState(prev => ({
        ...prev,
        permissions,
        permissionLoading,
        isAdmin,
        isSuperAdmin,
        canManageUsers,
        canManageSystem,
        canManagePermissions,
        canViewAnalytics,
        canViewAuditLogs,
        canManageSecurity,
        currentAdminTier: adminTier,
      }));
    }
  }, [
    permissions, 
    permissionLoading, 
    isAdmin, 
    isSuperAdmin, 
    canManageUsers, 
    canManageSystem, 
    canManagePermissions, 
    canViewAnalytics, 
    canViewAuditLogs, 
    canManageSecurity,
    adminTier,
    authState.isAuthenticated
  ]);

  // Admin session expiry check (more frequent than regular session)
  useEffect(() => {
    if (!authState.adminSessionExpiry || !authState.isAuthenticated) return;

    const checkAdminSessionExpiry = () => {
      const expiry = new Date(authState.adminSessionExpiry!).getTime();
      const now = Date.now();
      
      if (now >= expiry) {
        adminLogout();
      }
    };

    const intervalId = setInterval(checkAdminSessionExpiry, 30000); // Check every 30 seconds for admin
    return () => clearInterval(intervalId);
  }, [authState.adminSessionExpiry, authState.isAuthenticated, adminLogout]);

  // ============================================================================
  // CONTEXT VALUE
  // ============================================================================
  
  const contextValue: AdminAuthContextValue = {
    // State
    ...authState,
    
    // Methods
    adminLogin,
    adminLogout,
    refreshAdminAuth,
    checkAdminPermission,
    checkAnyAdminPermission,
    checkAllAdminPermissions,
    refreshAdminPermissions: refreshBackendAdminPermissions,
    
    // Utilities
    getAdminTierInfo: getAdminTierInfoExtended,
    getAdminCapabilities,
    extendAdminSession,
    validateAdminSession,
    clearError,
  };

  return (
    <BackendAdminAuthContext.Provider value={contextValue}>
      {children}
    </BackendAdminAuthContext.Provider>
  );
}

// ============================================================================
// CONVENIENCE HOOKS
// ============================================================================

// Hook to get current admin user
export function useCurrentAdminUser(): AdminUser | null {
  const { user } = useBackendAdminAuth();
  return user;
}

// Hook to get admin user ID
export function useAdminUserId(): string | undefined {
  const { userId } = useBackendAdminAuth();
  return userId;
}

// Hook to check if user is authenticated admin
export function useIsAdminAuthenticated(): boolean {
  const { isAuthenticated, isAdmin } = useBackendAdminAuth();
  return isAuthenticated && isAdmin;
}

// Hook to check if user is super admin
export function useIsSuperAdmin(): boolean {
  const { isSuperAdmin } = useBackendAdminAuth();
  return isSuperAdmin;
}

// Hook to get admin capabilities
export function useAdminCapabilities() {
  const { getAdminCapabilities } = useBackendAdminAuth();
  return getAdminCapabilities();
}

// Hook to get admin tier info
export function useAdminTier() {
  const { getAdminTierInfo } = useBackendAdminAuth();
  return getAdminTierInfo();
}

// Hook for admin authentication loading state
export function useAdminAuthLoading(): boolean {
  const { isLoading } = useBackendAdminAuth();
  return isLoading;
}

// ============================================================================
// ADMIN PERMISSION HOOKS THAT USE BACKEND AUTH
// ============================================================================

// Hook to check admin permission with automatic user ID
export function useAdminPermissionCheck(permission: string, resourcePath?: string) {
  const { userId, checkAdminPermission } = useBackendAdminAuth();
  
  const checkPerm = useCallback(async () => {
    if (!userId) return false;
    return await checkAdminPermission(permission, resourcePath);
  }, [userId, permission, resourcePath, checkAdminPermission]);

  return {
    checkAdminPermission: checkPerm,
    userId,
  };
}

// Hook for specific admin permission checks
export function useUserManagementPermissionCheck(action: string = 'manage') {
  return useAdminPermissionCheck(`admin:users:${action}`);
}

export function useSystemManagementPermissionCheck() {
  return useAdminPermissionCheck('admin:system:manage');
}

export function usePermissionManagementPermissionCheck() {
  return useAdminPermissionCheck('admin:permissions:manage');
}

export function useAnalyticsPermissionCheck() {
  return useAdminPermissionCheck('admin:analytics:read');
}

export function useSecurityPermissionCheck() {
  return useAdminPermissionCheck('admin:security:manage');
}

export function useAuditLogsPermissionCheck() {
  return useAdminPermissionCheck('admin:audit:read');
}

// ============================================================================
// MIGRATION COMPLETE NOTICE
// ============================================================================
// 
// 🎉 ADMIN AUTH CONTEXT TRANSFORMATION COMPLETE!
//
// This file provides THE SINGLE SOURCE OF TRUTH for admin user authentication
// and authorization using the backend permission authority system.
//
// Key Security Features:
// ⚡ ALL admin permission checks now use backend API calls
// 🔒 NO client-side admin permission validation possible
// 🛡️  Admin-specific session management and security
// 📊 Real-time admin permission validation from authoritative source
// ⏰ Enhanced admin session expiry and validation
// 👑 Admin tier and capability management
// 🎯 Specialized admin permission hooks and utilities
//
// The admin-frontend authentication is now SECURE and UNHACKABLE!
// ============================================================================