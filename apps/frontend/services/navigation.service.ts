interface NavItem {
  label: string;
  href: string;
  key: string;
  icon?: React.ReactNode;
  requireAuth?: boolean;
  hasDropdown?: boolean;
  children?: NavItem[];
  // Permission-aware properties
  requiredPermissions?: string[];
  requireAllPermissions?: boolean; // true = require ALL permissions, false = require ANY
  minimumTier?: 'free' | 'basic' | 'premium' | 'professional';
  upgradePrompt?: string;
  betaFeature?: boolean;
}

class NavigationService {
  // Permission-aware navigation items with user requirements
  private getPermissionAwareNavItems(): NavItem[] {
    return [
      {
        label: 'Analytics',
        href: '/analytics',
        key: 'analytics',
        hasDropdown: true,
        requiredPermissions: ['epsx:analytics:view', 'epsx:analytics:basic', 'epsx:analytics:premium'],
        requireAllPermissions: false, // User needs ANY of these permissions
        upgradePrompt: 'Upgrade to access analytics features',
        children: [
          {
            label: 'EPS Ranking',
            href: '/analytics',
            key: 'ranking',
            requiredPermissions: ['epsx:ranking:25', 'epsx:ranking:50', 'epsx:ranking:100', 'epsx:ranking:unlimited'],
            requireAllPermissions: false,
            upgradePrompt: 'Upgrade to view EPS rankings'
          },
          {
            label: 'Market Overview',
            href: '/analytics/market',
            key: 'market',
            requiredPermissions: ['epsx:analytics:premium', 'epsx:analytics:professional'],
            requireAllPermissions: false,
            minimumTier: 'premium',
            upgradePrompt: 'Premium feature - upgrade to view market overview'
          },
          {
            label: 'Stock Screener',
            href: '/analytics/screener',
            key: 'screener',
            requiredPermissions: ['epsx:filters:advanced', 'epsx:search:advanced'],
            requireAllPermissions: false,
            minimumTier: 'basic',
            upgradePrompt: 'Upgrade to access advanced screening tools'
          },
        ],
      },
      {
        label: 'My Data',
        href: '/my-data',
        key: 'my-data',
        requireAuth: true,
        requiredPermissions: ['epsx:profile:manage'],
        upgradePrompt: 'Sign in to view your data'
      },
      {
        label: 'Export',
        href: '/export',
        key: 'export',
        requireAuth: true,
        requiredPermissions: ['epsx:export:csv', 'epsx:export:excel', 'epsx:export:pdf'],
        requireAllPermissions: false,
        minimumTier: 'basic',
        upgradePrompt: 'Upgrade to export data in various formats'
      },
      {
        label: 'Real-time',
        href: '/realtime',
        key: 'realtime',
        requireAuth: true,
        requiredPermissions: ['epsx:realtime:access'],
        minimumTier: 'premium',
        upgradePrompt: 'Get premium access for real-time data',
        betaFeature: true
      },
      {
        label: 'Permissions',
        href: '/permissions',
        key: 'permissions',
        requireAuth: true,
        requiredPermissions: ['epsx:profile:manage'],
        upgradePrompt: 'Sign in to manage permissions'
      },
      {
        label: 'Settings',
        href: '/settings',
        key: 'settings',
        requireAuth: true,
        requiredPermissions: ['epsx:profile:manage'],
        upgradePrompt: 'Sign in to access settings'
      },
      {
        label: 'About Us',
        href: '/about',
        key: 'about',
        hasDropdown: true,
        children: [
          {
            label: 'Our Story',
            href: '/about',
            key: 'story',
          },
          {
            label: 'Team',
            href: '/about/team',
            key: 'team',
          },
          {
            label: 'Contact',
            href: '/about/contact',
            key: 'contact',
          },
        ],
      },
    ];
  }

