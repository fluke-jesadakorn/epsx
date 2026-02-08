
export interface NavItem {
    id: string;
    label: string;
    href: string;
    icon: string;
    description?: string;
    children?: NavItem[];
}

export const navigationItems: NavItem[] = [
    {
        id: 'dashboard',
        label: 'API Keys',
        href: '/developer',
        icon: '🔑',
        description: 'Manage your API keys',
    },
    {
        id: 'documentation',
        label: 'Documentation',
        href: '/developer/docs',
        icon: '📚',
        description: 'API Reference',
    },
    {
        id: 'usage',
        label: 'Usage & Monitoring',
        href: '/developer/usage',
        icon: '📊',
        description: 'Monitor your usage',
    },
];
