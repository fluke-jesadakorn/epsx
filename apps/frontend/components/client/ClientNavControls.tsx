'use client';

import { useState, useEffect } from 'react';
import { useRouter } from 'next/navigation';
import Link from 'next/link';
import { Menu, User, LogOut, LogIn, Crown } from 'lucide-react';
import { OptimizedThemeToggle } from './OptimizedThemeToggle';
import { Avatar, AvatarFallback } from '@/components/ui/avatar';
import { Sheet, SheetContent, SheetHeader, SheetTitle, SheetTrigger } from '@/components/ui/sheet';
import { Tooltip, TooltipContent, TooltipProvider, TooltipTrigger } from '@/components/ui/tooltip';
// Note: This component will be replaced by server-only auth flow
import { formatLevelAsNumber, getLevelColor } from '@/lib/level-utils';
import { Button, Badge } from '@/components/ui';

interface ClientNavControlsProps {
  user: { email: string; displayName: string } | null;
  navItems: Array<{ key: string; label: string; href: string }>;
}

/**
 * Client-side navigation controls (theme, user menu, mobile)
 */
export function ClientNavControls({ user: serverUser, navItems: _navItems }: ClientNavControlsProps) {
  const [isOpen, setIsOpen] = useState(false);
  const [mounted, setMounted] = useState(false);
  const user = serverUser; // Use server user data only
  const userLevel = 'free'; // Will be retrieved from server auth
  const router = useRouter();

  useEffect(() => {
    setMounted(true);
  }, []);

  const handleLogout = async () => {
    try {
      // Redirect to OIDC logout endpoint
      router.push('/auth/logout');
    } catch (error) {
      console.error('Error signing out:', error);
    }
  };

  // Use server user data until client auth loads
  const currentUser = user || serverUser;

  if (!mounted) {
    return (
      <div className="flex items-center gap-2 sm:gap-4">
        <div className="hidden sm:block">
          <OptimizedThemeToggle />
        </div>
        
        {serverUser && (
          <Link href="/settings" className="flex items-center gap-2">
            <Avatar className="h-8 w-8 sm:h-9 sm:w-9">
              <AvatarFallback>
                <User className="h-4 w-4" />
              </AvatarFallback>
            </Avatar>
          </Link>
        )}
        
        <Button variant="ghost" size="icon" className="lg:hidden bg-background shadow-md h-8 w-8 sm:h-10 sm:w-10">
          <Menu className="h-4 w-4" />
        </Button>
      </div>
    );
  }

  return (
    <div className="flex items-center gap-2 sm:gap-4">
      <div className="hidden sm:block">
        <OptimizedThemeToggle />
      </div>

      {/* User Level Display */}
      {currentUser && (
        <TooltipProvider>
          <Tooltip>
            <TooltipTrigger asChild>
              <div className="flex items-center gap-2">
                <Badge
                  variant="secondary"
                  className={`${getLevelColor(userLevel)} border-current bg-current/10 text-current font-bold text-xs px-2 py-1`}
                >
                  <Crown className="h-3 w-3 mr-1" />
                  {formatLevelAsNumber(userLevel)}
                </Badge>
              </div>
            </TooltipTrigger>
            <TooltipContent>
              <p>Your current level: {formatLevelAsNumber(userLevel)}</p>
            </TooltipContent>
          </Tooltip>
        </TooltipProvider>
      )}

      {currentUser && (
        <TooltipProvider>
          <Tooltip>
            <TooltipTrigger asChild>
              <Link href="/settings" className="flex items-center gap-2">
                <Avatar className="h-8 w-8 sm:h-9 sm:w-9">
                  <AvatarFallback>
                    <User className="h-4 w-4" />
                  </AvatarFallback>
                </Avatar>
                <span className="hidden lg:inline text-muted-foreground hover:text-primary text-sm">
                  Settings
                </span>
              </Link>
            </TooltipTrigger>
            <TooltipContent>
              <p>{currentUser.email}</p>
            </TooltipContent>
          </Tooltip>
        </TooltipProvider>
      )}

      {/* Mobile Menu */}
      <Sheet open={isOpen} onOpenChange={setIsOpen}>
        <SheetTrigger asChild className="lg:hidden">
          <Button variant="ghost" size="icon" className="bg-background shadow-md h-8 w-8 sm:h-10 sm:w-10">
            <Menu className="h-4 w-4" />
          </Button>
        </SheetTrigger>
        <SheetContent side="right" className="w-[280px] sm:w-[400px]">
          <SheetHeader>
            <SheetTitle className="text-left">Menu</SheetTitle>
          </SheetHeader>
          <div className="flex flex-col gap-4 mt-6">
            {/* Mobile content will be populated by JavaScript */}
          </div>
        </SheetContent>
      </Sheet>

      {/* Desktop Logout/Login */}
      <div className="hidden lg:block">
        {currentUser ? (
          <Button
            variant="ghost"
            onClick={handleLogout}
            className="flex items-center justify-center gap-2 rounded-lg px-4 py-2 text-sm font-medium transition-all duration-200 focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring focus-visible:ring-offset-2 hover:bg-primary/10 hover:text-accent-foreground active:scale-[0.98] disabled:pointer-events-none disabled:opacity-50 text-muted-foreground hover:text-primary"
          >
            <LogOut className="h-4 w-4" />
            <span className="hidden xl:block">Logout</span>
          </Button>
        ) : (
          <Link href="/login">
            <Button
              variant="ghost"
              className="flex items-center justify-center gap-2 rounded-lg px-4 py-2 text-sm font-medium transition-all duration-200 focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring focus-visible:ring-offset-2 hover:bg-primary/10 hover:text-accent-foreground active:scale-[0.98] disabled:pointer-events-none disabled:opacity-50 text-muted-foreground hover:text-primary"
            >
              <LogIn className="h-4 w-4" />
              <span className="hidden xl:block">Login</span>
            </Button>
          </Link>
        )}
      </div>
    </div>
  );
}