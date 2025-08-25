// Copied and adapted from frontend/components/nav.tsx
'use client';

import { ThemeSwitch } from '@/components/ui/ThemeSwitch';
import { useAuth } from '@/lib/auth';
import { BarChart, Home, LogIn, LogOut, DollarSign, Settings, Users, Code, Shield, FileText, Activity, Database, ShieldCheck, AlertTriangle, Lock, Bell, Gauge, Zap, Newspaper } from 'lucide-react';
import _Image from 'next/image';
import Link from 'next/link';
import { usePathname } from 'next/navigation';
import { useState } from 'react';

interface NavItem {
  href: string;
  label: string;
  icon: React.ComponentType<any>;
  requiredModules?: string[]; // Admin modules required to see this nav item
  description?: string;
}

const navItems: NavItem[] = [
  { 
    href: '/', 
    label: 'Home', 
    icon: Home,
    description: 'Admin dashboard overview'
  },
  { 
    href: '/users', 
    label: 'Users', 
    icon: Users,
    requiredModules: ['user_operations'],
    description: 'Manage user accounts and profiles'
  },
  { 
    href: '/analytics', 
    label: 'Analytics', 
    icon: BarChart,
    requiredModules: ['analytics_specialist'],
    description: 'View analytics and reporting'
  },
  { 
    href: '/news', 
    label: 'News', 
    icon: Newspaper,
    requiredModules: ['content_manager'],
    description: 'Manage news articles and content'
  },
  { 
    href: '/billing', 
    label: 'Billing', 
    icon: DollarSign,
    requiredModules: ['billing_admin'],
    description: 'Manage billing and subscriptions'
  },
  { 
    href: '/admin-roles', 
    label: 'Admin Roles', 
    icon: Shield,
    requiredModules: ['role_policy_manager'],
    description: 'Assign and manage admin roles'
  },
  { 
    href: '/modules', 
    label: 'System Config', 
    icon: Settings,
    requiredModules: ['system_admin'],
    description: 'Configure system settings and modules'
  },
  { 
    href: '/developer-portal', 
    label: 'Developer Portal', 
    icon: Code,
    requiredModules: ['developer_relations'],
    description: 'Manage developer tools and integrations'
  },
  { 
    href: '/support', 
    label: 'Support', 
    icon: FileText,
    requiredModules: ['support_specialist'],
    description: 'Handle support requests and documentation'
  },
  { 
    href: '/compliance', 
    label: 'Compliance', 
    icon: Activity,
    requiredModules: ['compliance_audit'],
    description: 'Monitor compliance and audit activities'
  },
  { 
    href: '/system-monitor', 
    label: 'System Monitor', 
    icon: Database,
    requiredModules: ['module_coordinator', 'system_admin'],
    description: 'Monitor system health and performance'
  },
  { 
    href: '/security', 
    label: 'Security', 
    icon: ShieldCheck,
    requiredModules: ['security_management', 'audit_logs'],
    description: 'Security monitoring and threat detection'
  },
];

