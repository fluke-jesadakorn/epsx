'use client'

import { useState } from 'react'
import { ChevronDownIcon, CheckIcon, CogIcon, ShieldCheckIcon } from '@heroicons/react/24/outline'
import { useAuth } from '@/lib/auth'
import { getPlatformDisplayName, getPlatformIcon } from '@/lib/auth'

interface AdminPlatformSwitcherProps {
  className?: string
  showIcon?: boolean
  showLabel?: boolean
  showAdminBadge?: boolean
}

export default function AdminPlatformSwitcher({
  className = '',
  showIcon = true,
  showLabel = true,
  showAdminBadge = true,
}: AdminPlatformSwitcherProps) {
  const [isOpen, setIsOpen] = useState(false)
  const [isLoading, setIsLoading] = useState(false)
  
  const {
    getCurrentPlatform,
    getAvailablePlatforms,
    canAccessPlatform,
    switchPlatform,
    error,
    canManagePlatforms,
  } = useAuth.getState()
  
  const currentPlatform = getCurrentPlatform()
  const availablePlatforms = getAvailablePlatforms()
  const canManage = canManagePlatforms()
  
  const handlePlatformSwitch = async (platform: string) => {
    if (platform === currentPlatform) {
      setIsOpen(false)
      return
    }
    
    setIsLoading(true)
    try {
      await switchPlatform(platform)
      setIsOpen(false)
      
      // Reload page to update platform-specific admin content
      window.location.reload()
      
    } catch (error) {
      console.error('Failed to switch admin platform:', error)
    } finally {
      setIsLoading(false)
    }
  }
  
  if (availablePlatforms.length <= 1) {
    return (
      <div className={`inline-flex items-center gap-2 px-3 py-2 ${className}`}>
        {showIcon && (
          <span className="text-lg" role="img" aria-hidden="true">
            {getPlatformIcon(currentPlatform)}
          </span>
        )}
        {showLabel && (
          <span className="text-sm font-medium text-gray-700">
            {getPlatformDisplayName(currentPlatform)}
          </span>
        )}
        {showAdminBadge && (
          <span className="inline-flex items-center gap-1 px-2 py-0.5 text-xs font-medium bg-red-100 text-red-800 rounded-full">
            <ShieldCheckIcon className="w-3 h-3" />
            Admin
          </span>
        )}
      </div>
    )
  }
  
  return (
    <div className={`relative ${className}`}>
      <button
        onClick={() => setIsOpen(!isOpen)}
        disabled={isLoading}
        className="flex items-center gap-2 px-3 py-2 text-sm font-medium text-gray-700 bg-white border border-gray-300 rounded-md shadow-sm hover:bg-gray-50 focus:outline-none focus:ring-2 focus:ring-offset-2 focus:ring-blue-500 disabled:opacity-50 disabled:cursor-not-allowed"
      >
        {showIcon && (
          <span className="text-lg" role="img" aria-hidden="true">
            {getPlatformIcon(currentPlatform)}
          </span>
        )}
        <div className="flex items-center gap-2">
          {showLabel && (
            <span>{getPlatformDisplayName(currentPlatform)}</span>
          )}
          {showAdminBadge && (
            <span className="inline-flex items-center gap-1 px-2 py-0.5 text-xs font-medium bg-red-100 text-red-800 rounded-full">
              <ShieldCheckIcon className="w-3 h-3" />
              Admin
            </span>
          )}
        </div>
        <ChevronDownIcon
          className={`w-4 h-4 transition-transform ${
            isOpen ? 'rotate-180' : ''
          }`}
        />
      </button>
      
      {isOpen && (
        <>
          {/* Backdrop */}
          <div
            className="fixed inset-0 z-10"
            onClick={() => setIsOpen(false)}
          />
          
          {/* Dropdown Menu */}
          <div className="absolute right-0 z-20 mt-2 w-72 bg-white border border-gray-200 rounded-md shadow-lg">
            <div className="py-1">
              <div className="px-4 py-3 text-xs font-medium text-gray-500 uppercase tracking-wider border-b border-gray-100 bg-gray-50">
                <div className="flex items-center gap-2">
                  <CogIcon className="w-4 h-4" />
                  Admin Platform Management
                </div>
              </div>
              
              {availablePlatforms.map((platform) => (
                <button
                  key={platform}
                  onClick={() => handlePlatformSwitch(platform)}
                  disabled={isLoading || !canAccessPlatform(platform)}
                  className={`w-full flex items-center gap-3 px-4 py-3 text-sm text-left hover:bg-gray-50 disabled:opacity-50 disabled:cursor-not-allowed ${
                    platform === currentPlatform
                      ? 'bg-blue-50 text-blue-700'
                      : 'text-gray-700'
                  }`}
                >
                  <span className="text-lg" role="img" aria-hidden="true">
                    {getPlatformIcon(platform)}
                  </span>
                  
                  <div className="flex-1">
                    <div className="flex items-center gap-2">
                      <span className="font-medium">
                        {getPlatformDisplayName(platform)}
                      </span>
                      {platform === currentPlatform && (
                        <span className="inline-flex items-center gap-1 px-2 py-0.5 text-xs font-medium bg-blue-100 text-blue-800 rounded-full">
                          Current
                        </span>
                      )}
                    </div>
                    <div className="text-xs text-gray-500">
                      Admin access to {getPlatformDescription(platform)}
                    </div>
                  </div>
                  
                  {platform === currentPlatform && (
                    <CheckIcon className="w-4 h-4 text-blue-600" />
                  )}
                </button>
              ))}
              
              {canManage && (
                <div className="border-t border-gray-100 pt-2">
                  <button className="w-full flex items-center gap-3 px-4 py-2 text-sm text-gray-600 hover:bg-gray-50">
                    <CogIcon className="w-4 h-4" />
                    <span>Manage Platforms</span>
                  </button>
                </div>
              )}
              
              {error && (
                <div className="px-4 py-2 text-xs text-red-600 border-t border-gray-100">
                  {error}
                </div>
              )}
            </div>
          </div>
        </>
      )}
    </div>
  )
}

function getPlatformDescription(platform: string): string {
  const descriptions: Record<string, string> = {
    'epsx': 'trading platform management',
    'epsx-pay': 'payment system administration',
    'epsx-token': 'token governance oversight',
  }
  
  return descriptions[platform] || 'platform administration'
}

// Admin platform context hook
export function useAdminPlatformContext() {
  const { 
    getCurrentPlatform, 
    getAvailablePlatforms, 
    canAccessPlatform, 
    switchPlatform,
    canManagePlatforms,
    canManageUsers,
    canManageSystem,
    canViewAnalytics 
  } = useAuth.getState()
  
  const currentPlatform = getCurrentPlatform()
  const availablePlatforms = getAvailablePlatforms()
  
  return {
    currentPlatform,
    availablePlatforms,
    canAccessPlatform,
    switchPlatform,
    canManagePlatforms: canManagePlatforms(),
    hasMultiplePlatforms: availablePlatforms.length > 1,
    platformDisplayName: getPlatformDisplayName(currentPlatform),
    platformIcon: getPlatformIcon(currentPlatform),
    
    // Admin-specific capabilities per platform
    adminCapabilities: {
      canManageUsers: canManageUsers(),
      canManageSystem: canManageSystem(),
      canViewAnalytics: canViewAnalytics(),
      canManagePlatforms: canManagePlatforms(),
    }
  }
}