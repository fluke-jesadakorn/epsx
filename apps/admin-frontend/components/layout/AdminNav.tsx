'use client';

import Link from 'next/link';
import { usePathname } from 'next/navigation';
import { useState, useEffect } from 'react';

interface NavItem {
  id: string;
  label: string;
  href: string;
  icon: string;
  gradient: string;
  description: string;
}

const navigationItems: NavItem[] = [
  {
    id: 'dashboard',
    label: 'Dashboard',
    href: '/',
    icon: '🏠',
    gradient: 'from-blue-500 to-blue-600',
    description: 'Overview & Stats',
  },
  {
    id: 'users',
    label: 'Users',
    href: '/users',
    icon: '👥',
    gradient: 'from-slate-500 to-slate-600',
    description: 'Manage Users',
  },
  {
    id: 'permissions',
    label: 'Permissions',
    href: '/permissions',
    icon: '🔑',
    gradient: 'from-green-500 to-green-600',
    description: 'Access Control',
  },
  {
    id: 'plans',
    label: 'Plans',
    href: '/plans',
    icon: '💳',
    gradient: 'from-purple-500 to-purple-600',
    description: 'Pricing Plans',
  },
  {
    id: 'promotions',
    label: 'Promotions',
    href: '/promotions',
    icon: '🎯',
    gradient: 'from-orange-500 to-orange-600',
    description: 'Marketing',
  },
  {
    id: 'analytics',
    label: 'Analytics',
    href: '/analytics',
    icon: '📊',
    gradient: 'from-indigo-500 to-indigo-600',
    description: 'Data Insights',
  },
  {
    id: 'notifications',
    label: 'Notifications',
    href: '/notifications',
    icon: '🔔',
    gradient: 'from-yellow-500 to-yellow-600',
    description: 'Send Alerts',
  },
  {
    id: 'remote-config',
    label: 'Config',
    href: '/remote-config',
    icon: '⚙️',
    gradient: 'from-gray-500 to-gray-600',
    description: 'App Settings',
  },
  {
    id: 'security',
    label: 'Security',
    href: '/security',
    icon: '🛡️',
    gradient: 'from-red-500 to-red-600',
    description: 'Security Monitor',
  },
  {
    id: 'developer-portal',
    label: 'Developer',
    href: '/developer-portal',
    icon: '👨‍💻',
    gradient: 'from-teal-500 to-teal-600',
    description: 'API Management',
  },
];

export default function AdminNav() {
  const pathname = usePathname();
  const [mounted, setMounted] = useState(false);

  useEffect(() => {
    setMounted(true);
  }, []);

  if (!mounted) {
    return null;
  }

  const isActive = (href: string) => {
    if (href === '/') {
      return pathname === '/';
    }
    return pathname.startsWith(href);
  };

  return (
    <nav className="h-full bg-white border-r border-gray-200 overflow-y-auto">
      <div className="p-6 border-b border-gray-200">
        <div className="flex items-center gap-3">
          <div className="w-10 h-10 bg-gradient-to-r from-blue-500 to-blue-600 rounded-lg flex items-center justify-center text-white font-semibold text-lg">
            E
          </div>
          <div>
            <h1 className="text-lg font-semibold text-gray-900">EPSX Admin</h1>
            <p className="text-xs text-gray-500">Analytics Platform</p>
          </div>
        </div>
      </div>

      <div className="p-4 space-y-2">
        {navigationItems.map((item) => (
          <Link
            key={item.id}
            href={item.href}
            className={`
              group relative flex items-center gap-3 px-3 py-2.5 rounded-lg transition-all duration-200
              ${
                isActive(item.href)
                  ? 'bg-blue-50 text-blue-700 border border-blue-200'
                  : 'text-gray-600 hover:bg-gray-50 hover:text-gray-900'
              }
            `}
          >
            <div
              className={`
                w-8 h-8 rounded-md flex items-center justify-center text-sm transition-all duration-200
                ${
                  isActive(item.href)
                    ? `bg-gradient-to-r ${item.gradient} text-white`
                    : 'bg-gray-100 group-hover:bg-gray-200'
                }
              `}
            >
              {item.icon}
            </div>
            <div className="flex-1 min-w-0">
              <p className="text-sm font-medium truncate">{item.label}</p>
              <p className="text-xs text-gray-500 truncate">{item.description}</p>
            </div>
            {isActive(item.href) && (
              <div className="w-2 h-2 bg-blue-500 rounded-full"></div>
            )}
          </Link>
        ))}
      </div>
    </nav>
  );
}