  // Permission checking utility
  private hasRequiredPermissions(
    item: NavItem, 
    userPermissions: string[] = [], 
    isLoggedIn: boolean = false
  ): boolean {
    // If authentication is required but user is not logged in
    if (item.requireAuth && !isLoggedIn) {
      return false;
    }

    // If no permission requirements, allow access
    if (!item.requiredPermissions || item.requiredPermissions.length === 0) {
      return true;
    }

    // Check if user has required permissions
    if (item.requireAllPermissions) {
      // Require ALL permissions
      return item.requiredPermissions.every(permission => 
        userPermissions.some(userPerm => 
          userPerm === permission || 
          userPerm.startsWith(permission.split(':').slice(0, 2).join(':'))
        )
      );
    } else {
      // Require ANY permission
      return item.requiredPermissions.some(permission =>
        userPermissions.some(userPerm => 
          userPerm === permission || 
          userPerm.startsWith(permission.split(':').slice(0, 2).join(':'))
        )
      );
    }
  }

  // Filter navigation items based on user permissions
  private filterNavItemsByPermissions(
    items: NavItem[], 
    userPermissions: string[] = [], 
    isLoggedIn: boolean = false
  ): NavItem[] {
    return items.filter(item => {
      // Check if user has access to main item
      const hasMainAccess = this.hasRequiredPermissions(item, userPermissions, isLoggedIn);
      
      // Filter children if they exist
      if (item.children) {
        const accessibleChildren = this.filterNavItemsByPermissions(
          item.children, 
          userPermissions, 
          isLoggedIn
        );
        
        // If main item has no access but has accessible children, show it
        // If main item has access, show it with filtered children
        if (hasMainAccess || accessibleChildren.length > 0) {
          return {
            ...item,
            children: accessibleChildren
          };
        }
        return false;
      }
      
      return hasMainAccess;
    }).map(item => {
      // Ensure we return the item with filtered children
      if (item.children) {
        return {
          ...item,
          children: this.filterNavItemsByPermissions(item.children, userPermissions, isLoggedIn)
        };
      }
      return item;
    });
  }

  getNavItems(isLoggedIn: boolean, userPermissions: string[] = []): NavItem[] {
    const allItems = this.getPermissionAwareNavItems();
    return this.filterNavItemsByPermissions(allItems, userPermissions, isLoggedIn);
  }

  // Get all navigation items (for admin/debugging purposes)
  getAllNavItems(): NavItem[] {
    return this.getPermissionAwareNavItems();
  }

  // Check if a specific route is accessible
  isRouteAccessible(route: string, userPermissions: string[] = [], isLoggedIn: boolean = false): boolean {
    const allItems = this.getPermissionAwareNavItems();
    
    const findItemByRoute = (items: NavItem[]): NavItem | null => {
      for (const item of items) {
        if (item.href === route) {
          return item;
        }
        if (item.children) {
          const childItem = findItemByRoute(item.children);
          if (childItem) return childItem;
        }
      }
      return null;
    };

    const item = findItemByRoute(allItems);
    if (!item) return true; // If route not found in nav, allow access (might be a public route)
    
    return this.hasRequiredPermissions(item, userPermissions, isLoggedIn);
  }

  // Get upgrade message for a specific route
  getUpgradeMessageForRoute(route: string): string | null {
    const allItems = this.getPermissionAwareNavItems();
    
    const findItemByRoute = (items: NavItem[]): NavItem | null => {
      for (const item of items) {
        if (item.href === route) {
          return item;
        }
        if (item.children) {
          const childItem = findItemByRoute(item.children);
          if (childItem) return childItem;
        }
      }
      return null;
    };

    const item = findItemByRoute(allItems);
    return item?.upgradePrompt || null;
  }

  getFooterLinks(): NavItem[] {
    return [
      {
        label: 'Terms of Service',
        href: '/terms',
        key: 'terms',
      },
      {
        label: 'Privacy Policy',
        href: '/privacy',
        key: 'privacy',
      },
    ];
  }
}

export const navigationService = new NavigationService();
