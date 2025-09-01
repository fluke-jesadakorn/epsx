/**
 * Bulk Grant Permissions Page - Server Component
 * Form for granting permissions to multiple users with Windows Phone tiles
 */

import { UserPlus, Check, AlertTriangle, ArrowLeft, Shield, Activity, BarChart3, Settings, Zap } from 'lucide-react'
import { AdminServerAPI } from '@/lib/server/admin-api'
import { revalidatePath } from 'next/cache'
import { redirect } from 'next/navigation'
import { Textarea } from '@/components/ui/textarea'
import { Button } from '@/components/ui/button'
import { adminCardVariants, cn } from '@/design-system'

interface Props {
  searchParams?: Promise<{
    users?: string
  }>
}

// Server action for bulk granting permissions
async function bulkGrantPermissionsAction(formData: FormData) {
  'use server'
  
  const userIds = formData.get('userIds')?.toString().split(',') || []
  const selectedPermissions = formData.getAll('permissions') as string[]
  const reason = formData.get('reason')?.toString() || ''
  
  if (userIds.length === 0 || selectedPermissions.length === 0) {
    redirect('/users/bulk/permissions/grant?error=invalid-data')
  }
  
  try {
    await AdminServerAPI.bulkGrantPermissions(userIds, selectedPermissions)
    revalidatePath('/users')
    redirect(`/users?success=bulk-permissions-granted&count=${userIds.length}`)
  } catch (error) {
    console.error('Failed to grant permissions:', error)
    redirect('/users/bulk/permissions/grant?error=operation-failed')
  }
}

