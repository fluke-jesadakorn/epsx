import { DeveloperStatsCards } from '@/components/developer/DeveloperStatsCards';
import { getCurrentUser } from '@/lib/server-actions';
import Link from 'next/link';

export const dynamic = 'force-dynamic';

export default async function DeveloperPage() {
  const user = await getCurrentUser();

  if (!user) {
    return null; // Layout handles auth guard
  }

  return (
    <div className="space-y-8">
      {/* Hero Section */}
      <div className="text-left mb-8">
        <h1 className="text-3xl md:text-4xl font-bold bg-gradient-to-r from-emerald-600 via-blue-600 to-purple-600 bg-clip-text text-transparent mb-4">
          Developer Portal
        </h1>
        <p className="text-lg text-gray-600 dark:text-gray-300 max-w-2xl leading-relaxed">
          Access EPSX analytics data programmatically with our REST API. Manage your API keys, monitor usage, and integrate our powerful financial data into your applications.
        </p>
      </div>

      {/* Stats Cards */}
      <DeveloperStatsCards currentUser={user} />

      {/* Quick Actions */}
      <div className="grid md:grid-cols-3 gap-6">
        <Link href="/developer/keys" className="group">
          <div className="relative overflow-hidden rounded-2xl bg-gradient-to-br from-white/80 to-white/40 dark:from-gray-800/80 dark:to-gray-800/40 backdrop-blur-xl border border-white/20 dark:border-gray-700/50 shadow-xl p-6 hover:shadow-2xl transition-all duration-300 hover:-translate-y-1">
            <div className="absolute inset-0 bg-gradient-to-br from-amber-500/10 to-orange-500/10 opacity-0 group-hover:opacity-100 transition-opacity duration-300" />
            <div className="relative">
              <div className="w-12 h-12 rounded-xl bg-gradient-to-br from-amber-500 to-orange-500 flex items-center justify-center shadow-lg shadow-amber-500/25 mb-4">
                <svg className="w-6 h-6 text-white" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                  <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M15 7a2 2 0 012 2m4 0a6 6 0 01-7.743 5.743L11 17H9v2H7v2H4a1 1 0 01-1-1v-2.586a1 1 0 01.293-.707l5.964-5.964A6 6 0 1121 9z" />
                </svg>
              </div>
              <h2 className="text-xl font-bold text-gray-900 dark:text-white mb-2">API Keys</h2>
              <p className="text-gray-600 dark:text-gray-400">Create, manage, and revoke your API keys for secure access.</p>
            </div>
          </div>
        </Link>

        <Link href="/developer/docs" className="group">
          <div className="relative overflow-hidden rounded-2xl bg-gradient-to-br from-white/80 to-white/40 dark:from-gray-800/80 dark:to-gray-800/40 backdrop-blur-xl border border-white/20 dark:border-gray-700/50 shadow-xl p-6 hover:shadow-2xl transition-all duration-300 hover:-translate-y-1">
            <div className="absolute inset-0 bg-gradient-to-br from-blue-500/10 to-indigo-500/10 opacity-0 group-hover:opacity-100 transition-opacity duration-300" />
            <div className="relative">
              <div className="w-12 h-12 rounded-xl bg-gradient-to-br from-blue-500 to-indigo-500 flex items-center justify-center shadow-lg shadow-blue-500/25 mb-4">
                <svg className="w-6 h-6 text-white" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                  <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M12 6.253v13m0-13C10.832 5.477 9.246 5 7.5 5S4.168 5.477 3 6.253v13C4.168 18.477 5.754 18 7.5 18s3.332.477 4.5 1.253m0-13C13.168 5.477 14.754 5 16.5 5c1.747 0 3.332.477 4.5 1.253v13C19.832 18.477 18.247 18 16.5 18c-1.746 0-3.332.477-4.5 1.253" />
                </svg>
              </div>
              <h2 className="text-xl font-bold text-gray-900 dark:text-white mb-2">Documentation</h2>
              <p className="text-gray-600 dark:text-gray-400">Interactive API documentation powered by Scalar.</p>
            </div>
          </div>
        </Link>

        <Link href="/developer/usage" className="group">
          <div className="relative overflow-hidden rounded-2xl bg-gradient-to-br from-white/80 to-white/40 dark:from-gray-800/80 dark:to-gray-800/40 backdrop-blur-xl border border-white/20 dark:border-gray-700/50 shadow-xl p-6 hover:shadow-2xl transition-all duration-300 hover:-translate-y-1">
            <div className="absolute inset-0 bg-gradient-to-br from-purple-500/10 to-pink-500/10 opacity-0 group-hover:opacity-100 transition-opacity duration-300" />
            <div className="relative">
              <div className="w-12 h-12 rounded-xl bg-gradient-to-br from-purple-500 to-pink-500 flex items-center justify-center shadow-lg shadow-purple-500/25 mb-4">
                <svg className="w-6 h-6 text-white" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                  <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M9 19v-6a2 2 0 00-2-2H5a2 2 0 00-2 2v6a2 2 0 002 2h2a2 2 0 002-2zm0 0V9a2 2 0 012-2h2a2 2 0 012 2v10m-6 0a2 2 0 002 2h2a2 2 0 002-2m0 0V5a2 2 0 012-2h2a2 2 0 012 2v14a2 2 0 01-2 2h-2a2 2 0 01-2-2z" />
                </svg>
              </div>
              <h2 className="text-xl font-bold text-gray-900 dark:text-white mb-2">Usage & Monitoring</h2>
              <p className="text-gray-600 dark:text-gray-400">Track API usage, view analytics, and monitor performance.</p>
            </div>
          </div>
        </Link>
      </div>

      {/* Info Card */}
      <div className="rounded-2xl bg-blue-50 dark:bg-blue-900/20 border border-blue-200 dark:border-blue-800 p-6">
        <h3 className="font-semibold text-blue-800 dark:text-blue-300 mb-2 flex items-center gap-2">
          🚀 Getting Started
        </h3>
        <div className="text-sm text-blue-700 dark:text-blue-400 space-y-2">
          <p>1. <strong>Create an API key</strong> - Generate your unique access key from the API Keys section.</p>
          <p>2. <strong>Read the documentation</strong> - Explore our interactive API docs to understand available endpoints.</p>
          <p>3. <strong>Start integrating</strong> - Use your API key to make requests and access EPSX data.</p>
        </div>
      </div>
    </div>
  );
}