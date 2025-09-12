'use client';

import { Bell } from 'lucide-react';

interface NavbarSkeletonProps {
  showNotifications?: boolean;
  showUser?: boolean;
}

export function NavbarSkeleton({ showNotifications = false, showUser = false }: NavbarSkeletonProps) {
  return (
    <>
      {/* Notifications Skeleton */}
      {showNotifications && (
        <div className="relative">
          <div className="p-3 bg-orange-50 dark:bg-slate-800/50 rounded-2xl transition-all duration-300 shadow-sm">
            <Bell className="!text-orange-500 !h-5 !w-5 animate-pulse opacity-50" />
          </div>
          <div className="absolute -top-1 -right-1 w-5 h-5 bg-gray-300 dark:bg-gray-600 rounded-full animate-pulse"></div>
        </div>
      )}
      
      {/* Theme Toggle Skeleton */}
      <div className="p-1.5 bg-slate-100 dark:bg-slate-800/50 rounded-2xl shadow-sm">
        <div className="h-12 w-12 bg-gradient-to-r from-purple-400 to-pink-500 text-white rounded-2xl font-semibold shadow-lg transition-all duration-200 flex items-center justify-center animate-pulse">
          <span className="text-xl">🌙</span>
        </div>
      </div>
      
      {/* User Profile Skeleton */}
      {showUser && (
        <div className="flex items-center gap-3 px-4 py-2.5 bg-gradient-to-r from-slate-50 to-orange-50 dark:from-slate-800 dark:to-slate-700 rounded-2xl shadow-sm border border-orange-100/50 dark:border-slate-600/50 animate-pulse">
          <div className="w-8 h-8 bg-gradient-to-br from-yellow-400 to-orange-500 rounded-xl"></div>
          <div className="h-4 w-16 bg-slate-400/50 dark:bg-slate-500/50 rounded"></div>
        </div>
      )}
      
      {/* Auth Action Skeleton */}
      {!showUser && (
        <div className="hidden md:block">
          <div className="px-6 py-3 bg-gradient-to-r from-yellow-400 to-orange-500 rounded-2xl font-bold flex items-center gap-2 text-sm shadow-lg border border-yellow-300/50 animate-pulse text-white">
            <div className="h-4 w-4 bg-white/40 rounded"></div>
            <div className="h-4 w-16 bg-white/40 rounded"></div>
          </div>
        </div>
      )}
    </>
  );
}