interface NavItem {
  label: string;
  href: string;
  key: string;
  icon?: React.ReactNode;
  requireAuth?: boolean;
}

class NavigationService {
  getNavItems(isLoggedIn: boolean): NavItem[] {
    const items: NavItem[] = [
      {
        label: "Docs",
        href: "https://your-gitbook-url.com",
        key: "docs",
      },
      {
        label: "Ranking",
        href: "/ranking",
        key: "ranking",
      },
    ];

    if (isLoggedIn) {
      items.push({
        label: "Settings",
        href: "/settings",
        key: "settings",
        requireAuth: true,
      });
    }

    return items;
  }

  getFooterLinks(): NavItem[] {
    return [
      {
        label: "Terms of Service",
        href: "/terms",
        key: "terms",
      },
      {
        label: "Privacy Policy",
        href: "/privacy",
        key: "privacy",
      },
    ];
  }
}

export const navigationService = new NavigationService();
