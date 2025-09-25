'use client';

import { useState } from 'react';
import Link from 'next/link';
import { usePathname } from 'next/navigation';
import { ChevronDown, Lock, Shield, Crown, AlertCircle } from 'lucide-react';
import { VisualPermissionGuard } from '@/components/guards/VisualPermissionGuard';
import { PermissionRequestModal } from '@/components/modals/PermissionRequestModal';
import { useVisualPermission } from '@/hooks/useVisualPermission';
// Tooltip components not available

interface MenuItem {
  id: string;
  label: string;
  href?: string;
  icon?: React.ReactNode;
  description: string;
  requiredPermission?: string;
  requiredRole?: string;
  items?: MenuItem[];
  type: 'single' | 'group';
}

interface RestrictedMenuItemProps {
  item: MenuItem;
  isExpanded?: boolean;
  onToggle?: () => void;
  depth?: number;
  sidebarCollapsed?: boolean;
  onRequestAccess?: (data: any) => Promise<void>;
}

export function RestrictedMenuItem({
  item,
  isExpanded = false,
  onToggle,
  depth = 0,
  sidebarCollapsed = false,
  onRequestAccess
}: RestrictedMenuItemProps) {
  const pathname = usePathname();
  const [showPermissionModal, setShowPermissionModal] = useState(false);
  
  const permission = useVisualPermission({
    permission: item.requiredPermission,
    customCheck: item.requiredRole ? (user) => {
      if (!user?.permissions) return false;
      const permissions = Array.isArray(user.permissions) ? user.permissions : [];
      
      // Check for admin wildcard
      if (permissions.includes('admin:*:*')) return true;
      
      // Check for role-based permission
      if (item.requiredRole === 'super_admin') {
        return permissions.some((p: string) => p.includes('admin:') && p.includes('manage'));
      }
      
      return permissions.some((p: string) => p.startsWith('admin:'));
    } : undefined
  });

  const isActive = item.href && pathname === item.href;
  const hasActiveChild = item.items?.some(child => child.href && pathname.startsWith(child.href));

  // Handle click for restricted items
  const handleRestrictedClick = (e: React.MouseEvent) => {
    e.preventDefault();
    e.stopPropagation();
    setShowPermissionModal(true);
  };

  // Render restriction icon
  const getRestrictionIcon = () => {
    if (item.requiredRole === 'super_admin') return <Shield className="h-3 w-3" />;
    if (item.requiredRole === 'system_admin') return <Crown className="h-3 w-3" />;
    return <Lock className="h-3 w-3" />;
  };

  // Get restriction reason
  const getRestrictionReason = () => {
    if (item.requiredPermission) return `Requires permission: ${item.requiredPermission}`;
    if (item.requiredRole) return `Requires ${item.requiredRole.replace('_', ' ')} role`;
    return 'Access restricted';
  };

  // Render single menu item
  if (item.type === 'single') {
    const content = (
      <div 
        className={`relative flex items-center space-x-3 px-3 py-2 rounded-md transition-all duration-200 group ${
          permission.hasPermission 
            ? isActive 
              ? 'bg-primary text-primary-foreground shadow-sm' 
              : 'hover:bg-muted text-foreground hover:text-foreground'
            : 'nav-item-restricted cursor-pointer'
        } ${depth > 0 ? 'ml-4' : ''}`}
        onClick={!permission.hasPermission ? handleRestrictedClick : undefined}
      >
        {/* Icon */}
        {item.icon && (
          <div className={`flex-shrink-0 ${permission.hasPermission ? '' : 'opacity-60'}`}>
            {item.icon}
          </div>
        )}
        
        {/* Label */}
        <span className={`flex-1 text-sm font-medium ${
          permission.hasPermission ? '' : 'text-muted-foreground'
        } ${sidebarCollapsed ? 'sr-only' : ''}`}>
          {item.label}
        </span>
        
        {/* Restriction indicator */}
        {!permission.hasPermission && (
          <div className="flex items-center space-x-1">
            {!sidebarCollapsed && (
              <span className="text-xs text-muted-foreground">
                {item.requiredRole?.replace('_', ' ') || 'Restricted'}
              </span>
            )}
            <div className="text-muted-foreground">
              {getRestrictionIcon()}
            </div>
          </div>
        )}
        
        {/* Active indicator */}
        {permission.hasPermission && isActive && (
          <div className="absolute -left-1 top-1/2 transform -translate-y-1/2 w-1 h-6 bg-primary-foreground rounded-r" />
        )}
      </div>
    );

    // Add title attribute for collapsed sidebar
    if (sidebarCollapsed) {
      const tooltipText = `${item.label} - ${item.description}${!permission.hasPermission ? ` - ${getRestrictionReason()}` : ''}`;
      const wrappedContent = (
        <div title={tooltipText}>
          {content}
        </div>
      );
      
      return permission.hasPermission && item.href ? (
        <Link href={item.href}>{wrappedContent}</Link>
      ) : (
        wrappedContent
      );
    }

    // Regular menu item
    return (
      <>
        {permission.hasPermission && item.href ? (
          <Link href={item.href}>{content}</Link>
        ) : (
          content
        )}
        
        {/* Permission request modal */}
        <PermissionRequestModal
          isOpen={showPermissionModal}
          onClose={() => setShowPermissionModal(false)}
          requiredPermission={item.requiredPermission}
          requiredRole={item.requiredRole}
          itemLabel={item.label}
          itemDescription={item.description}
        />
      </>
    );
  }

  // Render group menu item
  if (item.type === 'group' && item.items) {
    // Filter items to show (all items, but mark restricted ones)
    const allItems = item.items.map(child => ({
      ...child,
      isRestricted: !useVisualPermission({ 
        permission: child.requiredPermission,
        customCheck: child.requiredRole ? (user) => {
          if (!user?.permissions) return false;
          const permissions = Array.isArray(user.permissions) ? user.permissions : [];
          return permissions.includes('admin:*:*') || permissions.some((p: string) => p.startsWith('admin:'));
        } : undefined
      }).hasPermission
    }));

    const groupContent = (
      <div>
        {/* Group header */}
        <div 
          className={`relative flex items-center space-x-3 px-3 py-2 rounded-md transition-all duration-200 group cursor-pointer ${
            permission.hasPermission 
              ? hasActiveChild 
                ? 'bg-muted text-foreground' 
                : 'hover:bg-muted text-foreground hover:text-foreground'
              : 'nav-item-restricted'
          }`}
          onClick={permission.hasPermission && !sidebarCollapsed ? onToggle : handleRestrictedClick}
        >
          {/* Icon */}
          {item.icon && (
            <div className={`flex-shrink-0 ${permission.hasPermission ? '' : 'opacity-60'}`}>
              {item.icon}
            </div>
          )}
          
          {/* Label */}
          <span className={`flex-1 text-sm font-medium ${
            permission.hasPermission ? '' : 'text-muted-foreground'
          } ${sidebarCollapsed ? 'sr-only' : ''}`}>
            {item.label}
          </span>
          
          {/* Restriction indicator or expand arrow */}
          {!permission.hasPermission ? (
            <div className="flex items-center space-x-1">
              {!sidebarCollapsed && (
                <span className="text-xs text-muted-foreground">
                  {item.requiredRole?.replace('_', ' ') || 'Restricted'}
                </span>
              )}
              <div className="text-muted-foreground">
                {getRestrictionIcon()}
              </div>
            </div>
          ) : !sidebarCollapsed && (
            <ChevronDown 
              className={`h-4 w-4 transition-transform duration-200 ${
                isExpanded ? 'transform rotate-180' : ''
              }`} 
            />
          )}
        </div>
        
        {/* Submenu items */}
        {permission.hasPermission && isExpanded && !sidebarCollapsed && allItems.length > 0 && (
          <div className="ml-4 mt-1 space-y-1">
            {allItems.map((child) => (
              <RestrictedMenuItem
                key={child.id}
                item={child}
                depth={depth + 1}
                sidebarCollapsed={sidebarCollapsed}
                onRequestAccess={onRequestAccess}
              />
            ))}
          </div>
        )}
      </div>
    );

    // Add title attribute for collapsed sidebar
    if (sidebarCollapsed) {
      const childList = allItems.slice(0, 3).map(child => child.label).join(', ');
      const tooltipText = `${item.label} - ${item.description}${!permission.hasPermission ? ` - ${getRestrictionReason()}` : ''}${allItems.length > 0 ? ` - Includes: ${childList}${allItems.length > 3 ? '...' : ''}` : ''}`;
      
      return (
        <>
          <div title={tooltipText}>
            {groupContent}
          </div>
          
          {/* Permission request modal */}
          <PermissionRequestModal
            isOpen={showPermissionModal}
            onClose={() => setShowPermissionModal(false)}
            requiredPermission={item.requiredPermission}
            requiredRole={item.requiredRole}
            itemLabel={item.label}
            itemDescription={item.description}
          />
        </>
      );
    }

    return (
      <>
        {groupContent}
        
        {/* Permission request modal */}
        <PermissionRequestModal
          isOpen={showPermissionModal}
          onClose={() => setShowPermissionModal(false)}
          requiredPermission={item.requiredPermission}
          requiredRole={item.requiredRole}
          itemLabel={item.label}
          itemDescription={item.description}
        />
      </>
    );
  }

  return null;
}