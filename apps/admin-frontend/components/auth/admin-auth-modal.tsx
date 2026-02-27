'use client';

import { Loader2 } from 'lucide-react';

import { useSharedAuth } from '@/shared/components/auth';

interface AdminAuthModalProps {
    children: React.ReactNode;
    fallback?: React.ReactNode;
    initialHasAuthCookie?: boolean;
}

export function AdminAuthModal({ children, fallback, initialHasAuthCookie = false }: AdminAuthModalProps) {
    const { isAuthenticated, isLoading } = useSharedAuth();

    const isChecking = isLoading || (initialHasAuthCookie && !isAuthenticated);

    // Authenticated - render children
    if (isAuthenticated === true && children !== undefined) {
        return <>{children}</>;
    }

    // Custom fallback
    if (fallback !== undefined) {
        return <>{fallback}</>;
    }

    // Loading state (layout gate handles unauthenticated)
    return (
        <div className="flex min-h-screen items-center justify-center p-6">
            <div className="flex flex-col items-center gap-2">
                <Loader2 className="h-8 w-8 animate-spin text-primary/50" />
                <p className="text-sm text-muted-foreground animate-pulse">
                    {isChecking ? 'Verifying admin access...' : 'Loading...'}
                </p>
            </div>
        </div>
    );
}
