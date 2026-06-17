'use client';

import { useSharedAuth } from '@/shared/components/auth';
import { Loader2 } from 'lucide-react';
import { FrontendAuthGate } from './frontend-auth-gate';

interface GlobalAuthGuardProps {
    children?: React.ReactNode;
    fallback?: React.ReactNode;
    title?: string;
    debugInfo?: unknown;
}

export function GlobalAuthGuard({ children, fallback }: GlobalAuthGuardProps) {
    const { isAuthenticated, isLoading } = useSharedAuth();

    if (isLoading) {
        return (
            <div className="flex h-64 items-center justify-center p-6">
                <Loader2 className="h-8 w-8 animate-spin text-primary/50" />
            </div>
        );
    }

    if (isAuthenticated) {
        return <>{children}</>;
    }

    if (fallback !== undefined) {
        return <>{fallback}</>;
    }

    return <FrontendAuthGate />;
}
