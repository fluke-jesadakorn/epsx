'use client'

import { useState } from 'react'
import { useAuth } from '@/lib/auth'
import { usePermissions, useAnalyticsPermissions, usePaymentPermissions, useTokenPermissions } from '@/hooks/usePermissions'
import PlatformSwitcher from '@/components/auth/PlatformSwitcher'
import PlatformPermissionGuard, { 
  RequirePermission, 
  RequireRole, 
  RequireTier, 
  RequireAccess 
} from '@/components/auth/PlatformPermissionGuard'
import PlatformContextIndicator from '@/components/auth/PlatformContextIndicator'

export default function CrossPlatformExample() {
  const [selectedResource, setSelectedResource] = useState('analytics')
  const [selectedAction, setSelectedAction] = useState('read')
  
  const { user, isAuthenticated, error } = useAuth.getState()
  const permissions = usePermissions()
  const analyticsPerms = useAnalyticsPermissions()
  const paymentPerms = usePaymentPermissions()
  const tokenPerms = useTokenPermissions()
  
  if (!isAuthenticated) {
    return (
      <div className="p-6 bg-gray-50 rounded-lg">
        <h2 className="text-lg font-semibold text-gray-900 mb-4">
          Cross-Platform Authentication Example
        </h2>
        <p className="text-gray-600">Please log in to see cross-platform functionality.</p>
      </div>
    )
  }
  
  if (!user) {
    return (
      <div className="p-6 bg-red-50 rounded-lg">
        <p className="text-red-600">Error: User data not available</p>
        {error && <p className="text-sm text-red-500 mt-2">{error}</p>}
      </div>
    )
  }
  
  return (
    <div className="space-y-8 p-6 bg-white rounded-lg border border-gray-200">
      <div className="flex items-center justify-between">
        <h2 className="text-xl font-semibold text-gray-900">
          Cross-Platform System Demo
        </h2>
        <PlatformSwitcher />
      </div>
      
      {/* Platform Context Display */}
      <section className="space-y-4">
        <h3 className="text-lg font-medium text-gray-900">Platform Context</h3>
        
        <div className="grid grid-cols-1 md:grid-cols-3 gap-4">
          <div className="p-4 bg-blue-50 rounded-lg">
            <h4 className="font-medium text-blue-900">Current Platform</h4>
            <p className="text-2xl font-bold text-blue-600">
              {permissions.currentPlatform}
            </p>
          </div>
          
          <div className="p-4 bg-green-50 rounded-lg">
            <h4 className="font-medium text-green-900">Available Platforms</h4>
            <p className="text-sm text-green-700">
              {permissions.availablePlatforms.join(', ')}
            </p>
          </div>
          
          <div className="p-4 bg-purple-50 rounded-lg">
            <h4 className="font-medium text-purple-900">User Tier</h4>
            <p className="text-lg font-semibold text-purple-600">
              {user.package_tier}
            </p>
          </div>
        </div>
        
        <PlatformContextIndicator variant="full" />
      </section>
      
      {/* Permission Testing Interface */}
      <section className="space-y-4">
        <h3 className="text-lg font-medium text-gray-900">Permission Testing</h3>
        
        <div className="grid grid-cols-2 gap-4">
          <div>
            <label className="block text-sm font-medium text-gray-700 mb-2">
              Resource
            </label>
            <select
              value={selectedResource}
              onChange={(e) => setSelectedResource(e.target.value)}
              className="w-full px-3 py-2 border border-gray-300 rounded-md"
            >
              <option value="analytics">Analytics</option>
              <option value="payments">Payments</option>
              <option value="tokens">Tokens</option>
              <option value="users">Users</option>
              <option value="dashboard">Dashboard</option>
            </select>
          </div>
          
          <div>
            <label className="block text-sm font-medium text-gray-700 mb-2">
              Action
            </label>
            <select
              value={selectedAction}
              onChange={(e) => setSelectedAction(e.target.value)}
              className="w-full px-3 py-2 border border-gray-300 rounded-md"
            >
              <option value="read">Read</option>
              <option value="write">Write</option>
              <option value="create">Create</option>
              <option value="delete">Delete</option>
              <option value="manage">Manage</option>
            </select>
          </div>
        </div>
        
        <div className="p-4 bg-gray-50 rounded-lg">
          <p className="text-sm text-gray-600 mb-2">
            Testing permission: <code className="bg-gray-200 px-2 py-1 rounded">
              {permissions.currentPlatform}:{selectedResource}:{selectedAction}
            </code>
          </p>
          <p className={`font-medium ${
            permissions.hasPermission(selectedResource, selectedAction) 
              ? 'text-green-600' 
              : 'text-red-600'
          }`}>
            {permissions.hasPermission(selectedResource, selectedAction) ? '✅ Allowed' : '❌ Denied'}
          </p>
          
          <div className="mt-3 text-sm text-gray-600">
            <p>Access Level: <strong>{permissions.getAccessLevel(selectedResource)}</strong></p>
          </div>
        </div>
      </section>
      
      {/* Platform-Specific Features */}
      <section className="space-y-6">
        <h3 className="text-lg font-medium text-gray-900">Platform-Specific Features</h3>
        
        {/* EPSX Analytics Features */}
        <div className="border border-gray-200 rounded-lg p-4">
          <h4 className="font-medium text-gray-900 mb-3 flex items-center gap-2">
            📈 EPSX Analytics Features
          </h4>
          
          <div className="space-y-2">
            <RequirePermission permission="epsx:analytics:read" fallback={
              <p className="text-red-600 text-sm">❌ No analytics access</p>
            }>
              <p className="text-green-600 text-sm">✅ Can view analytics</p>
            </RequirePermission>
            
            <RequireAccess resource="export" action="read" fallback={
              <p className="text-red-600 text-sm">❌ No data export access</p>
            }>
              <p className="text-green-600 text-sm">✅ Can export data</p>
            </RequireAccess>
            
            <RequireTier tier="BRONZE" fallback={
              <p className="text-yellow-600 text-sm">❌ Premium features require Bronze tier</p>
            }>
              <p className="text-green-600 text-sm">✅ Premium analytics available</p>
            </RequireTier>
          </div>
        </div>
        
        {/* EPSX Pay Features */}
        <div className="border border-gray-200 rounded-lg p-4">
          <h4 className="font-medium text-gray-900 mb-3 flex items-center gap-2">
            💳 EPSX Pay Features
          </h4>
          
          <PlatformPermissionGuard
            platform="epsx-pay"
            fallback={<p className="text-red-600 text-sm">❌ No access to EPSX Pay platform</p>}
          >
            <div className="space-y-2">
              <RequireAccess resource="payments" action="read" platform="epsx-pay" fallback={
                <p className="text-red-600 text-sm">❌ No payment viewing access</p>
              }>
                <p className="text-green-600 text-sm">✅ Can view payments</p>
              </RequireAccess>
              
              <RequireAccess resource="payments" action="write" platform="epsx-pay" fallback={
                <p className="text-red-600 text-sm">❌ No payment processing access</p>
              }>
                <p className="text-green-600 text-sm">✅ Can process payments</p>
              </RequireAccess>
            </div>
          </PlatformPermissionGuard>
        </div>
        
        {/* EPSX Token Features */}
        <div className="border border-gray-200 rounded-lg p-4">
          <h4 className="font-medium text-gray-900 mb-3 flex items-center gap-2">
            🪙 EPSX Token Features
          </h4>
          
          <PlatformPermissionGuard
            platform="epsx-token"
            fallback={<p className="text-red-600 text-sm">❌ No access to EPSX Token platform</p>}
          >
            <div className="space-y-2">
              <RequireAccess resource="governance" action="vote" platform="epsx-token" fallback={
                <p className="text-red-600 text-sm">❌ No voting access</p>
              }>
                <p className="text-green-600 text-sm">✅ Can vote on proposals</p>
              </RequireAccess>
              
              <RequireAccess resource="governance" action="create" platform="epsx-token" fallback={
                <p className="text-red-600 text-sm">❌ Cannot create proposals</p>
              }>
                <p className="text-green-600 text-sm">✅ Can create proposals</p>
              </RequireAccess>
            </div>
          </PlatformPermissionGuard>
        </div>
      </section>
      
      {/* Role and Tier Checks */}
      <section className="space-y-4">
        <h3 className="text-lg font-medium text-gray-900">Role & Tier Checks</h3>
        
        <div className="grid grid-cols-1 md:grid-cols-2 gap-4">
          <div className="p-4 border border-gray-200 rounded-lg">
            <h4 className="font-medium text-gray-900 mb-2">Role-Based Access</h4>
            
            <RequireRole role="admin" fallback={
              <p className="text-red-600 text-sm">❌ Admin features not available</p>
            }>
              <p className="text-green-600 text-sm">✅ Admin features available</p>
            </RequireRole>
            
            <RequireRole role="user" fallback={
              <p className="text-red-600 text-sm">❌ User features not available</p>
            }>
              <p className="text-green-600 text-sm">✅ User features available</p>
            </RequireRole>
          </div>
          
          <div className="p-4 border border-gray-200 rounded-lg">
            <h4 className="font-medium text-gray-900 mb-2">Tier-Based Features</h4>
            
            <RequireTier tier="ENTERPRISE" fallback={
              <p className="text-yellow-600 text-sm">❌ Enterprise features require upgrade</p>
            }>
              <p className="text-green-600 text-sm">✅ Enterprise features available</p>
            </RequireTier>
            
            <RequireTier tier="GOLD" fallback={
              <p className="text-yellow-600 text-sm">❌ Premium features require Gold tier</p>
            }>
              <p className="text-green-600 text-sm">✅ Gold tier features available</p>
            </RequireTier>
          </div>
        </div>
      </section>
      
      {/* User Information */}
      <section className="space-y-4">
        <h3 className="text-lg font-medium text-gray-900">User Information</h3>
        
        <div className="p-4 bg-gray-50 rounded-lg">
          <dl className="grid grid-cols-1 md:grid-cols-2 gap-4 text-sm">
            <div>
              <dt className="font-medium text-gray-900">User ID:</dt>
              <dd className="text-gray-600">{user.id}</dd>
            </div>
            <div>
              <dt className="font-medium text-gray-900">Email:</dt>
              <dd className="text-gray-600">{user.email}</dd>
            </div>
            <div>
              <dt className="font-medium text-gray-900">Role:</dt>
              <dd className="text-gray-600">{user.role}</dd>
            </div>
            <div>
              <dt className="font-medium text-gray-900">Package Tier:</dt>
              <dd className="text-gray-600">{user.package_tier}</dd>
            </div>
            <div>
              <dt className="font-medium text-gray-900">Current Platform:</dt>
              <dd className="text-gray-600">{user.platform_context || user.primary_platform}</dd>
            </div>
            <div>
              <dt className="font-medium text-gray-900">Available Platforms:</dt>
              <dd className="text-gray-600">{user.platforms?.join(', ')}</dd>
            </div>
          </dl>
          
          <div className="mt-4">
            <dt className="font-medium text-gray-900 mb-2">Permissions:</dt>
            <dd className="text-gray-600">
              <div className="flex flex-wrap gap-1">
                {user.permissions?.map((permission, index) => (
                  <span
                    key={index}
                    className="inline-block px-2 py-1 text-xs bg-blue-100 text-blue-800 rounded"
                  >
                    {permission}
                  </span>
                ))}
              </div>
            </dd>
          </div>
        </div>
      </section>
    </div>
  )
}