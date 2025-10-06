'use client';

import { 
  Globe, 
  Coins, 
  Vote, 
  ChevronDown, 
  ExternalLink, 
  Settings,
  Users,
  Shield,
  BarChart3,
  Database,
  Zap,
  Wallet,
  Gavel
} from 'lucide-react';
import Link from 'next/link';
import { useRouter, usePathname } from 'next/navigation';
import { useState } from 'react';

import { useCrossPlatformAuth, usePlatformContext, usePlatformPermissions } from '../../hooks/useCrossPlatformAuth';
import { PlatformSwitcher } from '../shared/UnifiedPlatformSwitcher';

interface NavigationItem {
  id: string;
  label: string;
  href: string;
  icon: typeof Settings;
  permission?: string;
  description?: string;
  badge?: string;
}

interface PlatformNavigationConfig {
  [platformCode: string]: {
    navigation: NavigationItem[];
    color: string;
    gradient: string;
  };
}

const platformNavigation: PlatformNavigationConfig = {
  'epsx': {
    color: 'text-blue-600 dark:text-blue-400',
    gradient: 'from-blue-500 to-blue-600',
    navigation: [
      {
        id: 'dashboard',
        label: 'Dashboard',
        href: '/admin',
        icon: BarChart3,
        description: 'System overview and analytics'
      },
      {
        id: 'users',
        label: 'User Management',
        href: '/admin/users',
        icon: Users,
        permission: 'epsx:users:read',
        description: 'Manage platform users'
      },
      {
        id: 'analytics',
        label: 'Analytics',
        href: '/admin/analytics',
        icon: BarChart3,
        permission: 'epsx:analytics:read',
        description: 'EPS growth analytics'
      },
      {
        id: 'settings',
        label: 'Settings',
        href: '/admin/settings',
        icon: Settings,
        permission: 'epsx:settings:read',
        description: 'Platform configuration'
      }
    ]
  },
  'epsx-pay': {
    color: 'text-green-600 dark:text-green-400',
    gradient: 'from-green-500 to-green-600',
    navigation: [
      {
        id: 'pay-dashboard',
        label: 'Payment Dashboard',
        href: '/admin/pay',
        icon: Coins,
        description: 'Payment system overview'
      },
      {
        id: 'transactions',
        label: 'Transactions',
        href: '/admin/pay/transactions',
        icon: Zap,
        permission: 'epsx-pay:transactions:read',
        description: 'Crypto transaction management'
      },
      {
        id: 'wallets',
        label: 'Wallets',
        href: '/admin/pay/wallets',
        icon: Wallet,
        permission: 'epsx-pay:wallets:read',
        description: 'Multi-sig wallet management'
      },
      {
        id: 'defi',
        label: 'DeFi Protocols',
        href: '/admin/pay/defi',
        icon: BarChart3,
        permission: 'epsx-pay:defi:read',
        description: 'DeFi integration management'
      },
      {
        id: 'pay-settings',
        label: 'Payment Settings',
        href: '/admin/pay/settings',
        icon: Settings,
        permission: 'epsx-pay:settings:read',
        description: 'Payment system configuration'
      }
    ]
  },
  'epsx-token': {
    color: 'text-purple-600 dark:text-purple-400',
    gradient: 'from-purple-500 to-purple-600',
    navigation: [
      {
        id: 'token-dashboard',
        label: 'Governance Dashboard',
        href: '/admin/token',
        icon: Vote,
        description: 'Governance system overview'
      },
      {
        id: 'proposals',
        label: 'Proposals',
        href: '/admin/token/proposals',
        icon: Gavel,
        permission: 'epsx-token:governance:read',
        description: 'Governance proposals management'
      },
      {
        id: 'treasury',
        label: 'Treasury',
        href: '/admin/token/treasury',
        icon: Database,
        permission: 'epsx-token:treasury:read',
        description: 'Treasury funds management'
      },
      {
        id: 'contracts',
        label: 'Smart Contracts',
        href: '/admin/token/contracts',
        icon: Shield,
        permission: 'epsx-token:contracts:read',
        description: 'Smart contract management'
      },
      {
        id: 'token-settings',
        label: 'Token Settings',
        href: '/admin/token/settings',
        icon: Settings,
        permission: 'epsx-token:settings:read',
        description: 'Token system configuration'
      }
    ]
  }
};

const platformIcons: { [key: string]: typeof Globe } = {
  'epsx': Globe,
  'epsx-pay': Coins,
  'epsx-token': Vote,
};

/**
 *
 */
