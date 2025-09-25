import Link from 'next/link';
import { LineChart, BarChart, File, Settings, Database, Crown, Shield, Code } from 'lucide-react';
import { getCurrentUser } from '@/lib/actions/auth';

const iconMap = {
  docs: <File className="h-4 w-4" />,
  ranking: <LineChart className="h-4 w-4" />,
  analytics: <BarChart className="h-4 w-4" />,
  settings: <Settings className="h-4 w-4" />,
  'my-data': <Database className="h-4 w-4" />,
  permissions: <Shield className="h-4 w-4" />,
  vip: <Crown className="h-4 w-4" />,
  developer: <Code className="h-4 w-4" />,
};

const publicMenuItems = [
  { key: 'docs', href: '/docs', label: 'Docs' },
  { key: 'ranking', href: '/ranking', label: 'Ranking' },
  { key: 'analytics', href: '/analytics', label: 'Analytics' },
];

const userMenuItems = [
  { key: 'my-data', href: '/my-data', label: 'My Data' },
  { key: 'permissions', href: '/permissions', label: 'Permissions' },
  { key: 'developer', href: '/developer', label: 'Developer API' },
  { key: 'settings', href: '/settings', label: 'Settings' },
];

export async function NavigationServer() {
  const user = await getCurrentUser();
  const displayUser = user ? {
    displayName: user.name || user.email || 'User',
    email: user.email || '',
    role: user.role || 'user',
  } : null;

  return (
    <nav className="border-b bg-background/50 backdrop-blur supports-[backdrop-filter]:bg-background/60">
      <div className="container mx-auto px-4">
        <div className="flex h-16 items-center justify-between">
          {/* Logo */}
          <Link href="/" className="flex items-center space-x-2">
            <img 
              src="/logo.png" 
              alt="EPSX" 
              className="h-8 w-auto object-contain"
            />
          </Link>

          {/* Desktop Navigation */}
          <div className="hidden md:flex items-center space-x-8">
            {/* Public menu items */}
            {publicMenuItems.map((item) => (
              <Link
                key={item.key}
                href={item.href}
                className="flex items-center space-x-1 text-sm font-medium text-muted-foreground hover:text-foreground transition-colors"
              >
                {iconMap[item.key as keyof typeof iconMap]}
                <span>{item.label}</span>
              </Link>
            ))}

            {/* User menu items (only if authenticated) */}
            {displayUser && userMenuItems.map((item) => (
              <Link
                key={item.key}
                href={item.href}
                className="flex items-center space-x-1 text-sm font-medium text-muted-foreground hover:text-foreground transition-colors"
              >
                {iconMap[item.key as keyof typeof iconMap]}
                <span>{item.label}</span>
              </Link>
            ))}
          </div>

          {/* User section - this would need client component for interactivity */}
          <div className="flex items-center space-x-4">
            {displayUser ? (
              <div className="flex items-center space-x-2">
                <span className="text-sm text-muted-foreground">Welcome, {displayUser.displayName}</span>
                {/* Interactive logout button would be in client component */}
              </div>
            ) : (
              <Link
                href="/login"
                className="px-4 py-2 text-sm font-medium bg-primary text-primary-foreground rounded-md hover:bg-primary/90 transition-colors"
              >
                Connect
              </Link>
            )}
          </div>
        </div>
      </div>
    </nav>
  );
}