/**
 * User Permissions Content Component
 * Comprehensive permissions management consolidating IAM features
 */

'use client'

import { useState } from 'react'
import { Shield, Users, Key, Settings, Plus } from 'lucide-react'
import { Button } from '@/components/ui/button'
import type { UnifiedUserData } from '@/lib/types/unified-user'
import type { EnhancedAuthUser } from '@/lib/auth/server-auth'
import { adminCardVariants, adminButtonVariants, adminBadgeVariants, cn } from '@/design-system'
import { PermissionAssignmentCard } from './PermissionAssignmentCard'
import { RoleAssignmentCard } from './RoleAssignmentCard'
import { PermissionProfileCard } from '../permission-profiles/PermissionProfileCard'
import { RoleAssignmentForm } from './RoleAssignmentForm'
import { CustomPermissionForm } from './CustomPermissionForm'
import { PermissionProfileForm } from './PermissionProfileForm'
import { PermissionHistoryCard } from './PermissionHistoryCard'
import { TemporaryPermissionForm } from './TemporaryPermissionForm'
import { BulkOperationsInterface } from './BulkOperationsInterface'
import { PermissionImpactAnalysis } from './PermissionImpactAnalysis'
import { PermissionValidator } from './PermissionValidator'
import { PermissionConflictResolver } from './PermissionConflictResolver'
import { PermissionExportImport } from './PermissionExportImport'
import { PermissionProfileManager } from './PermissionProfileManager'
import { TemporaryPermissionManager } from './TemporaryPermissionManager'
import { PermissionAnalyticsDashboard } from '../analytics/PermissionAnalyticsDashboard'
import { PermissionRecommendations } from './PermissionRecommendations'
import { InteractivePermissionTreeView } from './InteractivePermissionTreeView'
import { DragDropBulkAssignment } from './DragDropBulkAssignment'

interface UserPermissionsContentProps {
  user: UnifiedUserData
  currentUser: EnhancedAuthUser
  onUserUpdated?: () => void
}

