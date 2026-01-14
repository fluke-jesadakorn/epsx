'use client';

import { WalletConnectAuth } from '@/components/auth/WalletConnectAuth';
import { loginAction } from '@/shared/auth/actions';
import { useRouter, useSearchParams } from 'next/navigation';
import { toast } from 'sonner';

export default function ConnectWalletPage() {
    const router = useRouter();
    const searchParams = useSearchParams();
    const returnUrl = searchParams.get('return_url') || '/';

    const handleAuthSuccess = async (walletAddress: string) => {
        // Note: complex auth logic is handled inside WalletConnectAuth
        // However, we need to ensure the server session is set.
        // Ideally WalletConnectAuth should return the full auth result,
        // but looking at its implementation, it calls onAuthSuccess with just the address.
        // We might need to rely on the side-effects of usePermissionAuth, 
        // OR primarily rely on the fact that `usePermissionAuth` implementation uses `web3Client`.

        // Wait a moment for client-side state to settle (localStorage etc)
        // In a real implementation with Server Actions, we should likely
        // pass the token TO the action.

        // Since WalletConnectAuth abstracts away the token, we need to 
        // check if we can get it from localStorage or cookies to pass to the server action.

        const token = localStorage.getItem('epsx.access_token');
        if (token) {
            // Construct user data similar to Admin
            const cookieData = {
                wallet_address: walletAddress,
                sub: walletAddress,
                auth_time: Date.now(),
                permissions: [], // Frontend usually fetches these dynamically
                groups: ['user'],
                isAdmin: false,
                expires_at: Date.now() + 2592000000 // 30 days
            };

            try {
                const result = await loginAction(token, cookieData);
                if (result.success) {
                    toast.success('Session established successfully');
                    router.push(returnUrl);
                    router.refresh();
                } else {
                    console.error('Server session failed:', result.error);
                    toast.error('Failed to establish server session');
                }
            } catch (e) {
                console.error('Server action error:', e);
                toast.error('Login error');
            }
        } else {
            // Fallback or retry?
            // If WalletConnectAuth succeeded, the token SHOULD be in storage/client state.
            console.warn('No token found after auth success');
            router.push(returnUrl);
        }
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
