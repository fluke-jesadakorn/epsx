'use client';

import { useSharedAuth } from '@/shared/components/auth';
import { FrontendAuthGate } from '@/components/auth/frontend-auth-gate';
import { Loader2 } from 'lucide-react';

export function PaymentAuthGuard() {
    const { isAuthenticated, isLoading } = useSharedAuth();

    if (isLoading) {
        return (
            <div className="flex flex-col items-center justify-center py-20">
                <Loader2 className="h-8 w-8 animate-spin text-primary/50 mb-4" />
                <p className="text-sm text-muted-foreground animate-pulse">Verifying access...</p>
            </div>
        );
    }

    if (isAuthenticated) {
        return null;
    }

    return <FrontendAuthGate />;
}
