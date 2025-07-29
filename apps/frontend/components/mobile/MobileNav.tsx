'use client';

import { useState } from 'react';
import Link from 'next/link';
import { usePathname } from 'next/navigation';
import { Button } from '@/components/ui/button';
import { Badge } from '@/components/ui/badge';
import { Sheet, SheetContent, SheetTrigger } from '@/components/ui/sheet';
import { 
  Menu, 
  X, 
  Home, 
  BarChart3, 
  TrendingUp, 
  Search, 
  Bell, 
  Settings, 
  User,
  CreditCard,
  Shield,
  HelpCircle,
  LogOut
} from 'lucide-react';

interface NavItem {
  label: string;
  href: string;
  icon: React.ComponentType<any>;
  badge?: string;
  isActive?: boolean;
}

const MAIN_NAV_ITEMS: NavItem[] = [
  { label: 'Home', href: '/', icon: Home },
  { label: 'Analytics', href: '/analytics', icon: BarChart3 },
  { label: 'EPS Analysis', href: '/analytics/eps', icon: TrendingUp },
  { label: 'Pattern Recognition', href: '/analytics/pattern-recognition', icon: Search },
  { label: 'Notifications', href: '/notifications', icon: Bell, badge: '3' }
];

const ACCOUNT_NAV_ITEMS: NavItem[] = [
  { label: 'Profile', href: '/profile', icon: User },
  { label: 'Billing', href: '/billing', icon: CreditCard },
  { label: 'Settings', href: '/settings', icon: Settings },
  { label: 'Admin', href: '/admin', icon: Shield },
  { label: 'Help', href: '/help', icon: HelpCircle }
];

interface MobileNavProps {
  user?: {
    name?: string;
    email?: string;
    avatar?: string;
  };
  onLogout?: () => void;
}

export function MobileNav({ user, onLogout }: MobileNavProps) {
  const [open, setOpen] = useState(false);
  const pathname = usePathname();

  const isItemActive = (href: string) => {
    if (href === '/') return pathname === '/';
    return pathname.startsWith(href);
  };

  const handleNavClick = () => {
    setOpen(false);
  };

  return (
    <Sheet open={open} onOpenChange={setOpen}>
      <SheetTrigger asChild>
        <Button 
          variant="ghost" 
          size="sm" 
          className="md:hidden p-2"
          aria-label="Open navigation menu"
        >
          <Menu className="h-5 w-5" />
        </Button>
      </SheetTrigger>
      
      <SheetContent side="left" className="w-80 p-0">
        <div className="flex flex-col h-full">
          {/* Header */}
          <div className="p-4 border-b bg-muted/20">
            <div className="flex items-center justify-between">
              <Link href="/" onClick={handleNavClick}>
                <h2 className="text-lg font-semibold">EPSX</h2>
              </Link>
              <Button 
                variant="ghost" 
                size="sm" 
                onClick={() => setOpen(false)}
                className="p-1"
              >
                <X className="h-4 w-4" />
              </Button>
            </div>
            
            {user && (
              <div className="mt-3 flex items-center gap-3">
                <div className="w-8 h-8 rounded-full bg-primary/20 flex items-center justify-center">
                  <User className="h-4 w-4" />
                </div>
                <div className="flex-1 min-w-0">
                  <p className="text-sm font-medium truncate">
                    {user.name || 'User'}
                  </p>
                  <p className="text-xs text-muted-foreground truncate">
                    {user.email}
                  </p>
                </div>
              </div>
            )}
          </div>

          {/* Main Navigation */}
          <div className="flex-1 overflow-y-auto py-4">
            <div className="px-3">
              <div className="mb-4">
                <h3 className="px-3 text-xs font-medium text-muted-foreground uppercase tracking-wider mb-2">
                  Main
                </h3>
                <nav className="space-y-1">
                  {MAIN_NAV_ITEMS.map((item) => {
                    const isActive = isItemActive(item.href);
                    
                    return (
                      <Link
                        key={item.href}
                        href={item.href}
                        onClick={handleNavClick}
                        className={`
                          flex items-center gap-3 px-3 py-2 rounded-lg text-sm transition-colors
                          ${isActive 
                            ? 'bg-primary/10 text-primary font-medium' 
                            : 'text-muted-foreground hover:text-foreground hover:bg-muted'
                          }
                        `}
                      >
                        <item.icon className="h-4 w-4 flex-shrink-0" />
                        <span className="flex-1">{item.label}</span>
                        {item.badge && (
                          <Badge variant="secondary" className="text-xs">
                            {item.badge}
                          </Badge>
                        )}
                      </Link>
                    );
                  })}
                </nav>
              </div>

              <div className="mb-4">
                <h3 className="px-3 text-xs font-medium text-muted-foreground uppercase tracking-wider mb-2">
                  Account
                </h3>
                <nav className="space-y-1">
                  {ACCOUNT_NAV_ITEMS.map((item) => {
                    const isActive = isItemActive(item.href);
                    
                    return (
                      <Link
                        key={item.href}
                        href={item.href}
                        onClick={handleNavClick}
                        className={`
                          flex items-center gap-3 px-3 py-2 rounded-lg text-sm transition-colors
                          ${isActive 
                            ? 'bg-primary/10 text-primary font-medium' 
                            : 'text-muted-foreground hover:text-foreground hover:bg-muted'
                          }
                        `}
                      >
                        <item.icon className="h-4 w-4 flex-shrink-0" />
                        <span className="flex-1">{item.label}</span>
                      </Link>
                    );
                  })}
                </nav>
              </div>
            </div>
          </div>

          {/* Footer */}
          {user && onLogout && (
            <div className="p-4 border-t">
              <Button 
                variant="ghost" 
                className="w-full justify-start text-muted-foreground hover:text-foreground"
                onClick={() => {
                  onLogout();
                  handleNavClick();
                }}
              >
                <LogOut className="h-4 w-4 mr-3" />
                Sign Out
              </Button>
            </div>
          )}
        </div>
      </SheetContent>
    </Sheet>
  );
}