'use client';

import { ServerAuthProvider } from '@/components/auth/ServerAuthProvider';
import { LoadingSpinner } from '@/components/common/LoadingSpinner';
import { Navigation } from '@/components/nav';
import { StateProvider } from '@/components/state/StateProvider';
import { ToastProvider } from '@/components/ui/toaster';
import { useLoadingState, useUI } from '@/context/ui-context';
import { useResponsive } from '@/hooks/state/useStateSelector';
import { ThemeProvider } from 'next-themes';
import React, { useEffect } from 'react';

interface AppLayoutProps {
  children: React.ReactNode;
  serverAuthState?: any;
  serverUserPreferences?: any;
  className?: string;
}

// Inner layout component that uses the state management hooks
function InnerLayout({
  children,
  className,
}: {
  children: React.ReactNode;
  className?: string;
}) {
  const { theme, globalLoading } = useUI();
  const { isAnyLoading } = useLoadingState();
  const { isMobile, isTablet } = useResponsive();

  // Handle responsive classes
  useEffect(() => {
    const body = document.body;
    body.classList.toggle('mobile', isMobile);
    body.classList.toggle('tablet', isTablet);
    body.classList.toggle('desktop', !isMobile && !isTablet);
  }, [isMobile, isTablet]);

  return (
    <div className={`min-h-screen ${className || ''}`}>
      {/* Global loading overlay */}
      {(globalLoading || isAnyLoading) && (
        <div className="fixed inset-0 bg-background/80 backdrop-blur-sm z-50 flex items-center justify-center">
          <LoadingSpinner size="lg" className="text-primary" />
        </div>
      )}

      {/* PancakeSwap-style background decorations */}
      <div className="fixed inset-0 pointer-events-none z-0 overflow-hidden">
        {/* Floating pancake emojis with better positioning */}
        <div
          className="pancake-emoji text-4xl opacity-15 animate-float-gentle"
          style={{ top: '8%', left: '3%', animationDelay: '0s' }}
        >
          🥞
        </div>
        <div
          className="pancake-emoji text-3xl opacity-10 animate-bounce-gentle"
          style={{ top: '15%', right: '8%', animationDelay: '1s' }}
        >
          🧁
        </div>
        <div
          className="pancake-emoji text-5xl opacity-12 animate-float"
          style={{ bottom: '25%', left: '6%', animationDelay: '2s' }}
        >
          🍰
        </div>
        <div
          className="pancake-emoji text-4xl opacity-15 animate-pulse-gentle"
          style={{
            bottom: '12%',
            right: '12%',
            animationDelay: '3s',
          }}
        >
          🥞
        </div>
        <div
          className="pancake-emoji text-2xl opacity-6 animate-float-gentle"
          style={{ top: '75%', left: '25%', animationDelay: '9s' }}
        >
          🍓
        </div>
        <div
          className="pancake-emoji text-3xl opacity-8 animate-bounce-gentle"
          style={{ top: '25%', right: '30%', animationDelay: '10s' }}
        >
          🍌
        </div>
        <div
          className="pancake-emoji text-2xl opacity-7 animate-float"
          style={{
            bottom: '60%',
            left: '30%',
            animationDelay: '11s',
          }}
        >
          🥛
        </div>

        {/* Gradient orbs with vibrant colors */}
        <div className="absolute -top-20 -left-20 w-48 h-48 bg-gradient-to-br from-orange-400/8 to-yellow-400/8 rounded-full blur-3xl animate-float" />
        <div className="absolute top-1/4 left-1/3 w-36 h-36 bg-gradient-to-br from-orange-400/6 to-yellow-400/6 rounded-full blur-2xl animate-bounce-gentle" />
        <div className="absolute bottom-1/3 right-1/4 w-52 h-52 bg-gradient-to-br from-blue-400/7 to-cyan-400/7 rounded-full blur-3xl animate-pulse-gentle" />
        <div className="absolute top-1/2 right-1/3 w-32 h-32 bg-gradient-to-br from-purple-400/6 to-pink-400/6 rounded-full blur-2xl animate-float-gentle" />
        <div className="absolute -bottom-16 -right-16 w-44 h-44 bg-gradient-to-br from-green-400/5 to-emerald-400/5 rounded-full blur-3xl animate-bounce-gentle" />

        {/* Mesh gradient overlays for depth */}
        <div className="absolute inset-0 bg-[radial-gradient(circle_at_20%_30%,_rgba(255,133,27,0.04)_0%,_transparent_50%)] animate-pulse-slow" />
        <div
          className="absolute inset-0 bg-[radial-gradient(circle_at_80%_70%,_rgba(59,130,246,0.03)_0%,_transparent_50%)] animate-pulse-slow"
          style={{ animationDelay: '2s' }}
        />
      </div>

      <Navigation />
      <main className="relative z-10">{children}</main>
    </div>
  );
}

// Main app layout component
export function AppLayout({
  children,
  serverAuthState,
  serverUserPreferences,
  className,
}: AppLayoutProps) {
  return (
    <ThemeProvider attribute="class" defaultTheme="system" enableSystem>
      <StateProvider
        serverAuthState={serverAuthState}
        serverUserPreferences={serverUserPreferences}
      >
        <ServerAuthProvider>
          <ToastProvider>
            <InnerLayout className={className}>{children}</InnerLayout>
          </ToastProvider>
        </ServerAuthProvider>
      </StateProvider>
    </ThemeProvider>
  );
}

// HOC for pages that need the app layout
export function withAppLayout<P extends object>(
  Component: React.ComponentType<P>,
  options: {
    serverAuthState?: any;
    serverUserPreferences?: any;
    className?: string;
  } = {}
) {
  const WrappedComponent = React.forwardRef<any, P>((props, ref) => {
    return (
      <AppLayout {...options}>
        <Component {...props} ref={ref} />
      </AppLayout>
    );
  });

  WrappedComponent.displayName = `withAppLayout(${Component.displayName || Component.name})`;

  return WrappedComponent;
}

// Custom hook for layout state
export function useLayoutState() {
  const {
    sidebarOpen,
    sidebarCollapsed,
    toggleSidebar,
    collapseSidebar,
    isMobile,
    isTablet,
    isDesktop,
  } = useUI();

  return {
    sidebar: {
      open: sidebarOpen,
      collapsed: sidebarCollapsed,
      toggle: toggleSidebar,
      collapse: collapseSidebar,
    },
    responsive: {
      isMobile,
      isTablet,
      isDesktop,
    },
  };
}

// Performance-optimized layout wrapper
export const MemoizedAppLayout = React.memo(AppLayout);

export default AppLayout;
