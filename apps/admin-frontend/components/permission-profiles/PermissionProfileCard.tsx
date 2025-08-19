/**
 * Permission Profile Card - Display individual permission profiles
 */

'use client'

import { useState } from 'react'
import { Settings, Users, Shield, Key, MoreVertical, Edit, Trash2, UserPlus } from 'lucide-react'
import { Button } from '@/components/ui/button'
import { Badge } from '@/components/ui/badge'
import { 
  DropdownMenu, 
  DropdownMenuContent, 
  DropdownMenuItem, 
  DropdownMenuSeparator, 
  DropdownMenuTrigger 
} from '@/components/ui/dropdown-menu'
import { formatDistanceToNow } from 'date-fns'
import type { PermissionProfile } from '@/lib/types/permission-profiles'

interface PermissionProfileCardProps {
  profile: PermissionProfile
  onEdit: () => void
  onDelete: () => void
  onAssign?: () => void
}

export function PermissionProfileCard({ 
  profile, 
  onEdit, 
  onDelete,
  onAssign 
}: PermissionProfileCardProps) {
  const [showDetails, setShowDetails] = useState(false)

  const getCategoryColor = (category: string) => {
    const colors = {
      user: 'bg-blue-100 text-blue-800 border-blue-200',
      moderator: 'bg-yellow-100 text-yellow-800 border-yellow-200',
      admin: 'bg-red-100 text-red-800 border-red-200',
      custom: 'bg-purple-100 text-purple-800 border-purple-200',
      system: 'bg-gray-100 text-gray-800 border-gray-200',
      business: 'bg-green-100 text-green-800 border-green-200',
      technical: 'bg-indigo-100 text-indigo-800 border-indigo-200',
      administrative: 'bg-orange-100 text-orange-800 border-orange-200',
      compliance: 'bg-pink-100 text-pink-800 border-pink-200'
    }
    return colors[category as keyof typeof colors] || colors.custom
  }

  const getTierColor = (tier: string) => {
    const colors = {
      free: 'text-gray-600',
      bronze: 'text-amber-600',
      silver: 'text-gray-500',
      gold: 'text-yellow-500',
      platinum: 'text-purple-600',
      admin: 'text-red-600',
      superadmin: 'text-red-800'
    }
    return colors[tier as keyof typeof colors] || colors.free
  }

  const formatCategoryName = (category: string) => {
    return category.charAt(0).toUpperCase() + category.slice(1)
  }

  const formatTierName = (tier: string) => {
    return tier.charAt(0).toUpperCase() + tier.slice(1)
  }

  return (
    <div className="pancake-card p-6 hover:shadow-md transition-shadow">
      {/* Header */}
      <div className="flex items-start justify-between mb-4">
        <div className="flex-1">
          <div className="flex items-center gap-2 mb-2">
            <h3 className="font-semibold text-lg">{profile.name}</h3>
            <Badge 
              variant="outline" 
              className={getCategoryColor(profile.category)}
            >
              {formatCategoryName(profile.category)}
            </Badge>
          </div>
          
          <p className="text-sm text-muted-foreground line-clamp-2">
            {profile.description}
          </p>
        </div>

        <DropdownMenu>
          <DropdownMenuTrigger asChild>
            <Button variant="ghost" size="sm">
              <MoreVertical className="h-4 w-4" />
            </Button>
          </DropdownMenuTrigger>
          <DropdownMenuContent align="end">
            <DropdownMenuItem onClick={onEdit}>
              <Edit className="h-4 w-4 mr-2" />
              Edit Profile
            </DropdownMenuItem>
            {onAssign && (
              <DropdownMenuItem onClick={onAssign}>
                <UserPlus className="h-4 w-4 mr-2" />
                Assign to Users
              </DropdownMenuItem>
            )}
            <DropdownMenuSeparator />
            <DropdownMenuItem 
              onClick={onDelete}
              className="text-red-600 hover:text-red-700"
            >
              <Trash2 className="h-4 w-4 mr-2" />
              Delete Profile
            </DropdownMenuItem>
          </DropdownMenuContent>
        </DropdownMenu>
      </div>

      {/* Stats */}
      <div className="grid grid-cols-3 gap-4 mb-4">
        <div className="text-center">
          <div className="flex items-center justify-center mb-1">
            <Key className="h-4 w-4 text-green-500" />
          </div>
          <p className="text-sm font-medium">{profile.permissions.length}</p>
          <p className="text-xs text-muted-foreground">Permissions</p>
        </div>
        
        <div className="text-center">
          <div className="flex items-center justify-center mb-1">
            <Users className="h-4 w-4 text-blue-500" />
          </div>
          <p className="text-sm font-medium">{profile.assignmentCount || 0}</p>
          <p className="text-xs text-muted-foreground">Assignments</p>
        </div>
        
        <div className="text-center">
          <div className="flex items-center justify-center mb-1">
            <Shield className={`h-4 w-4 ${getTierColor(profile.targetTier)}`} />
          </div>
          <p className={`text-sm font-medium ${getTierColor(profile.targetTier)}`}>
            {formatTierName(profile.targetTier)}
          </p>
          <p className="text-xs text-muted-foreground">Tier</p>
        </div>
      </div>

      {/* Permissions Preview */}
      <div className="mb-4">
        <Button
          variant="ghost"
          size="sm"
          onClick={() => setShowDetails(!showDetails)}
          className="w-full justify-start text-sm"
        >
          <Settings className="h-4 w-4 mr-2" />
          {showDetails ? 'Hide' : 'Show'} Permissions ({profile.permissions.length})
        </Button>
        
        {showDetails && (
          <div className="mt-2 space-y-1 max-h-32 overflow-y-auto">
            {profile.permissions.map((permission, index) => (
              <div 
                key={index}
                className="text-xs bg-gray-50 rounded px-2 py-1 flex justify-between"
              >
                <span className="font-mono text-green-700">{permission.action}</span>
                <span className="text-muted-foreground">{permission.resource}</span>
              </div>
            ))}
            {profile.permissions.length === 0 && (
              <p className="text-xs text-muted-foreground italic">No permissions defined</p>
            )}
          </div>
        )}
      </div>

      {/* Footer */}
      <div className="pt-4 border-t border-gray-100">
        <div className="flex items-center justify-between text-xs text-muted-foreground">
          <span>
            Created {formatDistanceToNow(new Date(profile.createdAt), { addSuffix: true })}
          </span>
          <span className={profile.isActive ? 'text-green-600' : 'text-red-600'}>
            {profile.isActive ? 'Active' : 'Inactive'}
          </span>
        </div>
      </div>
    </div>
  )
}