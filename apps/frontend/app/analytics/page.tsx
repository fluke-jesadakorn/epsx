'use client';

import { CardDashboardView } from '@/components/analytics/CardDashboardView';

export default function AnalyticsPage() {
  return (
    <div className="relative min-h-screen overflow-hidden">
      {/* PancakeSwap-style vibrant background */}
      <div className="fixed inset-0 z-0">
        {/* Main gradient background */}
        <div className="absolute inset-0 bg-gradient-to-br from-blue-50 via-orange-50 to-yellow-50 dark:from-slate-900 dark:via-slate-800 dark:to-slate-900" />

        {/* Floating gradient orbs - PancakeSwap style */}
        <div className="animate-bounce-slow absolute -top-40 -left-40 h-96 w-96 rounded-full bg-gradient-to-br from-orange-400/30 to-yellow-400/30 blur-3xl" />
        <div className="animate-float absolute top-20 -right-32 h-80 w-80 rounded-full bg-gradient-to-br from-blue-400/25 to-cyan-400/25 blur-3xl" />
        <div className="animate-pulse-gentle absolute bottom-20 left-20 h-72 w-72 rounded-full bg-gradient-to-br from-purple-400/20 to-pink-400/20 blur-3xl" />
        <div className="animate-float-reverse absolute top-1/2 right-1/4 h-64 w-64 rounded-full bg-gradient-to-br from-green-400/15 to-emerald-400/15 blur-3xl" />

        {/* Mesh gradient overlays for depth */}
        <div className="animate-pulse-slow absolute inset-0 bg-[radial-gradient(circle_at_25%_25%,_rgba(255,133,27,0.1)_0%,_transparent_50%)]" />
        <div
          className="animate-pulse-slow absolute inset-0 bg-[radial-gradient(circle_at_75%_75%,_rgba(59,130,246,0.08)_0%,_transparent_50%)]"
          style={{ animationDelay: '1s' }}
        />
        <div
          className="animate-pulse-slow absolute inset-0 bg-[radial-gradient(circle_at_50%_50%,_rgba(168,85,247,0.06)_0%,_transparent_60%)]"
          style={{ animationDelay: '2s' }}
        />

        {/* Decorative geometric shapes */}
        <div className="animate-spin-slow absolute top-1/4 left-1/4 h-32 w-32 rotate-45 rounded-2xl bg-gradient-to-br from-orange-300/10 to-yellow-300/10" />
        <div className="animate-bounce-gentle absolute right-1/3 bottom-1/3 h-24 w-24 rounded-full bg-gradient-to-br from-blue-300/10 to-cyan-300/10" />
      </div>

      {/* Main content with PancakeSwap styling */}
      <div className="relative z-10">
        <div className="container mx-auto px-4 py-8">
          {/* Header with enhanced styling */}
          <div className="mb-8 text-center">
            <h1 className="animate-gradient-x mb-4 bg-gradient-to-r from-orange-500 via-yellow-500 to-orange-600 bg-clip-text text-4xl font-bold text-transparent sm:text-5xl dark:from-orange-400 dark:via-yellow-400 dark:to-orange-500">
              🎯 EPS Analytics Platform
            </h1>
            <p className="mx-auto max-w-2xl text-lg text-gray-600 dark:text-gray-300">
              Performance monitoring system with tech-focused decision support
              and real-time analytics
            </p>
            {/* Decorative elements */}
            <div className="mt-6 flex items-center justify-center gap-4">
              <div className="h-2 w-2 animate-pulse rounded-full bg-orange-400" />
              <div
                className="h-3 w-3 animate-pulse rounded-full bg-yellow-400"
                style={{ animationDelay: '0.5s' }}
              />
              <div
                className="h-2 w-2 animate-pulse rounded-full bg-orange-400"
                style={{ animationDelay: '1s' }}
              />
            </div>
          </div>

          {/* Card Dashboard View with floating decorations */}
          <div className="relative">
            <div className="absolute -top-8 -left-8 h-16 w-16 rounded-full bg-gradient-to-br from-orange-400/20 to-yellow-400/20 blur-xl" />
            <div className="absolute -right-8 -bottom-8 h-20 w-20 rounded-full bg-gradient-to-br from-blue-400/20 to-cyan-400/20 blur-xl" />
            <CardDashboardView />
          </div>
        </div>
      </div>
    </div>
  );
}
