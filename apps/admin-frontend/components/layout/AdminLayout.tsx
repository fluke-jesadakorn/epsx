"use client";

import type { ReactNode } from "react";

interface AdminLayoutProps {
  children: ReactNode;
}

export default function AdminLayout({ children }: AdminLayoutProps) {
  return (
    <div className="min-h-screen flex">
      {/* Sidebar placeholder */}
      <aside className="w-64 bg-card border-r border-border">
        <div className="p-4">
          <h2 className="text-lg font-semibold">ESPx Admin</h2>
        </div>
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
