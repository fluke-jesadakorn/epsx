'use client';

import { ChevronDown } from 'lucide-react';
import Link from 'next/link';
import { usePathname } from 'next/navigation';

import {
  DropdownMenu,
  DropdownMenuContent,
  DropdownMenuItem,
  DropdownMenuTrigger,
} from '@/components/ui';

import { type NavGroup, NAV_GROUPS, isGroupActive, isItemActive } from './nav-config';

function GroupDropdown({ group }: { group: NavGroup }) {
  const pathname = usePathname();
  const active = isGroupActive(group, pathname);

  return (
    <DropdownMenu>
      <DropdownMenuTrigger asChild>
        <button
          className={`flex items-center gap-1 px-3 py-1.5 text-sm font-medium rounded-md transition-colors ${
            active
              ? 'text-slate-900 dark:text-white'
              : 'text-slate-500 hover:text-slate-900 dark:text-slate-400 dark:hover:text-white'
          }`}
        >
          {group.icon != null && <group.icon className="h-4 w-4 text-orange-500" />}
          {group.label}
          <ChevronDown className="h-3 w-3 text-orange-500" />
        </button>
      </DropdownMenuTrigger>
      <DropdownMenuContent
        align="start"
        sideOffset={8}
        style={{ zIndex: 99999 }}
        className="w-52 p-1.5 bg-white border border-slate-200 shadow-xl rounded-lg dark:bg-slate-900 dark:border-slate-700"
      >
        {group.items.map(item => {
          const Icon = item.icon;
          const itemActive = isItemActive(item, pathname);
          return (
            <DropdownMenuItem key={item.key} asChild className="p-0">
              <Link
                href={item.href}
                className={`flex items-center gap-2.5 px-2.5 py-2 rounded-md text-sm cursor-pointer transition-colors ${
                  itemActive
                    ? 'bg-slate-100 text-slate-900 dark:bg-slate-800 dark:text-white'
                    : 'text-slate-600 hover:bg-slate-50 hover:text-slate-900 dark:text-slate-300 dark:hover:bg-slate-800 dark:hover:text-white'
                }`}
              >
                {Icon != null && <Icon className="h-4 w-4 shrink-0 text-orange-500" />}
                <div className="min-w-0">
                  <div className="font-medium">{item.label}</div>
                  {item.desc != null && (
                    <div className="text-xs text-slate-400 dark:text-slate-500">{item.desc}</div>
                  )}
                </div>
              </Link>
            </DropdownMenuItem>
          );
        })}
      </DropdownMenuContent>
    </DropdownMenu>
  );
}

export function DesktopNav() {
  return (
    <div className="hidden lg:flex items-center gap-6">
      {/* Logo */}
      <Link href="/" className="flex items-center hover:opacity-80 transition-opacity">
        <span className="text-xl font-bold bg-gradient-to-r from-blue-500 to-purple-600 bg-clip-text text-transparent">
          EPSX
        </span>
      </Link>

      {/* Group dropdowns */}
      <nav className="flex items-center gap-0.5">
        {NAV_GROUPS.map(group => (
          <GroupDropdown key={group.key} group={group} />
        ))}
      </nav>
    </div>
  );
}
