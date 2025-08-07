'use client';

import { ChevronRight, Home } from 'lucide-react';
import Link from 'next/link';
import { usePathname } from 'next/navigation';
import { useState, useEffect } from 'react';

// Route configuration with proper titles and paths
const routeConfig: Record<string, { title: string; href: string }> = {
  '/': { title: 'Dashboard', href: '/' },
  '/users': { title: 'User Management', href: '/users' },
  '/users/permissions': { title: 'User Permissions', href: '/users/permissions' },
  '/users/roles': { title: 'User Roles', href: '/users/roles' },
  '/modules': { title: 'Module Management', href: '/modules' },
  '/developer-portal': { title: 'Developer Portal', href: '/developer-portal' },
  '/analytics': { title: 'Analytics Dashboard', href: '/analytics' },
  '/stock-ranking-packages': { title: 'Stock Ranking Packages', href: '/stock-ranking-packages' },
  '/database': { title: 'Database Management', href: '/database' },
  '/settings': { title: 'General Settings', href: '/settings' },
  '/billing': { title: 'Billing Management', href: '/billing' },
  '/iam': { title: 'IAM Management', href: '/iam' },
  '/permission-profiles': { title: 'Permission Profiles', href: '/permission-profiles' },
  '/permission-profiles/assign': { title: 'Assign Profiles', href: '/permission-profiles/assign' },
  '/access-denied': { title: 'Access Denied', href: '/access-denied' },
  '/unauthorized': { title: 'Unauthorized', href: '/unauthorized' },
  '/request-access': { title: 'Request Access', href: '/request-access' },
  '/login': { title: 'Admin Login', href: '/login' },
  '/docs/api': { title: 'API Documentation', href: '/docs/api' },
};

// Dynamic route patterns for user profiles
const dynamicRoutePatterns = [
  {
    pattern: /^\/users\/([^\/]+)$/,
    getTitle: (match: RegExpMatchArray, displayName?: string) => {
      if (displayName) {
        return displayName;
      }
      // Clean up user ID for display
      const userId = match[1];
      return userId.startsWith('user-') ? `User Profile` : `User: ${userId}`;
    },
    getParentPath: () => '/users'
  },
  {
    pattern: /^\/users\/([^\/]+)\/(overview|permissions|modules|packages|activity)$/,
    getTitle: (match: RegExpMatchArray) => {
      const sections: Record<string, string> = {
        overview: 'Overview',
        permissions: 'Permissions',
        modules: 'Modules',
        packages: 'Packages', 
        activity: 'Activity History'
      };
      return sections[match[2]] || match[2].charAt(0).toUpperCase() + match[2].slice(1);
    },
    getParentPath: (match: RegExpMatchArray) => `/users/${match[1]}`,
    getParentTitle: (match: RegExpMatchArray, displayName?: string) => {
      if (displayName) {
        return displayName;
      }
      const userId = match[1];
      return userId.startsWith('user-') ? 'User Profile' : `User: ${userId}`;
    }
  }
];

interface BreadcrumbItem {
  title: string;
  href: string;
  isCurrentPage?: boolean;
}

