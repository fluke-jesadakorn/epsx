'use client'

import { useState } from 'react'
import { ChevronDownIcon, CheckIcon } from '@heroicons/react/24/outline'
import { useAuth } from '@/lib/auth'
import { getPlatformDisplayName, getPlatformIcon } from '@/lib/auth'

interface PlatformSwitcherProps {
  className?: string
  showIcon?: boolean
  showLabel?: boolean
}

export default function PlatformSwitcher({
  className = '',
  showIcon = true,
  showLabel = true,
}: PlatformSwitcherProps) {
  const [isOpen, setIsOpen] = useState(false)
  const [isLoading, setIsLoading] = useState(false)
  
  const {
    getCurrentPlatform,
    getAvailablePlatforms,
    canAccessPlatform,
    switchPlatform,
    error,
  } = useAuth.getState()
  
  const currentPlatform = getCurrentPlatform()
  const availablePlatforms = getAvailablePlatforms()
  
  const handlePlatformSwitch = async (platform: string) => {
    if (platform === currentPlatform) {
      setIsOpen(false)
      return
    }
    
    setIsLoading(true)
    try {
      await switchPlatform(platform)
      setIsOpen(false)
      
      // Reload page to update platform-specific content
      window.location.reload()
      
    } catch (error) {
      console.error('Failed to switch platform:', error)
    } finally {
      setIsLoading(false)
    }
  }
  
  if (availablePlatforms.length <= 1) {
    // Don't show switcher if user only has access to one platform
    return null
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
        {showLabel && (
          <span>{getPlatformDisplayName(currentPlatform)}</span>
        )}
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
          <div className="absolute right-0 z-20 mt-2 w-56 bg-white border border-gray-200 rounded-md shadow-lg">
            <div className="py-1">
              <div className="px-4 py-2 text-xs font-medium text-gray-500 uppercase tracking-wider border-b border-gray-100">
                Switch Platform
              </div>
              
              {availablePlatforms.map((platform) => (
                <button
                  key={platform}
                  onClick={() => handlePlatformSwitch(platform)}
                  disabled={isLoading || !canAccessPlatform(platform)}
                  className={`w-full flex items-center gap-3 px-4 py-2 text-sm text-left hover:bg-gray-50 disabled:opacity-50 disabled:cursor-not-allowed ${
                    platform === currentPlatform
                      ? 'bg-blue-50 text-blue-700'
                      : 'text-gray-700'
                  }`}
                >
                  <span className="text-lg" role="img" aria-hidden="true">
                    {getPlatformIcon(platform)}
                  </span>
                  
                  <div className="flex-1">
                    <div className="font-medium">
                      {getPlatformDisplayName(platform)}
                    </div>
                    <div className="text-xs text-gray-500">
                      {getPlatformDescription(platform)}
                    </div>
                  </div>
                  
                  {platform === currentPlatform && (
                    <CheckIcon className="w-4 h-4 text-blue-600" />
                  )}
                </button>
              ))}
              
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
    'epsx': 'Stock analytics & trading',
    'epsx-pay': 'Payment processing',
    'epsx-token': 'Token governance',
  }
  
  return descriptions[platform] || 'Platform services'
}