export default async function BulkGrantPermissionsPage({ searchParams }: Props) {
  const resolvedSearchParams = await searchParams
  const selectedUserIds = resolvedSearchParams?.users?.split(',').filter(Boolean) || []
  
  if (selectedUserIds.length === 0) {
    redirect('/users?error=no-users-selected')
  }

  // Predefined permission options organized by category with Windows Phone styling
  const permissionCategories = [
    {
      name: 'analytics & data',
      icon: BarChart3,
      color: 'from-blue-600/90 to-blue-700/90',
      variant: 'analytics' as const,
      permissions: [
        { id: 'epsx:analytics:view', name: 'view analytics', description: 'access analytics dashboard and data' },
        { id: 'epsx:analytics:export', name: 'export analytics', description: 'download analytics reports' },
        { id: 'epsx:realtime:access', name: 'real-time data', description: 'access live market data feeds' }
      ]
    },
    {
      name: 'user management',
      icon: UserPlus,
      color: 'from-green-600/90 to-green-700/90',
      variant: 'user' as const,
      permissions: [
        { id: 'admin:users:view', name: 'view users', description: 'access user management interface' },
        { id: 'admin:users:manage', name: 'manage users', description: 'create, edit, and delete users' },
        { id: 'admin:permissions:manage', name: 'manage permissions', description: 'grant and revoke user permissions' }
      ]
    },
    {
      name: 'system administration',
      icon: Settings,
      color: 'from-purple-600/90 to-purple-700/90',
      variant: 'permission' as const,
      permissions: [
        { id: 'admin:system:configure', name: 'system config', description: 'access system settings and configuration' },
        { id: 'admin:audit:view', name: 'audit logs', description: 'access system audit trails' },
        { id: 'admin:notifications:manage', name: 'notifications', description: 'send and manage system notifications' }
      ]
    },
    {
      name: 'premium features',
      icon: Zap,
      color: 'from-yellow-600/90 to-yellow-700/90',
      variant: 'billing' as const,
      permissions: [
        { id: 'epsx:premium:access', name: 'premium access', description: 'access premium features and tools' },
        { id: 'epsx:api:unlimited', name: 'unlimited api', description: 'bypass api rate limits' },
        { id: 'epsx:support:priority', name: 'priority support', description: 'access to priority customer support' }
      ]
    }
  ]

  return (
    <div className="p-6">
      {/* Windows Phone Header */}
      <div className="mb-8">
        <div className="flex items-center gap-4 mb-4">
          <div className="w-14 h-14 bg-gradient-to-br from-green-600/20 to-green-700/30 border border-green-400/30 flex items-center justify-center">
            <UserPlus className="h-6 w-6 text-green-300" />
            <div className="absolute -bottom-0.5 -right-0.5 w-3 h-3 bg-yellow-400/80" />
          </div>
          <div>
            <h2 className="text-2xl font-light text-foreground uppercase tracking-wide">
              grant permissions
            </h2>
            <p className="text-muted-foreground font-light">
              add permissions to {selectedUserIds.length} selected user{selectedUserIds.length !== 1 ? 's' : ''}
            </p>
          </div>
        </div>
        
        {/* Progress indicator */}
        <div className="flex items-center gap-2">
          <div className="flex-1 h-1 bg-muted rounded-full overflow-hidden">
            <div className="h-full w-1/3 bg-gradient-to-r from-green-400 to-green-600 transition-all duration-1000 ease-out" />
          </div>
          <span className="text-xs text-muted-foreground font-light uppercase tracking-wider">selecting</span>
        </div>
      </div>

      {/* Form */}
      <form action={bulkGrantPermissionsAction} className="space-y-8">
        <input type="hidden" name="userIds" value={selectedUserIds.join(',')} />
        
        {/* Permission Categories - Windows Phone Tile Sections */}
        {permissionCategories.map((category, categoryIndex) => {
          const CategoryIcon = category.icon
          
          return (
            <div key={category.name} className="space-y-4">
              {/* Category Header Tile */}
              <div className={cn(
                adminCardVariants({ 
                  variant: category.variant,
                  hover: 'glow',
                  size: 'compact'
                }),
                'relative overflow-hidden wp-scale-hover',
                `stagger-${categoryIndex + 1}`
              )}>
                <div className="absolute left-0 top-0 bottom-0 w-1 bg-yellow-400/80" />
                <div className="absolute top-2 right-2 w-1.5 h-1.5 bg-yellow-400/80 rounded-full animate-pulse-subtle" />
                
                <div className="relative z-10 p-4">
                  <div className="flex items-center gap-3">
                    <div className="w-10 h-10 bg-white/10 border border-white/20 flex items-center justify-center">
                      <CategoryIcon className="h-5 w-5 text-white/90" />
                      <div className="absolute -bottom-0.5 -right-0.5 w-2 h-2 bg-yellow-400/80" />
                    </div>
                    <div>
                      <h3 className="text-lg font-light text-white uppercase tracking-wide">
                        {category.name}
                      </h3>
                      <p className="text-xs text-white/70 font-light uppercase tracking-wider">
                        {category.permissions.length} permissions
                      </p>
                    </div>
                  </div>
                </div>
              </div>
              
              {/* Permission Selection Tiles */}
              <div className="grid grid-cols-1 md:grid-cols-2 gap-3">
                {category.permissions.map((permission, permIndex) => (
                  <label
                    key={permission.id}
                    className={cn(
                      adminCardVariants({ variant: 'default', hover: 'both', size: 'compact' }),
                      'cursor-pointer relative overflow-hidden wp-scale-hover group/perm',
                      `stagger-${permIndex + 2}`
                    )}
                  >
                    {/* Selection indicator */}
                    <div className="absolute left-0 top-0 bottom-0 w-1 bg-muted group-hover/perm:bg-green-400/60 transition-colors duration-300" />
                    
                    <div className="relative z-10 p-4">
                      <div className="flex items-start gap-3">
                        <div className="mt-1">
                          <input
                            type="checkbox"
                            name="permissions"
                            value={permission.id}
                            className="h-4 w-4 text-green-600 focus:ring-green-500 border-muted bg-transparent rounded-sm"
                          />
                        </div>
                        <div className="flex-1">
                          <div className="font-light text-foreground group-hover/perm:text-green-400 transition-colors uppercase tracking-wider text-sm">
                            {permission.name}
                          </div>
                          <div className="text-xs text-muted-foreground mt-1 font-light leading-relaxed">
                            {permission.description}
                          </div>
                          <div className="text-xs text-muted-foreground mt-2 font-mono opacity-60">
                            {permission.id}
                          </div>
                        </div>
                      </div>
                    </div>
                    
                    {/* Hover accent dot */}
                    <div className="absolute top-2 right-2 w-1.5 h-1.5 bg-transparent group-hover/perm:bg-green-400/60 rounded-full transition-all duration-300" />
                  </label>
                ))}
              </div>
            </div>
          )
        })}

        {/* Reason Field - Windows Phone Input Tile */}
        <div className={cn(
          adminCardVariants({ variant: 'default', hover: 'both', size: 'default' }),
          'relative overflow-hidden wp-scale-hover p-6'
        )}>
          <div className="absolute left-0 top-0 bottom-0 w-1 bg-blue-400/80" />
          <div className="absolute top-3 right-3 w-2 h-2 bg-blue-400/60 rounded-full animate-pulse-subtle" />
          
          <div className="relative z-10 space-y-3">
            <label htmlFor="reason" className="block text-sm font-light text-foreground uppercase tracking-wider">
              reason for grant (optional)
            </label>
            <Textarea
              id="reason"
              name="reason"
              rows={3}
              variant="wp"
              size="default"
              placeholder="explain why these permissions are being granted..."
              className="bg-transparent border-muted/50 focus:border-blue-400/50"
            />
          </div>
        </div>

        {/* Warning Tile */}
        <div className={cn(
          adminCardVariants({ variant: 'warning', hover: 'glow', size: 'default' }),
          'relative overflow-hidden wp-scale-hover'
        )}>
          <div className="absolute left-0 top-0 bottom-0 w-1 bg-amber-400/80" />
          <div className="absolute top-3 right-3 w-2 h-2 bg-red-400/80 rounded-full animate-pulse" />
          
          <div className="relative z-10 p-6">
            <div className="flex items-start gap-4">
              <div className="w-12 h-12 bg-amber-600/20 border border-amber-400/30 flex items-center justify-center">
                <AlertTriangle className="h-6 w-6 text-amber-300" />
                <div className="absolute -bottom-0.5 -right-0.5 w-3 h-3 bg-yellow-400/80" />
              </div>
              
              <div className="flex-1">
                <h4 className="font-light text-lg text-foreground mb-2 uppercase tracking-wide">
                  important notice
                </h4>
                <p className="text-sm text-muted-foreground leading-relaxed font-light">
                  these permissions will be added to all {selectedUserIds.length} selected users. 
                  existing permissions will be preserved. this action will be logged for audit purposes.
                </p>
                
                <div className="mt-4 flex items-center gap-2">
                  <div className="w-2 h-2 bg-amber-400/60 rounded-full animate-pulse-subtle" />
                  <span className="text-xs text-muted-foreground font-light uppercase tracking-wider">
                    {selectedUserIds.length} users affected
                  </span>
                </div>
              </div>
            </div>
          </div>
        </div>

        {/* Action Buttons - Windows Phone Style */}
        <div className="flex gap-4 justify-end pt-6">
          <a
            href={`/users/bulk?users=${selectedUserIds.join(',')}`}
            className={cn(
              adminCardVariants({ variant: 'default', hover: 'both', size: 'compact' }),
              'inline-flex items-center gap-2 px-6 py-3 wp-scale-hover relative overflow-hidden'
            )}
          >
            <div className="absolute left-0 top-0 bottom-0 w-1 bg-gray-400/80" />
            <ArrowLeft className="h-4 w-4 text-muted-foreground" />
            <span className="font-light uppercase tracking-wider text-sm">back to operations</span>
          </a>
          
          <Button
            type="submit"
            variant="pancake"
            size="lg"
            className="relative overflow-hidden"
          >
            <div className="absolute left-0 top-0 bottom-0 w-1 bg-green-400/80" />
            <Check className="h-4 w-4" />
            <span className="font-light uppercase tracking-wider">grant permissions</span>
          </Button>
        </div>
      </form>
    </div>
  )
}