/**
 * Cross-Platform Authentication Hook
 * Stub implementation for build compatibility
 */

import { BarChart3, Coins, Shield } from 'lucide-react';

export function useCrossPlatformAuth() {
  return {
    user: {
      id: 'admin-user',
      email: 'admin@example.com',
      name: 'Admin User',
      platforms: ['admin', 'epsx', 'epsx-pay'],
      permissions: ['admin:*:*']
    },
    isAuthenticated: true,
    isLoading: false,
    error: null,
    hasPermission: (permission: string) => false,
    login: async () => { },
    logout: async () => { },
    switchPlatform: async (platform: string) => { },
  };
}

export function usePlatformContext() {
  return {
    currentPlatform: {
      id: 'admin',
      code: 'admin',
      name: 'Admin',
      icon: Shield,
      description: 'Administrative dashboard'
    },
    availablePlatforms: [
      { id: 'admin', code: 'admin', name: 'Admin', icon: Shield, description: 'Administrative dashboard' },
      { id: 'epsx', code: 'epsx', name: 'EPSX', icon: BarChart3, description: 'Market analytics platform' },
      { id: 'epsx-pay', code: 'epsx-pay', name: 'EPSX Pay', icon: Coins, description: 'Payment processing' }
    ],
    accessiblePlatforms: [
      { id: 'admin', code: 'admin', name: 'Admin', icon: Shield, description: 'Administrative dashboard' },
      { id: 'epsx', code: 'epsx', name: 'EPSX', icon: BarChart3, description: 'Market analytics platform' },
      { id: 'epsx-pay', code: 'epsx-pay', name: 'EPSX Pay', icon: Coins, description: 'Payment processing' }
    ],
    switchPlatform: (platform: string) => { },
    switchToPlatform: (platform: string) => { },
  };
}

export function usePlatformPermissions() {
  return {
    can: (permission: string) => false,
    hasAnyPermission: (permissions: string[]) => false,
    hasAllPermissions: (permissions: string[]) => false,
    platformPermissions: [],
  };
}