'use client';

import type { ReactNode } from 'react';
import { DeveloperSidebar } from './DeveloperSidebar';

interface DeveloperLayoutProps {
    children: ReactNode;
    title?: string;
    showSidebar?: boolean;
}

/**
 * Developer Portal Layout with Sidebar
 * Provides consistent layout for all developer pages
 */
export function DeveloperLayout({
    children,
    title = 'Developer',
    showSidebar = true
}: DeveloperLayoutProps) {
    return (
        <div className="flex min-h-screen bg-gradient-to-br from-slate-50 via-blue-50 to-indigo-50 dark:from-gray-900 dark:via-gray-900 dark:to-indigo-900">
            {/* Sidebar */}
            {showSidebar && (
                <DeveloperSidebar title={title} />
            )}

            {/* Main Content Area */}
            <main className="flex-1 overflow-y-auto">
                <div className="p-6 lg:p-8">
                    {children}
                </div>
            </main>
        </div>
    );
}
