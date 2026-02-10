'use client';

import { AuthModal, useSharedAuth } from '@/shared/components/auth';
import { Loader2, Lock, Shield, Wallet } from 'lucide-react';
import { useEffect, useState } from 'react';
import { useAccount } from 'wagmi';

export function PaymentAuthGuard() {
    const { isAuthenticated, isLoading } = useSharedAuth();
    const { isConnected } = useAccount();
    const [showModal, setShowModal] = useState(false);
    const [hasCheckedAuth, setHasCheckedAuth] = useState(false);

    useEffect(() => {
        if (!isLoading) {
            setHasCheckedAuth(true);
        }
    }, [isLoading]);

    // Loading state
    if (!hasCheckedAuth || isLoading) {
        return (
            <div className="flex flex-col items-center justify-center py-20">
                <Loader2 className="h-8 w-8 animate-spin text-primary/50 mb-4" />
                <p className="text-sm text-muted-foreground animate-pulse">Verifying access...</p>
            </div>
        );
    }

    // This component is only rendered when NOT authenticated (parent checks this),
    // but we double check here to be safe and avoid flashing
    if (isAuthenticated) {
        return null;
    }

    return (
        <div className="w-full max-w-md mx-auto">
            {/* Main Card */}
            <div className="bg-[#1A1D24] border border-gray-800 rounded-2xl p-8 text-center shadow-xl">
                {/* Icon Circle */}
                <div className="mx-auto bg-[#2A2D35] w-20 h-20 rounded-full flex items-center justify-center mb-6 border border-gray-700">
                    <Wallet className="h-10 w-10 text-orange-500" />
                </div>

                <h2 className="text-2xl font-bold text-white mb-2">Sign In to Continue</h2>
                <p className="text-gray-400 mb-8">
                    {isConnected
                        ? 'Please sign the message to verify your wallet.'
                        : 'Please connect your wallet to proceed with payment.'}
                </p>

                {/* Features List */}
                <div className="bg-[#13161B] rounded-xl p-4 text-left space-y-4 mb-8 border border-gray-800/50">
                    <div className="flex items-start gap-3">
                        <div className="mt-1">
                            <Shield className="h-5 w-5 text-green-500" />
                        </div>
                        <div>
                            <h3 className="text-white font-medium text-sm">Secure Authentication</h3>
                            <p className="text-gray-500 text-xs mt-0.5">Sign-In with Ethereum (SIWE)</p>
                        </div>
                    </div>

                    <div className="flex items-start gap-3">
                        <div className="mt-1">
                            <Lock className="h-5 w-5 text-blue-500" />
                        </div>
                        <div>
                            <h3 className="text-white font-medium text-sm">One-Time Sign In</h3>
                            <p className="text-gray-500 text-xs mt-0.5">Stay signed in until you disconnect</p>
                        </div>
                    </div>
                </div>

                {/* Main Action Button */}
                <button
                    onClick={() => setShowModal(true)}
                    className="w-full py-3.5 bg-blue-600 hover:bg-blue-700 text-white font-semibold rounded-xl transition-all shadow-lg hover:shadow-blue-900/20 active:scale-[0.98]"
                >
                    {isConnected ? 'Sign In' : 'Connect Wallet'}
                </button>
            </div>

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
        </div>
    );
}
