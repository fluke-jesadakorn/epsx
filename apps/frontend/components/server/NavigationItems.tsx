'use client';

import Link from 'next/link';
import { usePathname } from 'next/navigation';
import { 
  LineChart, 
  BarChart, 
  File, 
  Settings, 
  Database 
} from 'lucide-react';

const iconMap = {
  docs: <File className="h-4 w-4" />,
  ranking: <LineChart className="h-4 w-4" />,
  analytics: <BarChart className="h-4 w-4" />,
  settings: <Settings className="h-4 w-4" />,
  'my-data': <Database className="h-4 w-4" />,
};

interface NavigationItem {
  key: string;
  label: string;
  href: string;
}

interface NavigationItemsProps {
  items: NavigationItem[];
}

/**
 * Navigation items component with minimal client-side JavaScript
 */
export function NavigationItems({ items }: NavigationItemsProps) {
  const pathname = usePathname();

  return (
    <nav className="hidden lg:flex gap-2">
      {items.map(item => (
        <Link
          key={item.key}
          href={item.href}
          className={`flex items-center justify-center gap-2 rounded-lg px-4 py-2 text-sm font-medium transition-all duration-200 focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring focus-visible:ring-offset-2 hover:bg-primary/10 hover:text-accent-foreground active:scale-[0.98] ${
            pathname === item.href ? 'bg-primary/10 text-primary' : 'text-muted-foreground hover:text-primary'
          }`}
        >
          <span className="flex items-center justify-center">
            {iconMap[item.key as keyof typeof iconMap]}
          </span>
          <span className="hidden xl:block">{item.label}</span>
        </Link>
      ))}
    </nav>
  );
}