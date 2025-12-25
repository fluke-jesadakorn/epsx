'use client';

import { Button } from '@/components/ui/button';
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '@/components/ui/card';
import { authenticateWallet } from '@/lib/auth/api-direct';
import { useSharedAuth } from '@/shared/components/auth/Provider';
import { useConnectModal } from '@rainbow-me/rainbowkit';
import { AlertTriangle, CheckCircle, Loader2, Shield, Wallet } from 'lucide-react';
import { useCallback, useEffect, useState } from 'react';
import { toast } from 'sonner';
import { useAccount, useSignMessage } from 'wagmi';

interface GlobalAuthGuardProps {
    children?: React.ReactNode;
    fallback?: React.ReactNode;
    title?: string;
    debugInfo?: any;
}

export function GlobalAuthGuard({
    children,
    fallback,
    title = "Authentication Required",
    debugInfo
}: GlobalAuthGuardProps) {
    const { address, isConnected } = useAccount();
    const { signMessageAsync } = useSignMessage();
    const { isAuthenticated, isLoading: authLoading, authenticateWithDirectApi } = useSharedAuth();
    const { openConnectModal } = useConnectModal();

    const [isSigning, setIsSigning] = useState(false);
    const [hasCheckedAuth, setHasCheckedAuth] = useState(false);

    // Initial auth status check
    useEffect(() => {
        if (!authLoading) {
            setHasCheckedAuth(true);
        }
    }, [authLoading]);

    // Handle the sign-in flow
    const handleSignIn = useCallback(async () => {
        if (!address) {
            toast.error("No wallet connected");
            return;
        }

        try {
            setIsSigning(true);

            // Step 1: Authenticate with backend (get tokens)
            const result = await authenticateWallet(address, async (message) => {
                return await signMessageAsync({ message });
            });

            // Step 2: Sync the result to SharedAuthContext (persists state)
            await authenticateWithDirectApi({
                wallet_address: result.wallet_address,
                permissions: result.permissions,
                is_new_user: result.is_new_user,
                access_token: result.access_token,
            });

            toast.success("Authentication successful");

            // Hard reload to ensure server re-reads auth cookies
            window.location.reload();

        } catch (error: any) {
            console.error("Sign in failed:", error);

            if (error.message?.includes("User rejected")) {
                toast.error("Signature rejected");
            } else {
                toast.error(error.message || "Authentication failed");
            }
        } finally {
            setIsSigning(false);
        }
    }, [address, signMessageAsync, authenticateWithDirectApi]);

    // Loading state
    if (!hasCheckedAuth || authLoading) {
        return (
            <div className="flex h-64 items-center justify-center p-6 bg-slate-50/50 dark:bg-slate-900/50 rounded-xl border border-dashed border-slate-200 dark:border-slate-800">
                <div className="flex flex-col items-center gap-2">
                    <Loader2 className="h-8 w-8 animate-spin text-primary/50" />
                    <p className="text-sm text-muted-foreground animate-pulse">Verifying access...</p>
                </div>
            </div>
        );
    }

    // If authenticated, render children or show loading (for page refresh)
    if (isAuthenticated) {
        // If no children, show loading spinner (page should reload and display content from server)
        if (!children) {
            return (
                <div className="flex h-64 items-center justify-center p-6 bg-gradient-to-br from-slate-50 via-blue-50 to-indigo-50 dark:from-gray-900 dark:via-gray-900 dark:to-indigo-900 rounded-xl">
                    <div className="flex flex-col items-center gap-4">
                        <div className="relative">
                            <div className="absolute inset-0 blur-xl bg-gradient-to-r from-orange-500/30 to-blue-500/30 rounded-full animate-pulse" />
                            <Loader2 className="h-12 w-12 animate-spin text-orange-500 relative z-10" />
                        </div>
                        <div className="text-center space-y-2">
                            <p className="text-lg font-medium text-slate-700 dark:text-slate-300">
                                Welcome back!
                            </p>
                            <p className="text-sm text-muted-foreground animate-pulse">
                                Loading your dashboard...
                            </p>
                        </div>
                    </div>
                </div>
            );
        }
        return <>{children}</>;
    }

    // Fallback UI
    if (fallback) {
        return <>{fallback}</>;
    }

    // Default Guard UI
    return (
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

            <CardContent className="space-y-6 pt-4">
                {!isConnected ? (
                    <div className="space-y-4">
                        <div className="p-4 bg-white dark:bg-slate-950 rounded-lg border border-slate-100 dark:border-slate-800 shadow-sm">
                            <div className="flex items-start gap-3">
                                <Wallet className="h-5 w-5 text-slate-400 mt-0.5" />
                                <div className="space-y-1">
                                    <p className="text-sm font-medium">Step 1: Connect Wallet</p>
                                    <p className="text-xs text-muted-foreground">
                                        Connect your Web3 wallet to identify yourself.
                                    </p>
                                </div>
                            </div>
                        </div>

                        <Button
                            className="w-full gap-2"
                            onClick={openConnectModal}
                            disabled={!openConnectModal}
                        >
                            <Wallet className="h-4 w-4" />
                            Connect Wallet
                        </Button>
                    </div>
                ) : (
                    <div className="space-y-4">
                        <div className="p-4 bg-white dark:bg-slate-950 rounded-lg border border-slate-100 dark:border-slate-800 shadow-sm">
                            <div className="flex items-start gap-3">
                                <CheckCircle className="h-5 w-5 text-green-500 mt-0.5" />
                                <div className="space-y-1">
                                    <p className="text-sm font-medium">Step 1: Wallet Connected</p>
                                    <p className="text-xs text-muted-foreground font-mono">
                                        {address?.slice(0, 6)}...{address?.slice(-4)}
                                    </p>
                                </div>
                            </div>
                        </div>

                        <div className="p-4 bg-orange-100/50 dark:bg-orange-900/20 rounded-lg border border-orange-200 dark:border-orange-800">
                            <div className="flex items-start gap-3">
                                <AlertTriangle className="h-5 w-5 text-orange-600 dark:text-orange-400 mt-0.5" />
                                <div className="space-y-1">
                                    <p className="text-sm font-medium text-orange-800 dark:text-orange-300">
                                        Step 2: Verify Ownership
                                    </p>
                                    <p className="text-xs text-orange-700/80 dark:text-orange-400/80">
                                        Please sign a message to prove you own this wallet. This does not cost any gas.
                                    </p>
                                </div>
                            </div>
                        </div>

                        <Button
                            className="w-full gap-2 bg-orange-600 hover:bg-orange-700 text-white"
                            onClick={handleSignIn}
                            disabled={isSigning}
                        >
                            {isSigning ? (
                                <>
                                    <Loader2 className="h-4 w-4 animate-spin" />
                                    Verifying...
                                </>
                            ) : (
                                <>
                                    <Shield className="h-4 w-4" />
                                    Sign to Access
                                </>
                            )}
                        </Button>
                    </div>
                )}

                {debugInfo && (
                    <details className="text-xs text-muted-foreground mt-4 pt-4 border-t border-slate-100 dark:border-slate-800">
                        <summary className="cursor-pointer hover:text-foreground">Debug Info</summary>
                        <pre className="mt-2 p-2 bg-slate-100 dark:bg-slate-900 rounded overflow-auto max-h-40">
                            {JSON.stringify({
                                ...debugInfo,
                                clientState: {
                                    isConnected,
                                    isAuthenticated,
                                    hasAddress: !!address
                                }
                            }, null, 2)}
                        </pre>
                    </details>
                )}
            </CardContent>
        </Card>
    );
}
