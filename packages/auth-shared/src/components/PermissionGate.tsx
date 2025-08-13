import React from 'react'
import { useModernAuth } from '../hooks/useModernAuth'

/**
 * Props for permission-based components
 */
interface PermissionGateProps {
  /** Required permission to access content */
  permission?: string
  
  /** Required admin module to access content */
  adminModule?: string
  
  /** Required package tier to access content */
  packageTier?: string
  
  /** Required role to access content */
  role?: string
  
  /** Multiple permissions (user must have ALL) */
  permissions?: string[]
  
  /** Multiple admin modules (user must have ALL) */
  adminModules?: string[]
  
  /** Alternative permissions (user must have ANY) */
  anyPermissions?: string[]
  
  /** Alternative admin modules (user must have ANY) */
  anyAdminModules?: string[]
  
  /** Custom access check function */
  customCheck?: (user: any) => boolean
  
  /** Component to render when access is denied */
  fallback?: React.ReactNode
  
  /** Show loading state while checking auth */
  showLoading?: boolean
  
  /** Children to render when access is granted */
  children: React.ReactNode
}

/**
 * Universal permission gate component that replaces all complex auth guards
 * Uses JWT-based permissions from Auth.js v5 session
 */
export function PermissionGate({
  permission,
  adminModule,
  packageTier,
  role,
  permissions = [],
  adminModules = [],
  anyPermissions = [],
  anyAdminModules = [],
  customCheck,
  fallback = <AccessDenied />,
  showLoading = true,
  children
}: PermissionGateProps) {
  const { 
    user, 
    isLoading, 
    isAuthenticated, 
    hasPermission, 
    hasAdminModule, 
    hasPackageTier, 
    hasRole 
  } = useModernAuth()

  // Show loading state
  if (isLoading && showLoading) {
    return <AuthLoading />
  }

  // Not authenticated
  if (!isAuthenticated) {
    return <LoginRequired />
  }

  // Custom check takes precedence
  if (customCheck) {
    return customCheck(user) ? <>{children}</> : fallback
  }

  // Check single permission
  if (permission && !hasPermission(permission)) {
    return fallback
  }

  // Check single admin module
  if (adminModule && !hasAdminModule(adminModule)) {
    return fallback
  }

  // Check package tier
  if (packageTier && !hasPackageTier(packageTier)) {
    return fallback
  }

  // Check role
  if (role && !hasRole(role)) {
    return fallback
  }

  // Check multiple permissions (ALL required)
  if (permissions.length > 0 && !permissions.every(p => hasPermission(p))) {
    return fallback
  }

  // Check multiple admin modules (ALL required)
  if (adminModules.length > 0 && !adminModules.every(m => hasAdminModule(m))) {
    return fallback
  }

  // Check alternative permissions (ANY required)
  if (anyPermissions.length > 0 && !anyPermissions.some(p => hasPermission(p))) {
    return fallback
  }

  // Check alternative admin modules (ANY required)
  if (anyAdminModules.length > 0 && !anyAdminModules.some(m => hasAdminModule(m))) {
    return fallback
  }

  // All checks passed
  return <>{children}</>
}

/**
 * Admin-only gate component
 */
export function AdminGate({ 
  children, 
  fallback = <AccessDenied /> 
}: { 
  children: React.ReactNode
  fallback?: React.ReactNode 
}) {
  return (
    <PermissionGate
      customCheck={(user) => user?.isAdmin || false}
      fallback={fallback}
    >
      {children}
    </PermissionGate>
  )
}

/**
 * System Admin only gate component
 */
export function SystemAdminGate({ 
  children, 
  fallback = <AccessDenied /> 
}: { 
  children: React.ReactNode
  fallback?: React.ReactNode 
}) {
  return (
    <PermissionGate
      adminModule="system_admin"
      fallback={fallback}
    >
      {children}
    </PermissionGate>
  )
}

/**
 * Package tier gate component
 */
export function PackageTierGate({ 
  tier, 
  children, 
  fallback = <UpgradeRequired tier={tier} /> 
}: { 
  tier: string
  children: React.ReactNode
  fallback?: React.ReactNode 
}) {
  return (
    <PermissionGate
      packageTier={tier}
      fallback={fallback}
    >
      {children}
    </PermissionGate>
  )
}

/**
 * Premium features gate (Bronze tier or higher)
 */
export function PremiumGate({ 
  children, 
  fallback = <UpgradeRequired tier="BRONZE" /> 
}: { 
  children: React.ReactNode
  fallback?: React.ReactNode 
}) {
  return (
    <PackageTierGate tier="BRONZE" fallback={fallback}>
      {children}
    </PackageTierGate>
  )
}

/**
 * Loading component for authentication checks
 */
function AuthLoading() {
  return (
    <div className="flex items-center justify-center p-4">
      <div className="animate-spin h-5 w-5 border-2 border-blue-500 border-t-transparent rounded-full"></div>
      <span className="ml-2 text-sm text-gray-600">Checking permissions...</span>
    </div>
  )
}

/**
 * Login required component
 */
function LoginRequired() {
  return (
    <div className="text-center p-6 bg-gray-50 rounded-lg border">
      <h3 className="text-lg font-medium text-gray-900 mb-2">
        Login Required
      </h3>
      <p className="text-gray-600 mb-4">
        You need to be logged in to access this content.
      </p>
      <a 
        href="/login" 
        className="inline-flex items-center px-4 py-2 bg-blue-600 text-white rounded-md hover:bg-blue-700 transition-colors"
      >
        Login
      </a>
    </div>
  )
}

/**
 * Access denied component
 */
function AccessDenied() {
  return (
    <div className="text-center p-6 bg-red-50 rounded-lg border border-red-200">
      <h3 className="text-lg font-medium text-red-900 mb-2">
        Access Denied
      </h3>
      <p className="text-red-700">
        You don't have permission to access this content.
      </p>
    </div>
  )
}

/**
 * Upgrade required component
 */
function UpgradeRequired({ tier }: { tier: string }) {
  return (
    <div className="text-center p-6 bg-yellow-50 rounded-lg border border-yellow-200">
      <h3 className="text-lg font-medium text-yellow-900 mb-2">
        Upgrade Required
      </h3>
      <p className="text-yellow-700 mb-4">
        This feature requires {tier} tier or higher.
      </p>
      <a 
        href="/payment" 
        className="inline-flex items-center px-4 py-2 bg-yellow-600 text-white rounded-md hover:bg-yellow-700 transition-colors"
      >
        Upgrade Now
      </a>
    </div>
  )
}