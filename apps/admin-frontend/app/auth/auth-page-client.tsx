'use client';

/**
 * AUTH PAGE CLIENT COMPONENT
 * Client-side auth modal with Web3 wallet operations
 * Receives server-verified session state as props
 */

import { PageLayout, PageSkeleton } from '@/components/shared';
import { getAndClearReturnUrlAction } from '@/lib/auth/auth-actions';
import { AuthModal } from '@/shared/components/auth';
import { useSharedAuth } from '@/shared/components/auth/Provider';
import { logger } from '@/shared/utils/logger';
import { useRouter, useSearchParams } from 'next/navigation';
import React, { useEffect, useState } from 'react';
import { useAccount } from 'wagmi';

interface AuthPageClientProps {
    /** Server-verified session state - true if server has valid access token */
    serverHasSession: boolean;
}

export default function AuthPageClient({ serverHasSession }: AuthPageClientProps) {
    const router = useRouter();
    const searchParams = useSearchParams();
    const [showModal] = useState(true);
    const [mounted, setMounted] = useState(false);
    const redirectingRef = React.useRef(false);

    useAccount();
    const { isAuthenticated, user } = useSharedAuth();

    const reason = searchParams.get('reason');

    useEffect(() => {
        setMounted(true);
    }, []);

    // Redirect if BOTH server and client confirm authentication
    useEffect(() => {
        if (!mounted || redirectingRef.current) { return; }

        const checkAndRedirect = async () => {
            // Only redirect if server confirms session AND client has user
            if (serverHasSession && isAuthenticated && user) {
                if (redirectingRef.current) { return; }
                redirectingRef.current = true;

                const returnUrl = await getAndClearReturnUrlAction();
                logger.info('[AUTH] Server+Client authenticated, redirecting to', { returnUrl });
                router.push(returnUrl);
            }
        };

        void checkAndRedirect();
    }, [mounted, serverHasSession, isAuthenticated, user, router]);

    const handleAuthSuccess = async () => {
        try {
            if (redirectingRef.current) { return; }
            redirectingRef.current = true;

            const returnUrl = await getAndClearReturnUrlAction();
            logger.info('[AUTH] handleAuthSuccess: redirecting to', { returnUrl });
            router.replace(returnUrl);
        } catch (error) {
            logger.error('[AUTH] handleAuthSuccess error:', error instanceof Error ? error.message : String(error));
            redirectingRef.current = false;
            router.replace('/');
        }
    };

    const handleClose = () => {
        if (typeof window !== 'undefined' && window.history.length > 1) {
            router.back();
        }
    };

    if (!mounted) {
        return <PageSkeleton showHeader={false} stats={0} rows={0} />;
    }

    // CRITICAL FIX: Only show "Admin Access Granted" when BOTH server and client say authenticated
    // After logout, serverHasSession will be false even if client state is stale
    if (serverHasSession && isAuthenticated && user) {
        return (
            <PageLayout>
                <div className="flex flex-col items-center justify-center py-16">
                    <div className="w-16 h-16 bg-success/10 rounded-2xl flex items-center justify-center mb-4">
                        <span className="text-3xl">✅</span>
                    </div>
                    <p className="text-lg font-medium text-foreground">Admin Access Granted!</p>
                    <p className="text-muted-foreground">Redirecting...</p>
                </div>
            </PageLayout>
        );
    }

    // Show auth modal - server says no session (or stale client state after logout)
    return (
        <PageLayout>
            {reason && (
                <div className="fixed top-0 left-0 right-0 p-3 bg-destructive text-destructive-foreground text-center text-sm z-50">
                    {reason === 'no-session' && 'Your session has expired. Please sign in again.'}
                    {reason === 'no-admin-permissions' && 'Admin permissions required.'}
                </div>
            )}

            <AuthModal
                isOpen={showModal}
                onClose={handleClose}
                variant="admin"
                onSuccess={() => { void handleAuthSuccess(); }}
            />
        </PageLayout>
    );
}
