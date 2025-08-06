'use client';

import { useState } from 'react';
import Link from 'next/link';
import { Button } from '@/components/ui/button';
import { Badge } from '@/components/ui/badge';
import { MobileNav } from './MobileNav';
import { 
  Bell, 
  Search, 
  User,
  Settings as _Settings
} from 'lucide-react';

interface MobileHeaderProps {
  title?: string;
  user?: {
    name?: string;
    email?: string;
    avatar?: string;
  };
  showSearch?: boolean;
  showNotifications?: boolean;
  notificationCount?: number;
  onLogout?: () => void;
}

export function MobileHeader({ 
  title, 
  user, 
  showSearch = true, 
  showNotifications = true,
  notificationCount = 0,
  onLogout 
}: MobileHeaderProps) {
  const [searchOpen, setSearchOpen] = useState(false);

  return (
    <>
      <header className="md:hidden sticky top-0 z-40 w-full border-b bg-background/95 backdrop-blur supports-[backdrop-filter]:bg-background/60">
        <div className="flex h-14 items-center px-4">
          {/* Left side - Menu and Title */}
          <div className="flex items-center gap-3 flex-1">
            <MobileNav user={user} onLogout={onLogout} />
            
            {title ? (
              <h1 className="text-lg font-semibold truncate">{title}</h1>
            ) : (
              <Link href="/" className="text-lg font-semibold">
                EPSX
              </Link>
            )}
          </div>

          {/* Right side - Actions */}
          <div className="flex items-center gap-2">
            {showSearch && (
              <Button 
                variant="ghost" 
                size="sm" 
                className="p-2"
                onClick={() => setSearchOpen(true)}
                aria-label="Search"
              >
                <Search className="h-4 w-4" />
              </Button>
            )}

            {showNotifications && (
              <Button 
                variant="ghost" 
                size="sm" 
                className="p-2 relative"
                asChild
              >
                <Link href="/notifications" aria-label="Notifications">
                  <Bell className="h-4 w-4" />
                  {notificationCount > 0 && (
                    <Badge 
                      variant="destructive" 
                      className="absolute -top-1 -right-1 h-5 w-5 p-0 flex items-center justify-center text-xs"
                    >
                      {notificationCount > 9 ? '9+' : notificationCount}
                    </Badge>
                  )}
                </Link>
              </Button>
            )}

            {user && (
              <Button 
                variant="ghost" 
                size="sm" 
                className="p-2"
                asChild
              >
                <Link href="/profile" aria-label="Profile">
                  <User className="h-4 w-4" />
                </Link>
              </Button>
            )}
          </div>
        </div>
      </header>

      {/* Search Overlay */}
      {searchOpen && (
        <div className="md:hidden fixed inset-0 z-50 bg-background">
          <div className="flex h-14 items-center px-4 border-b">
            <div className="flex-1 mr-3">
              <input
                type="search"
                placeholder="Search stocks, patterns..."
                className="w-full px-3 py-2 text-sm bg-muted rounded-md border-0 ring-1 ring-border focus:ring-2 focus:ring-primary outline-none"
                autoFocus
              />
            </div>
            <Button 
              variant="ghost" 
              size="sm"
              onClick={() => setSearchOpen(false)}
            >
              Cancel
            </Button>
          </div>
          
          <div className="p-4">
            <div className="space-y-3">
              <div className="text-sm text-muted-foreground">Recent searches</div>
              
              <div className="space-y-2">
                {['AAPL', 'TSLA', 'MSFT', 'GOOGL'].map((symbol) => (
                  <button
                    key={symbol}
                    className="flex items-center w-full p-3 text-left rounded-lg bg-muted hover:bg-muted/80 transition-colors"
                    onClick={() => setSearchOpen(false)}
                  >
                    <Search className="h-4 w-4 mr-3 text-muted-foreground" />
                    <span className="font-medium">{symbol}</span>
                  </button>
                ))}
              </div>

              <div className="text-sm text-muted-foreground mt-6">Popular patterns</div>
              
              <div className="space-y-2">
                {['Bull Flag', 'Head and Shoulders', 'Triangle Breakout'].map((pattern) => (
                  <button
                    key={pattern}
                    className="flex items-center w-full p-3 text-left rounded-lg bg-muted hover:bg-muted/80 transition-colors"
                    onClick={() => setSearchOpen(false)}
                  >
                    <Search className="h-4 w-4 mr-3 text-muted-foreground" />
                    <span>{pattern}</span>
                  </button>
                ))}
              </div>
            </div>
          </div>
        </div>
      )}
    </>
  );
}