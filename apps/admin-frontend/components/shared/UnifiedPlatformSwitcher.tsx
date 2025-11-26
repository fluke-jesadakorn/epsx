/**
 * Unified Platform Switcher
 * Consolidates: components/platforms/PlatformSwitcher.tsx and components/auth/AdminPlatformSwitcher.tsx
 * Reduces ~200 lines of duplicate code
 */

'use client';

import { Check, ChevronDown, Coins, Globe, Settings, Shield, Vote } from 'lucide-react';
import { useState } from 'react';

import { useAuth } from '@/lib/auth';

// ============================================================================
// TYPES AND INTERFACES
// ============================================================================

interface Platform {
  id: string;
  code: string;
  name: string;
  description: string;
  icon?: typeof Globe;
  baseUrl?: string;
}

interface PlatformSwitcherConfig {
  mode: 'basic' | 'admin';
  showIcon?: boolean;
  showLabel?: boolean;
  showAdminBadge?: boolean;
  showDescription?: boolean;
  className?: string;
  size?: 'sm' | 'md' | 'lg';
  variant?: 'default' | 'minimal' | 'compact';
}

interface UnifiedPlatformSwitcherProps {
  // Configuration
  config?: Partial<PlatformSwitcherConfig>;
  
  // Basic mode props (for backward compatibility)
  currentPlatform?: Platform;
  availablePlatforms?: Platform[];
  userPlatformAccess?: string[];
  onPlatformSwitch?: (platformCode: string) => void;
  
  // Custom styling
  className?: string;
}

// ============================================================================
// PLATFORM MAPPINGS AND UTILITIES
// ============================================================================

const platformIcons: Record<string, typeof Globe> = {
  'epsx': Globe,
  'epsx-pay': Coins,
  'epsx-token': Vote,
};

const platformColors: Record<string, string> = {
  'epsx': 'bg-blue-500 text-white',
  'epsx-pay': 'bg-green-500 text-white',
  'epsx-token': 'bg-purple-500 text-white',
};

const platformDescriptions: Record<string, string> = {
  'epsx': 'Data analytics platform management',
  'epsx-pay': 'Payment system administration', 
  'epsx-token': 'Token governance oversight',
};

function getPlatformIcon(platformCode: string): typeof Globe {
  return platformIcons[platformCode] || Globe;
}

function getPlatformColor(platformCode: string): string {
  return platformColors[platformCode] || 'bg-gray-500 text-white';
}

function getPlatformDescription(platformCode: string): string {
  return platformDescriptions[platformCode] || 'Platform administration';
}

// ============================================================================
// MAIN COMPONENT
// ============================================================================

/**
 *
 * @param root0
 * @param root0.config
 * @param root0.currentPlatform
 * @param root0.availablePlatforms
 * @param root0.userPlatformAccess
 * @param root0.onPlatformSwitch
 * @param root0.className
 */
