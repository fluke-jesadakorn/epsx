/**
 * Bulk Operations Main Page - Server Component
 * Shows available bulk operations for selected users with Windows Phone large tiles
 */

import { Shield, UserPlus, UserMinus, UserCheck, ArrowRight, Zap, Activity, BarChart3, Settings } from 'lucide-react'
import { adminCardVariants, cn } from '@/design-system'

interface Props {
  searchParams?: Promise<{
    users?: string
  }>
}

export default async function BulkOperationsPage({ searchParams }: Props) {
  const resolvedSearchParams = await searchParams
  const selectedUserIds = resolvedSearchParams?.users?.split(',').filter(Boolean) || []
  const userParams = `users=${selectedUserIds.join(',')}`

  const operations = [
    {
      title: 'grant perms',
      subtitle: 'add permissions',
      description: 'Add new permissions to selected users. Existing permissions preserved.',
      icon: UserPlus,
      href: `/users/bulk/permissions/grant?${userParams}`,
      variant: 'user' as const,
      tileColor: 'from-green-600/90 to-green-700/90',
      progress: 0, // Could be dynamic based on current operations
      category: 'permissions'
    },
    {
      title: 'revoke perms',
      subtitle: 'remove access',
      description: 'Remove specific permissions from selected users safely.',
      icon: UserMinus,
      href: `/users/bulk/permissions/revoke?${userParams}`,
      variant: 'error' as const,
      tileColor: 'from-red-600/90 to-red-700/90',
      progress: 0,
      category: 'permissions'
    },
    {
      title: 'assign roles',
      subtitle: 'role management',
      description: 'Assign predefined roles with comprehensive permission sets.',
      icon: Shield,
      href: `/users/bulk/roles/assign?${userParams}`,
      variant: 'permission' as const,
      tileColor: 'from-blue-600/90 to-blue-700/90',
      progress: 0,
      category: 'administration'
    },
    {
      title: 'validate access',
      subtitle: 'audit permissions',
      description: 'Check permission consistency and validity across users.',
      icon: UserCheck,
      href: `/users/bulk/validate?${userParams}`,
      variant: 'analytics' as const,
      tileColor: 'from-purple-600/90 to-purple-700/90',
      progress: 0,
      category: 'audit'
    }
  ]

  return (
    <div className="p-6">
      {/* Windows Phone Header */}
      <div className="mb-8">
        <h2 className="text-2xl font-light text-foreground mb-2 tracking-wide uppercase">
          choose bulk operation
        </h2>
        <p className="text-muted-foreground font-light">
          perform operation on {selectedUserIds.length} selected user{selectedUserIds.length !== 1 ? 's' : ''}
        </p>
        
        {/* Progress indicator for bulk operations */}
        <div className="mt-4 flex items-center gap-2">
          <div className="flex-1 h-1 bg-muted rounded-full overflow-hidden">
            <div className="h-full w-0 bg-gradient-to-r from-yellow-400 to-yellow-600 transition-all duration-1000 ease-out" />
          </div>
          <span className="text-xs text-muted-foreground font-light uppercase tracking-wider">ready</span>
        </div>
      </div>

      {/* Windows Phone Large Tiles Grid */}
      <div className="grid grid-cols-1 lg:grid-cols-2 gap-4 mb-8">
        {operations.map((operation, index) => {
          const IconComponent = operation.icon
          
          return (
            <a
              key={operation.title}
              href={operation.href}
              className={cn(
                adminCardVariants({ 
                  variant: operation.variant,
                  hover: 'intense',
                  animation: 'subtle',
                  size: 'large'
                }),
                'group/tile relative overflow-hidden min-h-[160px] wp-intense-hover',
                `stagger-${index + 1}`
              )}
            >
              {/* Windows Phone accent bar */}
              <div className="absolute left-0 top-0 bottom-0 w-1 bg-yellow-400/80" />
              
              {/* Live tile status dot */}
              <div className="absolute top-3 right-3 w-2 h-2 bg-yellow-400/80 rounded-full animate-pulse-subtle" />
              
              {/* Tile content */}
              <div className="relative z-10 p-6 h-full flex flex-col">
                {/* Icon section */}
                <div className="flex items-start justify-between mb-4">
                  <div className={cn(
                    'w-12 h-12 flex items-center justify-center border border-white/20 relative',
                    'group-hover/tile:scale-110 transition-all duration-300'
                  )}>
                    <IconComponent className="h-6 w-6 text-white" />
                    <div className="absolute -bottom-1 -right-1 w-3 h-3 bg-yellow-400/80" />
                  </div>
                  
                  <ArrowRight className="h-5 w-5 text-white/60 group-hover/tile:text-yellow-300 group-hover/tile:translate-x-1 transition-all duration-300" />
                </div>
                
                {/* Text content */}
                <div className="flex-1">
                  <div className="mb-2">
                    <h3 className="text-xl font-light text-white uppercase tracking-wide mb-1">
                      {operation.title}
                    </h3>
                    <p className="text-sm text-white/80 font-light uppercase tracking-wider">
                      {operation.subtitle}
                    </p>
                  </div>
                  
                  <p className="text-sm text-white/70 leading-relaxed font-light mb-3">
                    {operation.description}
                  </p>
                </div>
                
                {/* Bottom stats */}
                <div className="flex items-center justify-between text-xs text-white/60">
                  <div className="flex items-center gap-1">
                    <Activity className="h-3 w-3" />
                    <span className="font-light uppercase tracking-wider">{operation.category}</span>
                  </div>
                  <div className="flex items-center gap-1">
                    <Zap className="h-3 w-3" />
                    <span className="font-light">{selectedUserIds.length}</span>
                  </div>
                </div>
                
                {/* Progress indicator */}
                {operation.progress > 0 && (
                  <div className="absolute bottom-0 left-0 right-0 h-0.5">
                    <div 
                      className="h-full bg-gradient-to-r from-yellow-400 to-yellow-600 transition-all duration-1000"
                      style={{ width: `${operation.progress}%` }}
                    />
                  </div>
                )}
              </div>
            </a>
          )
        })}
      </div>

      {/* Advanced Options - Windows Phone Mini Tiles */}
      <div className="mb-8">
        <h3 className="text-lg font-light text-foreground mb-4 uppercase tracking-wide">
          advanced options
        </h3>
        
        <div className="grid grid-cols-1 md:grid-cols-3 gap-3">
          <a
            href={`/users/bulk/export?${userParams}`}
            className={cn(
              adminCardVariants({ variant: 'billing', hover: 'both', size: 'compact' }),
              'group/mini-tile wp-scale-hover relative overflow-hidden min-h-[80px] stagger-5'
            )}
          >
            <div className="absolute left-0 top-0 bottom-0 w-1 bg-indigo-400/80" />
            <div className="absolute top-2 right-2 w-1.5 h-1.5 bg-yellow-400/80 rounded-full animate-pulse-subtle" />
            
            <div className="relative z-10 p-4 h-full flex items-center gap-3">
              <div className="w-10 h-10 bg-indigo-600/20 border border-indigo-400/30 flex items-center justify-center">
                <BarChart3 className="h-5 w-5 text-indigo-300" />
                <div className="absolute -bottom-0.5 -right-0.5 w-2 h-2 bg-yellow-400/80" />
              </div>
              <div className="flex-1">
                <p className="font-light text-sm text-foreground uppercase tracking-wider">export data</p>
                <p className="text-xs text-muted-foreground font-light">download user info</p>
              </div>
              <ArrowRight className="h-3 w-3 text-muted-foreground group-hover/mini-tile:text-yellow-300 group-hover/mini-tile:translate-x-0.5 transition-all duration-300" />
            </div>
          </a>
          
          <a
            href={`/users/bulk/audit?${userParams}`}
            className={cn(
              adminCardVariants({ variant: 'analytics', hover: 'both', size: 'compact' }),
              'group/mini-tile wp-scale-hover relative overflow-hidden min-h-[80px] stagger-6'
            )}
          >
            <div className="absolute left-0 top-0 bottom-0 w-1 bg-orange-400/80" />
            <div className="absolute top-2 right-2 w-1.5 h-1.5 bg-yellow-400/80 rounded-full animate-pulse-subtle" />
            
            <div className="relative z-10 p-4 h-full flex items-center gap-3">
              <div className="w-10 h-10 bg-orange-600/20 border border-orange-400/30 flex items-center justify-center">
                <Activity className="h-5 w-5 text-orange-300" />
                <div className="absolute -bottom-0.5 -right-0.5 w-2 h-2 bg-yellow-400/80" />
              </div>
              <div className="flex-1">
                <p className="font-light text-sm text-foreground uppercase tracking-wider">audit trail</p>
                <p className="text-xs text-muted-foreground font-light">operation history</p>
              </div>
              <ArrowRight className="h-3 w-3 text-muted-foreground group-hover/mini-tile:text-yellow-300 group-hover/mini-tile:translate-x-0.5 transition-all duration-300" />
            </div>
          </a>
          
          <a
            href={`/users/bulk/schedule?${userParams}`}
            className={cn(
              adminCardVariants({ variant: 'permission', hover: 'both', size: 'compact' }),
              'group/mini-tile wp-scale-hover relative overflow-hidden min-h-[80px] stagger-1'
            )}
          >
            <div className="absolute left-0 top-0 bottom-0 w-1 bg-teal-400/80" />
            <div className="absolute top-2 right-2 w-1.5 h-1.5 bg-yellow-400/80 rounded-full animate-pulse-subtle" />
            
            <div className="relative z-10 p-4 h-full flex items-center gap-3">
              <div className="w-10 h-10 bg-teal-600/20 border border-teal-400/30 flex items-center justify-center">
                <Settings className="h-5 w-5 text-teal-300" />
                <div className="absolute -bottom-0.5 -right-0.5 w-2 h-2 bg-yellow-400/80" />
              </div>
              <div className="flex-1">
                <p className="font-light text-sm text-foreground uppercase tracking-wider">schedule ops</p>
                <p className="text-xs text-muted-foreground font-light">delay execution</p>
              </div>
              <ArrowRight className="h-3 w-3 text-muted-foreground group-hover/mini-tile:text-yellow-300 group-hover/mini-tile:translate-x-0.5 transition-all duration-300" />
            </div>
          </a>
        </div>
      </div>

      {/* Safety Notice - Windows Phone Warning Tile */}
      <div className={cn(
        adminCardVariants({ variant: 'warning', hover: 'glow', size: 'default' }),
        'relative overflow-hidden wp-scale-hover'
      )}>
        <div className="absolute left-0 top-0 bottom-0 w-1 bg-amber-400/80" />
        <div className="absolute top-3 right-3 w-2 h-2 bg-red-400/80 rounded-full animate-pulse" />
        
        <div className="relative z-10 p-6">
          <div className="flex items-start gap-4">
            <div className="w-12 h-12 bg-amber-600/20 border border-amber-400/30 flex items-center justify-center">
              <Shield className="h-6 w-6 text-amber-300" />
              <div className="absolute -bottom-0.5 -right-0.5 w-3 h-3 bg-yellow-400/80" />
            </div>
            
            <div className="flex-1">
              <h4 className="font-light text-lg text-foreground mb-2 uppercase tracking-wide">
                safety information
              </h4>
              <p className="text-sm text-muted-foreground leading-relaxed font-light">
                bulk operations affect multiple users simultaneously and cannot be undone easily. 
                please review your selections carefully before proceeding. all operations are logged 
                for audit purposes.
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
    </div>
  )
}