export function CrossPlatformNavigation() {
  const { user, hasPermission } = useCrossPlatformAuth();
  const { currentPlatform, switchToPlatform, accessiblePlatforms } = usePlatformContext();
  const pathname = usePathname();
  const router = useRouter();
  const [isSwitching, setIsSwitching] = useState(false);

  if (!currentPlatform || !user) {
    return (
      <div className="bg-white dark:bg-gray-800 shadow-sm border-b border-gray-200 dark:border-gray-700">
        <div className="px-4 py-3">
          <div className="animate-pulse flex space-x-4">
            <div className="rounded-full bg-gray-300 h-8 w-8"></div>
            <div className="flex-1 space-y-2 py-1">
              <div className="h-4 bg-gray-300 rounded w-3/4"></div>
            </div>
          </div>
        </div>
      </div>
    );
  }

  const currentConfig = platformNavigation[currentPlatform.code];
  if (!currentConfig) {
    return null;
  }

  const handlePlatformSwitch = async (platformCode: string) => {
    try {
      setIsSwitching(true);
      await switchToPlatform(platformCode);
      
      // Navigate to the platform's default admin route
      if (platformCode === 'epsx') {
        router.push('/admin');
      } else {
        router.push(`/admin/${platformCode}`);
      }
    } catch (_error) {
      // eslint-disable-next-line no-console
      console.error('Failed to switch platform:', _error);
    } finally {
      setIsSwitching(false);
    }
  };

  const filteredNavigation = currentConfig.navigation.filter(item => {
    // If no permission required, show the item
    if (!item.permission) {return true;}
    
    // Check if user has the required permission
    return hasPermission(item.permission);
  });

  const isActiveRoute = (href: string): boolean => {
    if (href === '/admin') {
      return pathname === '/admin';
    }
    return pathname.startsWith(href);
  };

  return (
    <div className="bg-white dark:bg-gray-800 shadow-sm border-b border-gray-200 dark:border-gray-700">
      {/* Platform Header */}
      <div className="px-4 py-3 border-b border-gray-100 dark:border-gray-700">
        <div className="flex items-center justify-between">
          <div className="flex items-center gap-3">
            <PlatformSwitcher
              currentPlatform={currentPlatform}
              availablePlatforms={accessiblePlatforms}
              userPlatformAccess={user.platforms}
              onPlatformSwitch={handlePlatformSwitch}
            />
            
            {isSwitching && (
              <div className="flex items-center gap-2 text-sm text-gray-500">
                <div className="animate-spin rounded-full h-4 w-4 border-2 border-gray-300 border-t-blue-600"></div>
                Switching platform...
              </div>
            )}
          </div>
          
          {/* Platform Info */}
          <div className="hidden md:flex items-center gap-2 text-sm text-gray-500 dark:text-gray-400">
            <span>Platform:</span>
            <span className={`font-medium ${currentConfig.color}`}>
              {currentPlatform.name}
            </span>
          </div>
        </div>
      </div>

      {/* Navigation Menu */}
      <div className="px-4 py-2">
        <nav className="flex space-x-1 overflow-x-auto">
          {filteredNavigation.map((item) => {
            const Icon = item.icon;
            const isActive = isActiveRoute(item.href);
            
            return (
              <Link
                key={item.id}
                href={item.href}
                className={`flex items-center gap-2 px-3 py-2 rounded-lg text-sm font-medium transition-colors whitespace-nowrap ${
                  isActive
                    ? `bg-gradient-to-r ${currentConfig.gradient} text-white shadow-sm`
                    : 'text-gray-600 dark:text-gray-400 hover:text-gray-900 dark:hover:text-white hover:bg-gray-100 dark:hover:bg-gray-700'
                }`}
                title={item.description}
              >
                <Icon className="h-4 w-4" />
                <span>{item.label}</span>
                {item.badge && (
                  <span className="ml-1 px-1.5 py-0.5 bg-red-100 text-red-800 text-xs rounded-full">
                    {item.badge}
                  </span>
                )}
              </Link>
            );
          })}
        </nav>
      </div>

      {/* Cross-Platform Quick Actions */}
      <div className="px-4 py-2 border-t border-gray-100 dark:border-gray-700 bg-gray-50 dark:bg-gray-900">
        <div className="flex items-center justify-between">
          <div className="flex items-center gap-4 text-xs text-gray-500 dark:text-gray-400">
            <span>Quick Switch:</span>
            {accessiblePlatforms
              .filter(p => p.code !== currentPlatform.code)
              .slice(0, 2)
              .map((platform) => {
                const Icon = platformIcons[platform.code] || Globe;
                return (
                  <button
                    key={platform.code}
                    onClick={() => handlePlatformSwitch(platform.code)}
                    className="flex items-center gap-1 hover:text-gray-700 dark:hover:text-gray-300 transition-colors"
                    disabled={isSwitching}
                  >
                    <Icon className="h-3 w-3" />
                    <span>{platform.code.toUpperCase()}</span>
                    <ExternalLink className="h-2.5 w-2.5" />
                  </button>
                );
              })}
          </div>
          
          <div className="flex items-center gap-2 text-xs text-gray-500 dark:text-gray-400">
            <Shield className="h-3 w-3" />
            <span>{user.permissions.filter(p => p.startsWith(`${currentPlatform.code}:`)).length} permissions</span>
          </div>
        </div>
      </div>
    </div>
  );
}