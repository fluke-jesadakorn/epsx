'use client';

import { WalletConnectAuth } from '@/components/auth/WalletConnectauth';
import { useSharedAuth } from '@/shared/components/auth/Provider';
import { useRouter, useSearchParams } from 'next/navigation';
import { useEffect } from 'react';
import { toast } from 'sonner';

export default function AuthPage() {
    const router = useRouter();
    const searchParams = useSearchParams();
    const returnUrl = searchParams.get('return_url') ?? '/';
    const { isAuthenticated, user } = useSharedAuth();

    // Auto-redirect when authenticated
    useEffect(() => {
        if (isAuthenticated && user) {
            router.push(returnUrl);
            router.refresh();
        }
    }, [isAuthenticated, user, returnUrl, router]);

    const handleAuthSuccess = async (_walletAddress: string) => {
        // Shared auth provider handles the heavy lifting
        toast.success('Authenticated successfully');
        // Redirection is handled by the useEffect above
    };

    return (
        <div className="flex min-h-[calc(100vh-80px)] items-center justify-center p-4">
            <div className="w-full max-w-md space-y-8 rounded-2xl bg-white/50 p-8 shadow-xl backdrop-blur-xl dark:bg-slate-900/50">
                <div className="text-center">
                    <h1 className="mb-2 bg-gradient-to-r from-orange-500 to-purple-600 bg-clip-text text-3xl font-bold text-transparent">
                        Welcome to EPSX
                    </h1>
                    <p className="text-muted-foreground">
                        Connect your wallet to access premium analytics
                    </p>
                </div>

                <div className="flex justify-center">
                    <WalletConnectAuth
                        onAuthSuccess={handleAuthSuccess}
                        className="w-full justify-center py-4 text-lg"
                    />
                </div>

                <div className="text-center text-xs text-muted-foreground">
                    By connecting, you agree to our Terms of Service and Privacy Policy.
                </div>
            </div>
        </div>
    );
}
