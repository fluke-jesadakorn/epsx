import { getSessionInfo } from '@/lib/auth-server';
import { navigationService } from '@/services/navigation.service';
import Link from 'next/link';
import { ClientNavControls } from '../client/ClientNavControls';
import { NavigationItems } from './NavigationItems';

/**
 * Server-side navigation component with minimal client hydration
 */
export async function ServerNavigation() {
  const sessionInfo = await getSessionInfo();
  const navItems = navigationService.getNavItems(sessionInfo.isAuthenticated);

  return (
    <div className="relative z-50 bg-gradient-to-r from-blue-500/10 via-purple-500/10 to-pink-500/10 border-b backdrop-blur-sm">
      <div className="flex h-20 items-center px-4 sm:px-6 justify-between max-w-7xl mx-auto">
        <div className="flex items-center gap-4 sm:gap-8">
          <Link href="/" className="flex items-center gap-2 group">
            <span className="text-2xl sm:text-3xl font-bold bg-gradient-to-r from-blue-500 via-purple-500 to-pink-500 bg-clip-text text-transparent group-hover:scale-105 transition-transform duration-300">
              EPSX
            </span>
          </Link>
          
          {/* Server-rendered navigation items */}
          <NavigationItems items={navItems} />
        </div>

        {/* Client-side controls (theme, user menu, mobile menu) */}
        <ClientNavControls 
          user={sessionInfo.isAuthenticated ? {
            email: sessionInfo.email || '',
            displayName: sessionInfo.displayName || sessionInfo.email?.split('@')[0] || ''
          } : null}
          navItems={navItems}
        />
      </div>
    </div>
  );
}