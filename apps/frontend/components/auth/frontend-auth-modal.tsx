'use client';

import { Wallet } from 'lucide-react';
import { useRouter } from 'next/navigation';
import { useEffect } from 'react';

import { useSharedAuth } from '@/shared/components/auth';
import {
    Dialog,
    DialogContent,
    DialogDescription,
    DialogHeader,
    DialogTitle,
} from '@/shared/components/ui/dialog';

export function FrontendAuthModal() {
    const { isAuthenticated, showSignInModal, closeSignInModal } = useSharedAuth();
    const router = useRouter();

    useEffect(() => {
        if (isAuthenticated) {
            closeSignInModal();
        }
    }, [isAuthenticated, closeSignInModal]);

    const handleConnectWallet = () => {
        router.push('/auth');
    };

    if (isAuthenticated) {
        return null;
    }

    return (
        <Dialog open={showSignInModal} onOpenChange={closeSignInModal}>
            <DialogContent className="sm:max-w-md" showClose={true}>
                <DialogHeader>
                    <div className="flex items-center justify-center w-12 h-12 mx-auto mb-4 rounded-full bg-gradient-to-br from-purple-500/20 to-pink-500/20">
                        <Wallet className="w-6 h-6 text-purple-400" />
                    </div>
                    <DialogTitle className="text-center text-xl">Sign In Required</DialogTitle>
                    <DialogDescription className="text-center">
                        Sign in with your wallet to access full rankings and premium features
                    </DialogDescription>
                </DialogHeader>
                <div className="flex flex-col gap-3 pt-4">
                    <button
                        onClick={handleConnectWallet}
                        className="inline-flex items-center justify-center rounded-lg bg-gradient-to-r from-purple-500 to-pink-500 px-4 py-3 text-sm font-semibold text-white shadow-lg transition-all hover:from-purple-600 hover:to-pink-600 focus:outline-none focus:ring-2 focus:ring-purple-500 focus:ring-offset-2 focus:ring-offset-slate-900"
                    >
                        <Wallet className="mr-2 h-4 w-4" />
                        Sign In with Wallet
                    </button>
                    <button
                        onClick={closeSignInModal}
                        className="inline-flex items-center justify-center rounded-lg border border-slate-700 bg-gray-100 dark:bg-slate-800/50 px-4 py-3 text-sm font-medium text-slate-300 transition-colors hover:bg-gray-100 dark:bg-slate-800 focus:outline-none focus:ring-2 focus:ring-slate-600 focus:ring-offset-2 focus:ring-offset-slate-900"
                    >
                        Maybe Later
                    </button>
                </div>
                <p className="mt-4 text-center text-xs text-slate-500">
                    By signing in, you agree to our Terms of Service and Privacy Policy
                </p>
            </DialogContent>
        </Dialog>
    );
}
