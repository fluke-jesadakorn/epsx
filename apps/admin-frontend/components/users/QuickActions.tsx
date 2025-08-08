/**
 * Quick Actions Component
 * Provides quick action buttons for user management
 */

'use client'

import { useState } from 'react'
import { MoreHorizontal, Power, Mail, Key, Trash2 } from 'lucide-react'
import { EditProfileButton } from './EditProfileButton'
import type { UnifiedUserData } from '@/lib/types/unified-user'
import type { EnhancedAuthUser } from '@/lib/auth/server-auth-enhanced'

interface QuickActionsProps {
  user: UnifiedUserData
  currentUser: EnhancedAuthUser
}

export function QuickActions({ user, currentUser }: QuickActionsProps) {
  const [isLoading, setIsLoading] = useState(false)

  // Quick action handlers
  const handleToggleStatus = async () => {
    setIsLoading(true)
    try {
      // In real implementation, this would call a Server Action
      console.log('Toggle status for user:', user.id)
    } catch (error) {
      console.error('Failed to toggle status:', error)
    } finally {
      setIsLoading(false)
    }
  }

  const handleSendEmail = async () => {
    // Open mailto link or show email modal
    window.location.href = `mailto:${user.email}`
  }

  const canModifyUser = currentUser.isSuperAdmin || currentUser.canManageUsers

  return (
    <div className="flex items-center gap-2">
      {/* Edit Profile Button */}
      {canModifyUser && (
        <EditProfileButton 
          userId={user.id}
          disabled={isLoading}
        />
      )}

      {/* Quick Actions Dropdown */}
      <div className="relative">
        <button
          className="inline-flex items-center gap-1 px-3 py-2 text-sm border border-muted-foreground rounded-lg hover:bg-muted transition-colors"
          disabled={isLoading}
        >
          <MoreHorizontal className="h-4 w-4" />
          <span className="sr-only">More actions</span>
        </button>
        
        {/* Dropdown menu would go here */}
        {/* For now, showing as disabled buttons */}
      </div>

      {/* Individual Action Buttons (for demo) */}
      <div className="hidden lg:flex items-center gap-2">
        <button
          onClick={handleSendEmail}
          className="p-2 text-muted-foreground hover:text-foreground hover:bg-muted rounded-lg transition-colors"
          title="Send email"
        >
          <Mail className="h-4 w-4" />
        </button>

        {canModifyUser && (
          <>
            <button
              onClick={handleToggleStatus}
              disabled={isLoading}
              className={`p-2 rounded-lg transition-colors ${
                user.status === 'active' 
                  ? 'text-red-600 hover:text-red-700 hover:bg-red-50' 
                  : 'text-green-600 hover:text-green-700 hover:bg-green-50'
              }`}
              title={user.status === 'active' ? 'Disable user' : 'Enable user'}
            >
              <Power className="h-4 w-4" />
            </button>

            <button
              className="p-2 text-blue-600 hover:text-blue-700 hover:bg-blue-50 rounded-lg transition-colors"
              title="Reset API keys"
            >
              <Key className="h-4 w-4" />
            </button>

            {currentUser.isSuperAdmin && (
              <button
                className="p-2 text-red-600 hover:text-red-700 hover:bg-red-50 rounded-lg transition-colors"
                title="Delete user"
              >
                <Trash2 className="h-4 w-4" />
              </button>
            )}
          </>
        )}
      </div>

    </div>
  )
}