export function Breadcrumb() {
  const pathname = usePathname();
  const [userDisplayName, setUserDisplayName] = useState<string | null>(null);
  const [isLoadingUserName, setIsLoadingUserName] = useState(false);

  // Extract user ID from current path if it's a user route
  const userIdMatch = pathname.match(/^\/users\/([^\/]+)/);
  const currentUserId = userIdMatch ? userIdMatch[1] : null;

  // Fetch user data for display name when on user routes
  useEffect(() => {
    if (currentUserId && !userDisplayName && !isLoadingUserName) {
      setIsLoadingUserName(true);
      
      // Mock user data fetch for demonstration
      // In a real implementation, this would call getUnifiedUserData(currentUserId)
      setTimeout(() => {
        // Simulate different user names based on user ID
        const mockUsers: Record<string, string> = {
          'user-123': 'John Doe',
          'user-456': 'Jane Smith',
          'user-789': 'Admin User',
        };
        
        const displayName = mockUsers[currentUserId] || 
          (currentUserId.startsWith('user-') ? 'User Profile' : `User: ${currentUserId}`);
        
        setUserDisplayName(displayName);
        setIsLoadingUserName(false);
      }, 500); // Simulate network delay
    }
  }, [currentUserId, userDisplayName, isLoadingUserName]);

  const generateBreadcrumbs = (): BreadcrumbItem[] => {
    const breadcrumbs: BreadcrumbItem[] = [];

    // Handle root path (Dashboard)
    if (pathname === '/') {
      breadcrumbs.push({ title: 'Admin', href: '/' });
      breadcrumbs.push({ title: 'Dashboard', href: '/', isCurrentPage: true });
      return breadcrumbs;
    }

    // For all other paths, start with Admin as root
    breadcrumbs.push({ title: 'Admin', href: '/' });

    // Check dynamic routes first
    for (const route of dynamicRoutePatterns) {
      const match = pathname.match(route.pattern);
      if (match) {
        // Add User Management parent for user routes
        const isUserRoute = pathname.startsWith('/users/');
        if (isUserRoute) {
          breadcrumbs.push({
            title: 'User Management',
            href: '/users'
          });
        }

        // Add parent breadcrumb if it exists (for sub-pages)
        const parentPath = route.getParentPath ? route.getParentPath(match) : null;
        if (parentPath && parentPath !== '/users') {
          // This is a user sub-page, add the user profile breadcrumb
          const parentTitle = (route as any).getParentTitle 
            ? (route as any).getParentTitle(match, userDisplayName)
            : userDisplayName || 'User Profile';
          
          breadcrumbs.push({
            title: parentTitle,
            href: parentPath
          });
        }
        
        // Add current page
        const title = route.getTitle.length > 1 
          ? (route.getTitle as any)(match, userDisplayName) 
          : route.getTitle(match);
        
        breadcrumbs.push({
          title,
          href: pathname,
          isCurrentPage: true
        });
        return breadcrumbs;
      }
    }

    // Check static routes
    if (routeConfig[pathname]) {
      breadcrumbs.push({
        title: routeConfig[pathname].title,
        href: routeConfig[pathname].href,
        isCurrentPage: true
      });
      return breadcrumbs;
    }

    // Fallback: parse pathname segments
    const segments = pathname.split('/').filter(Boolean);
    
    segments.forEach((segment, index) => {
      const currentPath = '/' + segments.slice(0, index + 1).join('/');
      const isLast = index === segments.length - 1;
      
      // Try to find in route config
      const route = routeConfig[currentPath];
      if (route) {
        breadcrumbs.push({
          title: route.title,
          href: route.href,
          isCurrentPage: isLast
        });
      } else {
        // Fallback to segment name with proper formatting
        const title = segment
          .split('-')
          .map(word => word.charAt(0).toUpperCase() + word.slice(1))
          .join(' ');
        
        breadcrumbs.push({
          title,
          href: currentPath,
          isCurrentPage: isLast
        });
      }
    });

    return breadcrumbs;
  };

  const breadcrumbs = generateBreadcrumbs();

  return (
    <nav aria-label="Breadcrumb" className="flex items-center gap-2 text-sm">
      <Home className="h-4 w-4 text-gray-500 dark:text-gray-400" />
      
      {breadcrumbs.map((item, index) => (
        <div key={`breadcrumb-${index}-${item.title.replace(/\s+/g, '-').toLowerCase()}`} className="flex items-center gap-2">
          {index > 0 && (
            <ChevronRight className="h-3 w-3 text-gray-400 dark:text-gray-500" />
          )}
          
          {item.isCurrentPage ? (
            <span className="text-blue-600 dark:text-blue-400 font-medium flex items-center gap-2">
              {item.title}
              {isLoadingUserName && currentUserId && item.title.includes('User') && (
                <div className="animate-spin h-3 w-3 border border-blue-400 border-t-transparent rounded-full"></div>
              )}
            </span>
          ) : (
            <Link
              href={item.href}
              className="text-gray-600 dark:text-gray-400 hover:text-gray-900 dark:hover:text-gray-200 transition-colors duration-150 flex items-center gap-2"
            >
              {item.title}
              {isLoadingUserName && currentUserId && item.title.includes('User') && (
                <div className="animate-spin h-3 w-3 border border-gray-400 border-t-transparent rounded-full"></div>
              )}
            </Link>
          )}
        </div>
      ))}
    </nav>
  );
}