export function UserPermissionsContent({ 
  user, 
  currentUser, 
  onUserUpdated 
}: UserPermissionsContentProps) {
  const canManagePermissions = currentUser.isSuperAdmin || currentUser.canManageUsers
  

  // Calculate permission stats
  const activeRoles = user.roles.filter(r => r.isActive).length
  const totalPermissions = user.customPermissions.length
  const activeProfiles = user.permissionProfiles.filter(p => p.isActive).length

  // Get existing role names and profile IDs for the modals
  const existingRoles = user.roles.map(r => r.name)
  const existingProfileIds = user.permissionProfiles.map(p => p.id)

  const handleRefreshData = () => {
    onUserUpdated?.()
  }

  return (
    <div className="space-y-6">
      {/* Permission Summary Stats */}
      <div className="grid grid-cols-1 md:grid-cols-3 gap-4">
        <div className={cn(adminCardVariants({ variant: 'pancake' }))}>
          <div className="flex items-center justify-between">
            <div>
              <p className="text-2xl font-bold">{activeRoles}</p>
              <p className="text-sm text-muted-foreground">Active Roles</p>
            </div>
            <Users className="h-8 w-8 text-blue-500" />
          </div>
        </div>
        
        <div className={cn(adminCardVariants({ variant: 'pancake' }))}>
          <div className="flex items-center justify-between">
            <div>
              <p className="text-2xl font-bold">{totalPermissions}</p>
              <p className="text-sm text-muted-foreground">Custom Permissions</p>
            </div>
            <Key className="h-8 w-8 text-green-500" />
          </div>
        </div>

        <div className={cn(adminCardVariants({ variant: 'pancake' }))}>
          <div className="flex items-center justify-between">
            <div>
              <p className="text-2xl font-bold">{activeProfiles}</p>
              <p className="text-sm text-muted-foreground">Permission Profiles</p>
            </div>
            <Shield className="h-8 w-8 text-purple-500" />
          </div>
        </div>
      </div>

      <div className="grid grid-cols-1 lg:grid-cols-2 gap-6">
        {/* Role Assignments */}
        <div className={cn(adminCardVariants({ variant: 'pancake' }))}>
          <h3 className="text-lg font-semibold mb-4 flex items-center gap-2">
            <Users className="h-5 w-5" />
            Role Assignments
          </h3>
          
          <div className="space-y-4">
            {user.roles.length > 0 && (
              <div className="space-y-3">
                {user.roles.map((role) => (
                  <RoleAssignmentCard 
                    key={role.id}
                    role={role}
                    userId={user.id}
                    canManage={canManagePermissions}
                    onRoleRemoved={handleRefreshData}
                  />
                ))}
              </div>
            )}
            
            {canManagePermissions && (
              <RoleAssignmentForm
                userId={user.id}
                existingRoles={existingRoles}
                onRoleUpdated={handleRefreshData}
              />
            )}
          </div>
        </div>

        {/* Permission Profiles - Enhanced */}
        <div className={cn(adminCardVariants({ variant: 'pancake' }))}>
          <PermissionProfileManager
            userId={user.id}
            currentProfiles={user.permissionProfiles}
            canManage={canManagePermissions}
            onProfileUpdated={handleRefreshData}
          />
        </div>
      </div>

      {/* Custom Permissions */}
      <div className={cn(adminCardVariants({ variant: 'pancake' }))}>
        <h3 className="text-lg font-semibold mb-4 flex items-center gap-2">
          <Key className="h-5 w-5" />
          Custom Permissions
        </h3>
        
        <div className="space-y-4">
          {user.customPermissions.length > 0 && (
            <div className="space-y-2">
              {user.customPermissions.map((permission) => (
                <PermissionAssignmentCard 
                  key={permission.id}
                  permission={permission}
                  userId={user.id}
                  canManage={canManagePermissions}
                  onPermissionRemoved={handleRefreshData}
                />
              ))}
            </div>
          )}
          
          {canManagePermissions && (
            <CustomPermissionForm
              userId={user.id}
              existingPermissions={user.customPermissions.map(p => p.permission)}
              onPermissionUpdated={handleRefreshData}
            />
          )}
        </div>
      </div>

      {/* Permission History */}
      <PermissionHistoryCard userId={user.id} />
      
      {/* Advanced Management Tools */}
      {canManagePermissions && (
        <>
          {/* Temporary Permissions */}
          <div className={cn(adminCardVariants({ variant: 'pancake' }))}>
            <TemporaryPermissionManager userId={user.id} />
          </div>
          
          {/* Bulk Management */}
          <div className={cn(adminCardVariants({ variant: 'pancake' }))}>
            <BulkOperationsInterface
              selectedUserIds={[user.id]}
              onOperationComplete={handleRefreshData}
            />
          </div>
        </>
      )}
      
      {/* Permission Impact Analysis */}
      <div className={cn(adminCardVariants({ variant: 'pancake' }))}>
        <h3 className="text-lg font-semibold mb-4 flex items-center gap-2">
          <Shield className="h-5 w-5" />
          Permission Impact
        </h3>
        
        <PermissionImpactAnalysis userId={user.id} />
      </div>
      
      {/* Analytics Dashboard */}
      {canManagePermissions && (
        <div className={cn(adminCardVariants({ variant: 'pancake' }))}>
          <PermissionAnalyticsDashboard />
        </div>
      )}

      {/* Smart Recommendations */}
      {canManagePermissions && (
        <div className={cn(adminCardVariants({ variant: 'pancake' }))}>
          <PermissionRecommendations
            user={{
              id: user.id,
              email: user.email,
              currentPermissions: user.customPermissions.map(p => p.permission),
              role: user.roles[0]?.name || 'user',
              department: 'General' // Would be extracted from user data
            }}
            onApplyRecommendation={async (recommendation) => {
              // Handle recommendation application
              console.log('Applying recommendation:', recommendation);
              handleRefreshData();
            }}
          />
        </div>
      )}

      {/* Interactive Permission Tree */}
      {canManagePermissions && (
        <div className={cn(adminCardVariants({ variant: 'pancake' }))}>
          <InteractivePermissionTreeView
            permissions={[
              {
                id: 'resources',
                name: 'Resource Permissions',
                type: 'resource',
                granted: true,
                inherited: false,
                riskLevel: 'low',
                category: 'General',
                children: [
                  {
                    id: 'dashboard',
                    name: 'Dashboard',
                    type: 'resource',
                    granted: true,
                    inherited: false,
                    riskLevel: 'low',
                    category: 'UI',
                    children: [
                      {
                        id: 'dashboard.read',
                        name: 'Read Dashboard',
                        type: 'permission',
                        granted: true,
                        inherited: false,
                        riskLevel: 'low',
                        category: 'Read',
                        description: 'View dashboard data'
                      },
                      {
                        id: 'dashboard.edit',
                        name: 'Edit Dashboard',
                        type: 'permission',
                        granted: false,
                        inherited: false,
                        riskLevel: 'medium',
                        category: 'Write',
                        description: 'Modify dashboard configuration'
                      }
                    ]
                  },
                  {
                    id: 'users',
                    name: 'User Management',
                    type: 'resource',
                    granted: false,
                    inherited: true,
                    source: 'admin role',
                    riskLevel: 'high',
                    category: 'Admin',
                    children: [
                      {
                        id: 'users.read',
                        name: 'View Users',
                        type: 'permission',
                        granted: true,
                        inherited: true,
                        source: 'admin role',
                        riskLevel: 'medium',
                        category: 'Read'
                      },
                      {
                        id: 'users.create',
                        name: 'Create Users',
                        type: 'permission',
                        granted: false,
                        inherited: false,
                        riskLevel: 'high',
                        category: 'Write'
                      }
                    ]
                  }
                ]
              }
            ]}
            onPermissionChange={(nodeId, granted) => {
              console.log('Permission change:', nodeId, granted);
              handleRefreshData();
            }}
            onBulkPermissionChange={(nodeIds, granted) => {
              console.log('Bulk permission change:', nodeIds, granted);
              handleRefreshData();
            }}
            readonly={false}
            showSearch={true}
            showBulkActions={true}
            highlightChanges={true}
          />
        </div>
      )}

      {/* Drag & Drop Bulk Assignment */}
      {canManagePermissions && (
        <div className={cn(adminCardVariants({ variant: 'pancake' }))}>
          <DragDropBulkAssignment
            users={[
              { id: user.id, type: 'user', name: user.firstName + ' ' + user.lastName, email: user.email }
            ]}
            profiles={user.permissionProfiles.map(p => ({
              id: p.id,
              type: 'profile' as const,
              name: p.name,
              description: p.description,
              riskLevel: 'medium' as const,
              category: p.category
            }))}
            permissions={user.customPermissions.map(p => ({
              id: p.id,
              type: 'permission' as const,
              name: p.permission,
              description: p.resource + ' - ' + p.action,
              riskLevel: 'low' as const,
              category: 'Custom'
            }))}
            onBulkAssign={async (assignments) => {
              console.log('Bulk assign:', assignments);
              // Process assignments
              handleRefreshData();
            }}
          />
        </div>
      )}

      {/* Advanced Management Tools */}
      {canManagePermissions && (
        <>
          {/* Permission Validation & Conflict Resolution */}
          <div className={cn(adminCardVariants({ variant: 'pancake' }))}>
            <PermissionConflictResolver
              userId={user.id}
              proposedChanges={{
                addPermissions: user.customPermissions.map(p => p.permission),
                addRoles: user.roles.map(r => r.name),
                addProfiles: user.permissionProfiles.map(p => p.id),
              }}
              onValidationComplete={(result) => {
                // Handle validation completion if needed
                console.log('Validation completed:', result);
              }}
              autoValidate={false}
            />
          </div>
          
          {/* Export/Import */}
          <div className={cn(adminCardVariants({ variant: 'pancake' }))}>
            <h3 className="text-lg font-semibold mb-4 flex items-center gap-2">
              <Settings className="h-5 w-5" />
              Backup & Migration
            </h3>
            
            <PermissionExportImport
              user={user}
              onPermissionsUpdated={handleRefreshData}
            />
          </div>
        </>
      )}

    </div>
  )
}