import { GlobalAuthGuard } from '@/components/auth/GlobalAuthGuard';
import { getCurrentUser } from '@/lib/server-actions';
import { getDebugSessionInfo } from '@/lib/server-actions-user';
import Link from 'next/link';
import type { ReactNode } from 'react';

export const dynamic = 'force-dynamic';

interface DeveloperLayoutProps {
    children: ReactNode;
}

export default async function DeveloperLayout({ children }: DeveloperLayoutProps) {
    const user = await getCurrentUser();
    const debugInfo = !user ? await getDebugSessionInfo() : null;

    if (!user) {
        return (
            <div className="container mx-auto p-6">
                <GlobalAuthGuard title="Developer API" debugInfo={debugInfo} />
            </div>
        );
    }

    return (
        <div className="min-h-screen bg-gradient-to-br from-slate-50 via-blue-50 to-indigo-50 dark:from-gray-900 dark:via-gray-900 dark:to-indigo-900">
            <div className="container mx-auto px-4 py-12">
                {/* Hero Section */}
                <div className="text-center mb-12">
                    <h1 className="text-4xl md:text-6xl font-bold bg-gradient-to-r from-emerald-600 via-blue-600 to-purple-600 bg-clip-text text-transparent mb-6">
                        Developer API Portal
                    </h1>
                    <p className="text-xl text-gray-600 dark:text-gray-300 max-w-3xl mx-auto leading-relaxed">
                        Access EPSX analytics data programmatically with our REST API. Manage your API keys, monitor usage, and integrate our powerful financial data into your applications.
                    </p>
                </div>

                {/* Navigation */}
                <DeveloperNav />

                {/* Page Content */}
                {children}
            </div>
        </div>
    );
}

function DeveloperNav() {
    const navItems = [
        {
            href: '/developer/keys',
            label: 'API Keys',
            icon: (
                <svg className="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                    <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M15 7a2 2 0 012 2m4 0a6 6 0 01-7.743 5.743L11 17H9v2H7v2H4a1 1 0 01-1-1v-2.586a1 1 0 01.293-.707l5.964-5.964A6 6 0 1121 9z" />
                </svg>
            ),
            description: 'Manage your API keys'
        },
        {
            href: '/developer/docs',
            label: 'Documentation',
            icon: (
                <svg className="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                    <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M12 6.253v13m0-13C10.832 5.477 9.246 5 7.5 5S4.168 5.477 3 6.253v13C4.168 18.477 5.754 18 7.5 18s3.332.477 4.5 1.253m0-13C13.168 5.477 14.754 5 16.5 5c1.747 0 3.332.477 4.5 1.253v13C19.832 18.477 18.247 18 16.5 18c-1.746 0-3.332.477-4.5 1.253" />
                </svg>
            ),
            description: 'Interactive API docs'
        },
        {
            href: '/developer/usage',
            label: 'Usage & Monitoring',
            icon: (
                <svg className="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                    <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M9 19v-6a2 2 0 00-2-2H5a2 2 0 00-2 2v6a2 2 0 002 2h2a2 2 0 002-2zm0 0V9a2 2 0 012-2h2a2 2 0 012 2v10m-6 0a2 2 0 002 2h2a2 2 0 002-2m0 0V5a2 2 0 012-2h2a2 2 0 012 2v14a2 2 0 01-2 2h-2a2 2 0 01-2-2z" />
                </svg>
            ),
            description: 'Monitor your usage'
        }
    ];

    return (
        <nav className="mb-8">
            <div className="flex flex-wrap justify-center gap-4">
                {navItems.map((item) => (
                    <Link
                        key={item.href}
                        href={item.href}
                        className="group flex items-center gap-3 px-6 py-4 rounded-2xl bg-white/80 dark:bg-gray-800/80 backdrop-blur-xl border border-white/20 dark:border-gray-700/50 shadow-lg hover:shadow-xl transition-all duration-300 hover:-translate-y-1"
                    >
                        <div className="w-10 h-10 rounded-xl bg-gradient-to-br from-emerald-500 to-blue-500 flex items-center justify-center text-white group-hover:scale-110 transition-transform">
                            {item.icon}
                        </div>
                        <div className="text-left">
                            <div className="font-semibold text-gray-900 dark:text-white">{item.label}</div>
                            <div className="text-xs text-gray-500 dark:text-gray-400">{item.description}</div>
                        </div>
                    </Link>
                ))}
            </div>
        </nav>
    );
}