export function UnifiedPlatformSwitcher({
  config = {},
  currentPlatform,
  availablePlatforms,
  userPlatformAccess,
  onPlatformSwitch,
  className = '',
}: UnifiedPlatformSwitcherProps) {
  const [isOpen, setIsOpen] = useState(false);
  const [isLoading, setIsLoading] = useState(false);

  // Merge default config
  const finalConfig: PlatformSwitcherConfig = {
    mode: 'basic',
    showIcon: true,
    showLabel: true,
    showAdminBadge: false,
    showDescription: false,
    size: 'md',
    variant: 'default',
    className: '',
    ...config,
  };

  // ============================================================================
  // DATA RESOLUTION (Admin vs Basic Mode)
  // ============================================================================

  let resolvedCurrentPlatform: string;
  let resolvedAvailablePlatforms: string[];
  let resolvedUserAccess: string[];
  let resolvedOnSwitch: (platform: string) => void;

  if (finalConfig.mode === 'admin') {
    // Admin mode: use auth hooks
    const auth = useAuth.getState();
    resolvedCurrentPlatform = 'admin'; // Default current platform for admin
    resolvedAvailablePlatforms = ['admin', 'epsx'];
    resolvedUserAccess = resolvedAvailablePlatforms.filter(platform => 
      platform === 'admin' ? auth.isAdmin() : auth.can(`${platform}:*:*`)
    );
    resolvedOnSwitch = async (platform: string) => {
      if (platform === resolvedCurrentPlatform) {
        setIsOpen(false);
        return;
      }
      
      setIsLoading(true);
      try {
        // For admin mode, switch to different platform URLs
        if (platform === 'epsx') {
          window.location.href = 'https://epsx.io';
        } else if (platform === 'admin') {
          window.location.reload(); // Stay on admin
        }
        setIsOpen(false);
      } catch (_error) {
        // eslint-disable-next-line no-console
        console.error('Failed to switch admin platform:', _error);
      } finally {
        setIsLoading(false);
      }
    };
  } else {
    // Basic mode: use props
    resolvedCurrentPlatform = currentPlatform?.code || '';
    resolvedAvailablePlatforms = availablePlatforms?.map(p => p.code) || [];
    resolvedUserAccess = userPlatformAccess || [];
    resolvedOnSwitch = (platform: string) => {
      onPlatformSwitch?.(platform);
      setIsOpen(false);
    };
  }

  // Filter accessible platforms
  const accessiblePlatforms = resolvedAvailablePlatforms.filter(platform =>
    resolvedUserAccess.includes(platform)
  );

  // ============================================================================
  // UI VARIANTS
  // ============================================================================

  const sizeClasses = {
    sm: 'px-2 py-1 text-xs',
    md: 'px-3 py-2 text-sm',
    lg: 'px-4 py-3 text-base',
  };

  const iconSizes = {
    sm: 'h-3 w-3',
    md: 'h-4 w-4',
    lg: 'h-5 w-5',
  };

  // Single platform display (no dropdown needed)
  if (accessiblePlatforms.length <= 1) {
    const PlatformIcon = getPlatformIcon(resolvedCurrentPlatform);
    
    // Simple platform display name mapping
    const platformDisplayNames: Record<string, string> = {
      'admin': 'Admin Dashboard',
      'epsx': 'EPSX Platform',
    };
    
    const platformName = finalConfig.mode === 'admin' 
      ? platformDisplayNames[resolvedCurrentPlatform] || resolvedCurrentPlatform
      : currentPlatform?.name || resolvedCurrentPlatform;

    return (
      <div className={`inline-flex items-center gap-2 ${sizeClasses[finalConfig.size!]} ${className} ${finalConfig.className}`}>
        {finalConfig.showIcon && (
          <PlatformIcon className={iconSizes[finalConfig.size!]} />
        )}
        {finalConfig.showLabel && (
          <span className="font-medium">
            {platformName}
          </span>
        )}
        {finalConfig.showAdminBadge && finalConfig.mode === 'admin' && (
          <span className="inline-flex items-center gap-1 px-2 py-0.5 text-xs font-medium bg-red-100 text-red-800 rounded-full">
            <Shield className="w-3 h-3" />
            Admin
          </span>
        )}
      </div>
    );
  }

  // ============================================================================
  // DROPDOWN COMPONENT
  // ============================================================================

  const CurrentIcon = getPlatformIcon(resolvedCurrentPlatform);
  const currentColor = getPlatformColor(resolvedCurrentPlatform);
  // Platform display name mapping
  const platformDisplayNames: Record<string, string> = {
    'admin': 'Admin Dashboard',
    'epsx': 'EPSX Platform',
  };
  
  const currentName = finalConfig.mode === 'admin'
    ? platformDisplayNames[resolvedCurrentPlatform] || resolvedCurrentPlatform
    : currentPlatform?.name || resolvedCurrentPlatform;

  const buttonClasses = finalConfig.variant === 'minimal'
    ? `inline-flex items-center gap-2 ${sizeClasses[finalConfig.size!]} hover:bg-gray-50 dark:hover:bg-gray-800 rounded-md transition-colors`
    : finalConfig.mode === 'admin'
    ? `flex items-center gap-2 ${sizeClasses[finalConfig.size!]} font-medium text-gray-700 bg-white border border-gray-300 rounded-md shadow-sm hover:bg-gray-50 focus:outline-none focus:ring-2 focus:ring-offset-2 focus:ring-blue-500 disabled:opacity-50 disabled:cursor-not-allowed`
    : `inline-flex items-center gap-2 ${sizeClasses[finalConfig.size!]} rounded-lg font-medium hover:opacity-90 ${currentColor}`;

  return (
    <div className={`relative ${className} ${finalConfig.className}`}>
      <button
        onClick={() => setIsOpen(!isOpen)}
        disabled={isLoading}
        className={buttonClasses}
      >
        {finalConfig.showIcon && (
          <CurrentIcon className={iconSizes[finalConfig.size!]} />
        )}
        
        <div className="flex items-center gap-2">
          {finalConfig.showLabel && (
            <span className={finalConfig.variant === 'compact' ? 'hidden sm:inline' : ''}>
              {currentName}
            </span>
          )}
          {finalConfig.variant === 'compact' && (
            <span className="sm:hidden">
              {resolvedCurrentPlatform.toUpperCase()}
            </span>
          )}
          {finalConfig.showAdminBadge && finalConfig.mode === 'admin' && (
            <span className="inline-flex items-center gap-1 px-2 py-0.5 text-xs font-medium bg-red-100 text-red-800 rounded-full">
              <Shield className="w-3 h-3" />
              Admin
            </span>
          )}
        </div>
        
        <ChevronDown className={`${iconSizes[finalConfig.size!]} transition-transform ${
          isOpen ? 'rotate-180' : ''
        }`} />
      </button>

      {isOpen && (
        <>
          {/* Backdrop */}
          <div
            className="fixed inset-0 z-40"
            onClick={() => setIsOpen(false)}
          />
          
          {/* Dropdown Menu */}
          <div className={`absolute ${finalConfig.mode === 'admin' ? 'right-0' : 'left-0'} z-50 mt-2 w-64 bg-white dark:bg-gray-800 rounded-lg shadow-lg border border-gray-200 dark:border-gray-700`}>
            {/* Header */}
            <div className="px-4 py-3 border-b border-gray-200 dark:border-gray-700">
              <div className="flex items-center gap-2">
                {finalConfig.mode === 'admin' && <Settings className="w-4 h-4" />}
                <h3 className="text-sm font-medium text-gray-900 dark:text-white">
                  {finalConfig.mode === 'admin' ? 'Admin Platform Management' : 'Switch Platform'}
                </h3>
              </div>
              {finalConfig.mode !== 'admin' && (
                <p className="text-xs text-gray-500 dark:text-gray-400 mt-1">
                  Select a platform to manage
                </p>
              )}
            </div>
            
            {/* Platform List */}
            <div className="py-2">
              {accessiblePlatforms.map((platform) => {
                const PlatformIcon = getPlatformIcon(platform);
                const isActive = platform === resolvedCurrentPlatform;
                const color = getPlatformColor(platform);
                const platformName = finalConfig.mode === 'admin'
                  ? platformDisplayNames[platform] || platform
                  : availablePlatforms?.find(p => p.code === platform)?.name || platform;
                const description = finalConfig.mode === 'admin'
                  ? `Admin access to ${getPlatformDescription(platform)}`
                  : availablePlatforms?.find(p => p.code === platform)?.description || getPlatformDescription(platform);
                
                return (
                  <button
                    key={platform}
                    onClick={() => resolvedOnSwitch(platform)}
                    disabled={isLoading}
                    className={`w-full flex items-center gap-3 px-4 py-3 hover:bg-gray-50 dark:hover:bg-gray-700 disabled:opacity-50 disabled:cursor-not-allowed transition-colors ${
                      isActive ? 'bg-gray-50 dark:bg-gray-700' : ''
                    }`}
                  >
                    <div className={`p-2 rounded-lg ${color} opacity-80`}>
                      <PlatformIcon className="h-4 w-4" />
                    </div>
                    
                    <div className="flex-1 text-left">
                      <div className="flex items-center gap-2">
                        <span className="text-sm font-medium text-gray-900 dark:text-white">
                          {platformName}
                        </span>
                        {isActive && finalConfig.mode === 'admin' && (
                          <span className="inline-flex items-center gap-1 px-2 py-0.5 text-xs font-medium bg-blue-100 text-blue-800 rounded-full">
                            Current
                          </span>
                        )}
                      </div>
                      {(finalConfig.showDescription || finalConfig.mode === 'admin') && (
                        <div className="text-xs text-gray-500 dark:text-gray-400">
                          {description}
                        </div>
                      )}
                    </div>
                    
                    {isActive && (
                      finalConfig.mode === 'admin' 
                        ? <Check className="w-4 h-4 text-blue-600" />
                        : <div className="w-2 h-2 bg-blue-500 rounded-full" />
                    )}
                  </button>
                );
              })}
              
              {/* Manage Platforms Link (Admin mode only) */}
              {finalConfig.mode === 'admin' && useAuth.getState().isAdmin() && (
                <div className="border-t border-gray-100 dark:border-gray-600 pt-2 mt-2">
                  <button className="w-full flex items-center gap-3 px-4 py-2 text-sm text-gray-600 dark:text-gray-400 hover:bg-gray-50 dark:hover:bg-gray-700">
                    <Settings className="w-4 h-4" />
                    <span>Manage Platforms</span>
                  </button>
                </div>
              )}
              
              {/* Empty State */}
              {accessiblePlatforms.length === 0 && (
                <div className="px-4 py-3 text-sm text-gray-500 dark:text-gray-400">
                  No platforms accessible
                </div>
              )}
            </div>
          </div>
        </>
      )}
    </div>
  );
}

