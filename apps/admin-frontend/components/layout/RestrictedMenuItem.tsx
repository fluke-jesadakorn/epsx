'use client';

import { ChevronDown } from 'lucide-react';
import Link from 'next/link';
import { usePathname } from 'next/navigation';
import { useState } from 'react';

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

/**
 *
 * @param root0
 * @param root0.item
 * @param root0.isExpanded
 * @param root0.onToggle
 * @param root0.depth
 * @param root0.sidebarCollapsed
 */
export function RestrictedMenuItem({
  item,
  isExpanded = false,
  onToggle,
  depth = 0,
  sidebarCollapsed = false,
}: RestrictedMenuItemProps) {
  const pathname = usePathname();

  const isActive = item.href && pathname === item.href;
  const hasActiveChild = item.items?.some(child => child.href && pathname.startsWith(child.href));

  // Render single menu item - no permission checking
  if (item.type === 'single') {
    const content = (
      <div 
        className={`relative flex items-center space-x-3 px-3 py-2 rounded-md transition-all duration-200 group ${
          isActive 
            ? 'bg-primary text-primary-foreground shadow-sm' 
            : 'hover:bg-muted text-foreground hover:text-foreground'
        } ${depth > 0 ? 'ml-4' : ''}`}
      >
        {/* Icon */}
        {item.icon && (
          <div className="flex-shrink-0">
            {item.icon}
          </div>
        )}
        
        {/* Label */}
        <span className={`flex-1 text-sm font-medium ${sidebarCollapsed ? 'sr-only' : ''}`}>
          {item.label}
        </span>
        
        {/* Active indicator */}
        {isActive && (
          <div className="absolute -left-1 top-1/2 transform -translate-y-1/2 w-1 h-6 bg-primary-foreground rounded-r" />
        )}
      </div>
    );

    // Add title attribute for collapsed sidebar
    if (sidebarCollapsed) {
      const tooltipText = `${item.label} - ${item.description}`;
      const wrappedContent = (
        <div title={tooltipText}>
          {content}
        </div>
      );
      
      return item.href ? (
        <Link href={item.href}>{wrappedContent}</Link>
      ) : (
        wrappedContent
      );
    }

    // Regular menu item
    return item.href ? (
      <Link href={item.href}>{content}</Link>
    ) : (
      content
    );
  }

  // Render group menu item - no permission checking
  if (item.type === 'group' && item.items) {
    const allItems = item.items;

    const groupContent = (
      <div>
        {/* Group header */}
        <div 
          className={`relative flex items-center space-x-3 px-3 py-2 rounded-md transition-all duration-200 group cursor-pointer ${
            hasActiveChild 
              ? 'bg-muted text-foreground' 
              : 'hover:bg-muted text-foreground hover:text-foreground'
          }`}
          onClick={!sidebarCollapsed ? onToggle : undefined}
        >
          {/* Icon */}
          {item.icon && (
            <div className="flex-shrink-0">
              {item.icon}
            </div>
          )}
          
          {/* Label */}
          <span className={`flex-1 text-sm font-medium ${sidebarCollapsed ? 'sr-only' : ''}`}>
            {item.label}
          </span>
          
          {/* Expand arrow */}
          {!sidebarCollapsed && (
            <ChevronDown 
              className={`h-4 w-4 transition-transform duration-200 ${
                isExpanded ? 'transform rotate-180' : ''
              }`} 
            />
          )}
        </div>
        
        {/* Submenu items */}
        {isExpanded && !sidebarCollapsed && allItems.length > 0 && (
          <div className="ml-4 mt-1 space-y-1">
            {allItems.map((child) => (
              <RestrictedMenuItem
                key={child.id}
                item={child}
                depth={depth + 1}
                sidebarCollapsed={sidebarCollapsed}
              />
            ))}
          </div>
        )}
      </div>
    );

    // Add title attribute for collapsed sidebar
    if (sidebarCollapsed) {
      const childList = allItems.slice(0, 3).map(child => child.label).join(', ');
      const tooltipText = `${item.label} - ${item.description}${allItems.length > 0 ? ` - Includes: ${childList}${allItems.length > 3 ? '...' : ''}` : ''}`;
      
      return (
        <div title={tooltipText}>
          {groupContent}
        </div>
      );
    }

    return groupContent;
  }

  return null;
}