export function Navigation() {
  const pathname = usePathname();
  const { user, isLoading, logout } = useAuth();
  const [_mobileOpen, _setMobileOpen] = useState(false);
  const [isSigningOut, setIsSigningOut] = useState(false);

  const loading = isLoading;

  const handleSignOut = async () => {
    setIsSigningOut(true);
    try {
      await logout();
    } catch (error) {
      console.error('Logout failed:', error);
    } finally {
      setIsSigningOut(false);
    }
  };

  // Filter navigation items based on user's admin modules
  const filteredItems = navItems.filter(item => {
    if (!item.requiredModules || item.requiredModules.length === 0) {
      return true; // Show items that don't require specific modules
    }
    return item.requiredModules.some(module => 
      user?.admin_modules?.includes(module)
    );
  });

  if (loading) {
    return (
      <nav className="sticky top-0 z-50 w-full border-b border-border/20 bg-background/80 backdrop-blur-xl supports-[backdrop-filter]:bg-background/60 shadow-lg pancake-card-gradient">
        <div className="container mx-auto px-4 lg:px-6">
          <div className="flex h-20 items-center justify-between">
            <div className="flex items-center gap-3">
              <div className="flex h-8 w-8 items-center justify-center rounded-lg bg-gradient-to-r from-orange-500 to-yellow-500">
                <span className="text-lg font-bold text-white">E</span>
              </div>
              <div className="flex flex-col">
                <span className="text-2xl font-bold bg-gradient-to-r from-orange-500 to-yellow-500 bg-clip-text text-transparent">
                  EPSX
                </span>
                <span className="text-xs text-muted-foreground font-medium">
                  Admin Dashboard
                </span>
              </div>
            </div>
            <div className="flex items-center gap-4">
              <div className="animate-pulse bg-muted h-10 w-20 rounded-full"></div>
            </div>
          </div>
        </div>
      </nav>
    );
  }

  return (
    <nav className="sticky top-0 z-50 w-full border-b border-border/20 bg-background/80 backdrop-blur-xl supports-[backdrop-filter]:bg-background/60 shadow-lg pancake-card-gradient">
      <div className="container mx-auto px-4 lg:px-6">
        <div className="flex h-20 items-center justify-between">
          {/* Enhanced Brand Logo */}
          <Link href="/" className="flex items-center gap-3 group">
            <div className="flex h-8 w-8 items-center justify-center rounded-lg bg-gradient-to-r from-orange-500 to-yellow-500 transition-all group-hover:scale-110">
              <span className="text-lg font-bold text-white">E</span>
            </div>
            <div className="flex flex-col">
              <span className="text-2xl font-bold bg-gradient-to-r from-orange-500 to-yellow-500 bg-clip-text text-transparent group-hover:from-yellow-500 group-hover:to-orange-500 transition-all">
                EPSX
              </span>
              <span className="text-xs text-muted-foreground font-medium">
                Admin Dashboard
              </span>
            </div>
          </Link>

          <div className="flex items-center gap-4">
            <ThemeSwitch />
            {filteredItems.map((item, idx) => (
              <Link
                key={item.href}
                href={item.href}
                title={item.description || item.label}
                className={`group flex items-center gap-2 px-4 py-2 rounded-full font-medium transition-all duration-300 hover:scale-105 relative ${
                  pathname === item.href
                    ? 'bg-gradient-to-r from-orange-500 to-yellow-500 text-white shadow-lg'
                    : 'hover:bg-orange-500/10 hover:text-orange-500'
                }`}
              >
                {idx === 0 && (
                  <span
                    className={`w-2 h-2 rounded-full transition-all ${
                      pathname === item.href
                        ? 'bg-white'
                        : 'bg-orange-500 group-hover:scale-125'
                    }`}
                  ></span>
                )}
                <item.icon className="w-5 h-5 transition-transform group-hover:scale-110" />
                <span>{item.label}</span>
                
                {/* Module requirement indicator for development */}
                {process.env.NODE_ENV === 'development' && item.requiredModules && (
                  <span 
                    className="absolute -top-1 -right-1 w-3 h-3 bg-green-500 rounded-full text-xs flex items-center justify-center text-white font-bold opacity-0 group-hover:opacity-100 transition-opacity"
                    title={`Requires: ${item.requiredModules.join(', ')}`}
                  >
                    ✓
                  </span>
                )}
              </Link>
            ))}
            {user ? (
              <>
                <span className="text-sm text-muted-foreground">{user.email}</span>
                <button
                  onClick={handleSignOut}
                  disabled={isSigningOut}
                  className="pancake-button-secondary flex items-center gap-2 text-sm font-medium disabled:opacity-50"
                >
                  <LogOut className="w-4 h-4" />
                  {isSigningOut ? 'Signing out...' : 'Logout'}
                </button>
              </>
            ) : (
              <Link href="/login">
                <button className="pancake-button flex items-center gap-2 text-sm font-medium">
                  <LogIn className="w-4 h-4" />
                  Login
                </button>
              </Link>
            )}
          </div>
        </div>
      </div>
    </nav>
  );
}