// ============================================================================
// CONVENIENCE EXPORTS FOR BACKWARD COMPATIBILITY
// ============================================================================

/**
 *
 * @param props
 * @param props.currentPlatform
 * @param props.availablePlatforms
 * @param props.userPlatformAccess
 * @param props.onPlatformSwitch
 * @param props.className
 */
export function PlatformSwitcher(props: {
  currentPlatform: Platform;
  availablePlatforms: Platform[];
  userPlatformAccess: string[];
  onPlatformSwitch: (platformCode: string) => void;
  className?: string;
}) {
  return (
    <UnifiedPlatformSwitcher
      config={{ mode: 'basic', variant: 'compact', showDescription: true }}
      {...props}
    />
  );
}

/**
 *
 * @param props
 * @param props.className
 * @param props.showIcon
 * @param props.showLabel
 * @param props.showAdminBadge
 */
export function AdminPlatformSwitcher(props: {
  className?: string;
  showIcon?: boolean;
  showLabel?: boolean;
  showAdminBadge?: boolean;
}) {
  return (
    <UnifiedPlatformSwitcher
      config={{
        mode: 'admin',
        showIcon: props.showIcon,
        showLabel: props.showLabel,
        showAdminBadge: props.showAdminBadge,
        variant: 'default',
        size: 'md',
      }}
      className={props.className}
    />
  );
}

export default UnifiedPlatformSwitcher;