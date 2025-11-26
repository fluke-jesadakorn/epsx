interface NavItem {
  label: string;
  href: string;
  key: string;
  icon?: React.ReactNode;
  hasDropdown?: boolean;
  children?: NavItem[];
}

class NavigationService {
  // Simplified navigation items - no permission filtering
  private getAllNavigationItems(): NavItem[] {
    return [
      {
        label: 'Analytics',
        href: '/analytics',
        key: 'analytics',
        hasDropdown: true,
        children: [
          {
            label: 'EPS Ranking',
            href: '/analytics',
            key: 'ranking',
          },
          {
            label: 'Portfolio',
            href: '/portfolio',
            key: 'portfolio',
          },
        ],
      },
      {
        label: 'About Us',
        href: '/about',
        key: 'about',
      },
    ];
  }

  // Simplified method - just return all navigation items without filtering
  getNavItems(): NavItem[] {
    return this.getAllNavigationItems();
  }

  // Get all navigation items (same as getNavItems now)
  getAllNavItems(): NavItem[] {
    return this.getAllNavigationItems();
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
