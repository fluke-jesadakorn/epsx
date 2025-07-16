"use client";

import type { ReactNode } from "react";
import Link from "next/link";
import { useAdminAuth } from '@/hooks/useAdminAuth';

interface AdminLayoutProps {
  children: ReactNode;
}

export default function AdminLayout({ children }: AdminLayoutProps) {
  const { adminUser, canManageUsers, canViewPayments, canManageSystem, canViewAnalytics } = useAdminAuth();

  return (
    <div className="min-h-screen flex">
      {/* Sidebar with role-based navigation */}
      <aside className="w-64 bg-card border-r border-border">
        <div className="p-4">
          <h2 className="text-lg font-semibold">ESPx Admin</h2>
          <p className="text-sm text-muted-foreground">{adminUser.role}</p>
          <p className="text-xs text-muted-foreground">{adminUser.email}</p>
        </div>
        
        <nav className="p-4 space-y-2">
          <Link href="/admin" className="block p-2 rounded hover:bg-accent">
            Dashboard
          </Link>
          
          {canManageUsers() && (
            <Link href="/admin/users" className="block p-2 rounded hover:bg-accent">
              User Management
            </Link>
          )}
          
          {canViewPayments() && (
            <Link href="/admin/payments" className="block p-2 rounded hover:bg-accent">
              Payment Management
            </Link>
          )}
          
          {canViewAnalytics() && (
            <Link href="/admin/analytics" className="block p-2 rounded hover:bg-accent">
              Analytics
            </Link>
          )}
          
          {canManageSystem() && (
            <Link href="/admin/system" className="block p-2 rounded hover:bg-accent">
              System Settings
            </Link>
          )}
        </nav>
      </aside>
      
      {/* Main content */}
      <main className="flex-1 overflow-auto">
        <div className="container p-6">
          {children}
        </div>
      </main>
    </div>
  );
}
