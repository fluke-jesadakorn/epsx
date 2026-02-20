import type { LucideIcon } from 'lucide-react';
import {
  BarChart3,
  BookOpen,
  Building2,
  Code,
  HelpCircle,
  Info,
  LineChart,
  Mail,
  TrendingUp,
} from 'lucide-react';

export interface NavItem {
  label: string;
  href: string;
  key: string;
  icon?: LucideIcon;
  desc?: string;
}

export interface NavGroup {
  label: string;
  key: string;
  icon?: LucideIcon;
  items: NavItem[];
}

export const NAV_GROUPS: NavGroup[] = [
  {
    label: 'Market',
    key: 'market',
    icon: BarChart3,
    items: [
      { label: 'Rankings', href: '/analytics', key: 'rankings', icon: LineChart, desc: 'EPS stock rankings' },
      { label: 'Portfolio', href: '/portfolio', key: 'portfolio', icon: TrendingUp, desc: 'Watchlist & tracking' },
    ],
  },
  {
    label: 'Developer',
    key: 'developer',
    icon: Code,
    items: [
      { label: 'API Keys', href: '/developer', key: 'api-keys', icon: Code, desc: 'Manage API access' },
      { label: 'Documentation', href: '/developer/docs', key: 'docs', icon: BookOpen, desc: 'API reference' },
    ],
  },
  {
    label: 'Company',
    key: 'company',
    icon: Building2,
    items: [
      { label: 'About', href: '/about', key: 'about', icon: Info },
      { label: 'Contact', href: '/contact', key: 'contact', icon: Mail },
      { label: 'Support', href: '/chat', key: 'support', icon: HelpCircle },
    ],
  },
];

export const FOOTER_LINKS: NavItem[] = [
  { label: 'Terms of Service', href: '/terms', key: 'terms' },
  { label: 'Privacy Policy', href: '/privacy', key: 'privacy' },
  { label: 'Contact', href: '/contact', key: 'contact' },
];

/** Check if a path is active within a group */
export function isGroupActive(group: NavGroup, pathname: string): boolean {
  return group.items.some(item => pathname === item.href);
}

/** Check if a specific item is the active route */
export function isItemActive(item: NavItem, pathname: string): boolean {
  return pathname === item.href;
}
