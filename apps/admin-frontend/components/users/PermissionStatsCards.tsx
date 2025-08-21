/**
 * Permission Stats Cards - Pure Server Component
 * Displays permission statistics without client-side state
 */

import { Shield, Users, Key } from 'lucide-react'
import { adminCardVariants, cn } from '@/design-system'

interface PermissionStatsCardsProps {
  activeRoles: number
  totalPermissions: number
  activeProfiles: number
}

export function PermissionStatsCards({ 
  activeRoles, 
  totalPermissions, 
  activeProfiles 
}: PermissionStatsCardsProps) {
  return (
    <div className="grid grid-cols-1 md:grid-cols-3 gap-4">
      <div className={cn(adminCardVariants({ variant: 'pancake' }))}>
        <div className="flex items-center justify-between">
          <div>
            <p className="text-2xl font-bold text-info-600">{activeRoles}</p>
            <p className="text-sm text-muted-foreground">Active Roles</p>
          </div>
          <div className="p-2 bg-info-100 rounded-lg">
            <Users className="h-8 w-8 text-info-500" />
          </div>
        </div>
      </div>
      
      <div className={cn(adminCardVariants({ variant: 'pancake' }))}>
        <div className="flex items-center justify-between">
          <div>
            <p className="text-2xl font-bold text-success-600">{totalPermissions}</p>
            <p className="text-sm text-muted-foreground">Custom Permissions</p>
          </div>
          <div className="p-2 bg-success-100 rounded-lg">
            <Key className="h-8 w-8 text-success-500" />
          </div>
        </div>
      </div>

      <div className={cn(adminCardVariants({ variant: 'pancake' }))}>
        <div className="flex items-center justify-between">
          <div>
            <p className="text-2xl font-bold text-primary-600">{activeProfiles}</p>
            <p className="text-sm text-muted-foreground">Permission Profiles</p>
          </div>
          <div className="p-2 bg-primary-100 rounded-lg">
            <Shield className="h-8 w-8 text-primary-500" />
          </div>
        </div>
      </div>
    </div>
  )
}