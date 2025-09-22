interface NavItem {
  label: string;
  href: string;
  key: string;
  icon?: React.ReactNode;
  requireAuth?: boolean;
  hasDropdown?: boolean;
  children?: NavItem[];
}

class NavigationService {
  getNavItems(isLoggedIn: boolean): NavItem[] {
    const items: NavItem[] = [
      // {
      //   label: 'Docs',
      //   href: 'https://your-gitbook-url.com',
      //   key: 'docs',
      // },
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
            label: 'Market Overview',
            href: '/analytics/market',
            key: 'market',
          },
          {
            label: 'Stock Screener',
            href: '/analytics/screener',
            key: 'screener',
          },
        ],
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

    return items;
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
