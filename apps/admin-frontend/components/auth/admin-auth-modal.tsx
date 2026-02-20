'use client';

import { Loader2 } from 'lucide-react';
import { useRouter } from 'next/navigation';
import { useEffect } from 'react';

import { useSharedAuth } from '@/shared/components/auth';

interface AdminAuthModalProps {
    children: React.ReactNode;
    fallback?: React.ReactNode;
    initialHasAuthCookie?: boolean;
}

export function AdminAuthModal({ children, fallback, initialHasAuthCookie = false }: AdminAuthModalProps) {
    const router = useRouter();
    const { isAuthenticated, isLoading } = useSharedAuth();

    const isChecking = isLoading ?? (initialHasAuthCookie && !isAuthenticated);

    // Redirect to /auth when not authenticated and done checking
    useEffect(() => {
        if (!isChecking && !isAuthenticated) {
            router.replace('/auth');
        }
    }, [isChecking, isAuthenticated, router]);

    // Authenticated - render children
    if (isAuthenticated && children) {
        return <>{children}</>;
    }

    // Custom fallback
    if (fallback) {
        return <>{fallback}</>;
    }

    // Loading / redirecting state
    return (
        <div className="flex min-h-screen items-center justify-center p-6">
            <div className="flex flex-col items-center gap-2">
                <Loader2 className="h-8 w-8 animate-spin text-primary/50" />
                <p className="text-sm text-muted-foreground animate-pulse">
                    {isChecking ? 'Verifying admin access...' : 'Redirecting to login...'}
                </p>
            </div>
        </div>
    );
}
