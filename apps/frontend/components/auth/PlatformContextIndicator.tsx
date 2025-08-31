'use client'

import { useState } from 'react'
import { useAuth } from '@/lib/auth'
import { getPlatformDisplayName, getPlatformIcon } from '@/lib/auth'
import PlatformSwitcher from './PlatformSwitcher'

interface PlatformContextIndicatorProps {
  className?: string
  variant?: 'minimal' | 'badge' | 'full'
  position?: 'top-left' | 'top-right' | 'bottom-left' | 'bottom-right'
}

export default function PlatformContextIndicator({
  className = '',
  variant = 'badge',
  position = 'top-right',
}: PlatformContextIndicatorProps) {
  const [showDetails, setShowDetails] = useState(false)
  const { 
    user, 
    isAuthenticated,
    getCurrentPlatform,
    getAvailablePlatforms 
  } = useAuth.getState()
  
  if (!isAuthenticated || !user) {
    return null
  }
  
  const currentPlatform = getCurrentPlatform()
  const availablePlatforms = getAvailablePlatforms()
  const hasMultiplePlatforms = availablePlatforms.length > 1
  
  const positionClasses = {
    'top-left': 'top-4 left-4',
    'top-right': 'top-4 right-4',
    'bottom-left': 'bottom-4 left-4',
    'bottom-right': 'bottom-4 right-4',
  }
  
  if (variant === 'minimal') {
    return (
      <div className={`flex items-center gap-2 ${className}`}>
        <span className="text-lg" role="img" aria-hidden="true">
          {getPlatformIcon(currentPlatform)}
        </span>
        {hasMultiplePlatforms && (
          <PlatformSwitcher 
            showIcon={false} 
            showLabel={false}
            className="scale-90"
          />
        )}
      </div>
    )
  }
  
  if (variant === 'badge') {
    return (
      <div className={`inline-flex items-center gap-2 ${className}`}>
        <div 
          className="inline-flex items-center gap-2 px-2.5 py-1.5 text-xs font-medium bg-gray-100 text-gray-800 rounded-full cursor-pointer hover:bg-gray-200 transition-colors"
          onClick={() => setShowDetails(!showDetails)}
        >
          <span className="text-sm" role="img" aria-hidden="true">
            {getPlatformIcon(currentPlatform)}
          </span>
          <span>{getPlatformDisplayName(currentPlatform)}</span>
          {hasMultiplePlatforms && (
            <span className="text-gray-500 text-[10px]">
              +{availablePlatforms.length - 1}
            </span>
          )}
        </div>
        
        {hasMultiplePlatforms && showDetails && (
          <div className="ml-2">
            <PlatformSwitcher showIcon={false} />
          </div>
        )}
      </div>
    )
  }
  
  if (variant === 'full') {
    return (
      <div className={`bg-white rounded-lg border border-gray-200 p-4 shadow-sm ${className}`}>
        <div className="flex items-center justify-between mb-3">
          <div className="flex items-center gap-2">
            <span className="text-lg" role="img" aria-hidden="true">
              {getPlatformIcon(currentPlatform)}
            </span>
            <div>
              <h3 className="text-sm font-medium text-gray-900">
                Current Platform
              </h3>
              <p className="text-lg font-semibold text-blue-600">
                {getPlatformDisplayName(currentPlatform)}
              </p>
            </div>
          </div>
          
          {hasMultiplePlatforms && (
            <PlatformSwitcher />
          )}
        </div>
        
        {hasMultiplePlatforms && (
          <div className="border-t border-gray-100 pt-3">
            <p className="text-xs text-gray-500 mb-2">
              Available Platforms:
            </p>
            <div className="flex flex-wrap gap-1">
              {availablePlatforms.map((platform) => (
                <span
                  key={platform}
                  className={`inline-flex items-center gap-1 px-2 py-1 text-xs rounded ${
                    platform === currentPlatform
                      ? 'bg-blue-100 text-blue-800'
                      : 'bg-gray-100 text-gray-700'
                  }`}
                >
                  <span role="img" aria-hidden="true">
                    {getPlatformIcon(platform)}
                  </span>
                  {getPlatformDisplayName(platform)}
                </span>
              ))}
            </div>
          </div>
        )}
        
        <div className="mt-3 pt-3 border-t border-gray-100">
          <div className="flex items-center gap-4 text-xs text-gray-500">
            <span>User: {user.email}</span>
            <span>Role: {user.role}</span>
            <span>Tier: {user.package_tier}</span>
          </div>
        </div>
      </div>
    )
  }
  
  return null
}

// Fixed position overlay version
export function PlatformContextOverlay({
  position = 'top-right',
  variant = 'badge',
}: Omit<PlatformContextIndicatorProps, 'className'>) {
  const { isAuthenticated, user } = useAuth.getState()
  
  if (!isAuthenticated || !user) {
    return null
  }
  
  const positionClasses = {
    'top-left': 'top-4 left-4',
    'top-right': 'top-4 right-4',
    'bottom-left': 'bottom-4 left-4',
    'bottom-right': 'bottom-4 right-4',
  }
  
  return (
    <div className={`fixed z-50 ${positionClasses[position]}`}>
      <PlatformContextIndicator variant={variant} />
    </div>
  )
}

// Hook for platform context
export function usePlatformIndicator() {
  const { 
    user, 
    isAuthenticated,
    getCurrentPlatform,
    getAvailablePlatforms 
  } = useAuth.getState()
  
  if (!isAuthenticated || !user) {
    return {
      isVisible: false,
      currentPlatform: null,
      availablePlatforms: [],
      hasMultiplePlatforms: false,
    }
  }
  
  const currentPlatform = getCurrentPlatform()
  const availablePlatforms = getAvailablePlatforms()
  
  return {
    isVisible: true,
    currentPlatform,
    availablePlatforms,
    hasMultiplePlatforms: availablePlatforms.length > 1,
    platformDisplayName: getPlatformDisplayName(currentPlatform),
    platformIcon: getPlatformIcon(currentPlatform),
  }
}