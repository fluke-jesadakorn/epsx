'use client';

import { Shield } from 'lucide-react';
import { useEffect, useState } from 'react';

import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '@/components/ui/card';
import { AuthModal } from '@/shared/components/auth';
import { useSharedAuth } from '@/shared/components/auth/Provider';
import { Loader2 } from 'lucide-react';

interface AdminAuthModalProps {
    children: React.ReactNode;
    fallback?: React.ReactNode;
}

export function AdminAuthModal({ children, fallback }: AdminAuthModalProps) {
    const { isAuthenticated, isLoading } = useSharedAuth();
    const [showModal, setShowModal] = useState(false);
    const [hasCheckedAuth, setHasCheckedAuth] = useState(false);

    useEffect(() => {
        if (!isLoading) {
            setHasCheckedAuth(true);
        }
    }, [isLoading]);

    // Authenticated - render children immediately
    if (isAuthenticated && children) {
        return <>{children}</>;
    }

    // Loading state
    if (!hasCheckedAuth || isLoading) {
        const hasUserCookie = typeof document !== 'undefined' ? document.cookie.includes('epsx.user') : false;
        return (
            <div className="flex min-h-screen items-center justify-center p-6">
                <div className="flex flex-col items-center gap-2">
                    <Loader2 className="h-8 w-8 animate-spin text-primary/50" />
                    <p className="text-sm text-muted-foreground animate-pulse">Verifying admin access...</p>
                    <div className="text-xs text-muted-foreground mt-4 font-mono bg-muted p-2 rounded">
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
            <div className="flex min-h-screen items-center justify-center p-6">
                <Card className="max-w-md border-blue-200 dark:border-blue-900 bg-blue-50/30 dark:bg-blue-900/10">
                    <CardHeader className="text-center pb-2">
                        <div className="mx-auto bg-blue-100 dark:bg-blue-900/40 w-12 h-12 rounded-full flex items-center justify-center mb-4">
                            <Shield className="h-6 w-6 text-blue-600 dark:text-blue-400" />
                        </div>
                        <CardTitle>Admin Access Required</CardTitle>
                        <CardDescription>
                            Connect your admin wallet to access this area
                        </CardDescription>
                    </CardHeader>

                    <CardContent className="pt-4">
                        <button
                            className="w-full flex items-center justify-center gap-2 p-3 rounded-lg bg-gradient-to-r from-blue-500 to-purple-600 text-white font-semibold hover:opacity-90 transition-opacity"
                            onClick={() => setShowModal(true)}
                        >
                            <Shield className="h-4 w-4" />
                            Connect Admin Wallet
                        </button>
                    </CardContent>
                </Card>
            </div>

            <AuthModal
                isOpen={showModal}
                onClose={() => setShowModal(false)}
                variant="admin"
                onSuccess={() => {
                    setShowModal(false);
                    if (typeof window !== 'undefined') {
                        window.location.reload();
                    }
                }}
                onError={(err) => console.error('Auth error:', err)}
            />
        </>
    );
}
