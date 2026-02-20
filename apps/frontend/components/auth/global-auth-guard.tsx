'use client';

/**
 * GLOBAL AUTH GUARD
 * Shows authentication UI when user needs to sign in
 * Uses shared AuthModal for consistent experience
 */

import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '@/components/ui/card';
import { AuthModal } from '@/shared/components/auth';
import { useSharedAuth } from '@/shared/components/auth';
import { Loader2, Shield } from 'lucide-react';
import { useEffect, useState } from 'react';

interface GlobalAuthGuardProps {
    children?: React.ReactNode;
    fallback?: React.ReactNode;
    title?: string;
    debugInfo?: Record<string, unknown>;
}

export function GlobalAuthGuard({
    children,
    fallback,
    title = "Authentication Required",
    debugInfo
}: GlobalAuthGuardProps) {
    const { isAuthenticated, isLoading } = useSharedAuth();
    const [showModal, setShowModal] = useState(false);
    const [hasCheckedAuth, setHasCheckedAuth] = useState(false);

    useEffect(() => {
        if (!isLoading) {
            setHasCheckedAuth(true);
        }
    }, [isLoading]);

    // Authenticated - render children immediately (Optimistic UI)
    // We prioritize this over loading state because if we have the user object (from cookies),
    // we should show the content while background verification happens.
    if (isAuthenticated && children) {
        return <>{children}</>;
    }

    // Loading state
    if (!hasCheckedAuth || isLoading) {
        // Debug check for cookie presence
        const hasUserCookie = typeof document !== 'undefined' ? document.cookie.includes('epsx.user') : false;

        return (
            <div className="flex h-64 items-center justify-center p-6 bg-slate-50/50 dark:bg-card rounded-xl border border-dashed border-slate-200 dark:border-slate-800">
                <div className="flex flex-col items-center gap-2">
                    <Loader2 className="h-8 w-8 animate-spin text-primary/50" />
                    <p className="text-sm text-muted-foreground animate-pulse">Verifying access...</p>
                    <div className="text-xs text-muted-foreground mt-4 font-mono bg-slate-100 dark:bg-slate-800 p-2 rounded">
                        Debug: Auth={String(isAuthenticated)}, Load={String(isLoading)}, Cookie={String(hasUserCookie)}
                    </div>
                </div>
            </div>
        );
    }

    // Custom fallback
    if (fallback) {
        return <>{fallback}</>;
    }

    // Default: Show auth card with modal trigger
    return (
        <>
            <Card className="max-w-md mx-auto border-orange-200 dark:border-orange-900 bg-orange-50/30 dark:bg-orange-900/10">
                <CardHeader className="text-center pb-2">
                    <div className="mx-auto bg-orange-100 dark:bg-orange-900/40 w-12 h-12 rounded-full flex items-center justify-center mb-4">
                        <Shield className="h-6 w-6 text-orange-600 dark:text-orange-400" />
                    </div>
                    <CardTitle>{title}</CardTitle>
                    <CardDescription>
                        Security verification needed to access this area
                    </CardDescription>
                </CardHeader>

                <CardContent className="pt-4">
                    <button
                        className="w-full flex items-center justify-center gap-2 p-3 rounded-lg bg-gradient-to-r from-orange-500 to-pink-500 text-white font-semibold hover:opacity-90 transition-opacity"
                        onClick={() => setShowModal(true)}
                    >
                        <Shield className="h-4 w-4" />
                        Sign In to Access
                    </button>

                    {debugInfo && (
                        <details className="text-xs text-muted-foreground mt-4 pt-4 border-t border-slate-100 dark:border-slate-800">
                            <summary className="cursor-pointer hover:text-foreground">Debug Info</summary>
                            <pre className="mt-2 p-2 bg-slate-100 dark:bg-card rounded overflow-auto max-h-40">
                                {JSON.stringify(debugInfo, null, 2)}
                            </pre>
                        </details>
                    )}
                </CardContent>
            </Card>

            <AuthModal
                isOpen={showModal}
                onClose={() => setShowModal(false)}
                variant="user"
                onSuccess={() => {
                    setShowModal(false);
                    if (typeof window !== 'undefined') {
                        window.location.reload();
                    }
                }}
            />
        </>
    );
}
