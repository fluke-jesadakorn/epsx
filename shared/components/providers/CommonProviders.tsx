'use client';

import { TooltipProvider } from '@/shared/components/ui/tooltip';
import { ThemeProvider } from 'next-themes';
import React from 'react';

/**
 * COMMON PROVIDERS
 * A unified wrapper for non-auth providers used across all apps
 */
export function CommonProviders({ children }: { children: React.ReactNode }) {
    return (
        <TooltipProvider>
            <ThemeProvider attribute="class" defaultTheme="dark" enableSystem={false} disableTransitionOnChange>
                {/* 
              NOTE: These components are currently app-specific. 
              Long term they should be moved to @shared/components/providers/ 
          */}
                {children}
            </ThemeProvider>
        </TooltipProvider>
    );
}
