'use client';

import { LogIn } from 'lucide-react';
import { useSharedAuth } from './Provider';

export interface AuthBannerProps {
    message?: string;
    buttonText?: string;
    description?: string;
    onSignIn?: () => void;
}

export function AuthBanner({
    message = 'Sign in to unlock full access',
    buttonText = 'Sign In',
    description = 'Get access to all rankings and premium features',
    onSignIn
}: AuthBannerProps) {
    const { isAuthenticated, openSignInModal } = useSharedAuth();

    if (isAuthenticated) {
        return null;
    }

    return (
        <div className="mx-auto mb-6 max-w-7xl rounded-xl border border-purple-500/20 bg-purple-500/5 p-4 text-center backdrop-blur-sm">
            <div className="flex flex-col items-center gap-3 sm:flex-row sm:justify-between">
                <div className="flex items-center gap-2">
                    <div className="flex h-8 w-8 items-center justify-center rounded-full bg-purple-500/20">
                        <LogIn className="h-4 w-4 text-purple-400" />
                    </div>
                    <div className="text-left">
                        <p className="text-sm font-medium text-purple-300">{message}</p>
                        <p className="text-xs text-purple-400/70">{description}</p>
                    </div>
                </div>
                <button
                    onClick={onSignIn || openSignInModal}
                    className="inline-flex items-center gap-2 rounded-lg bg-purple-500 px-4 py-2 text-sm font-medium text-white transition-colors hover:bg-purple-600 focus:outline-none focus:ring-2 focus:ring-purple-500 focus:ring-offset-2 focus:ring-offset-slate-900"
                >
                    <LogIn className="h-4 w-4" />
                    {buttonText}
                </button>
            </div>
        </div>
    );
}

export default AuthBanner;
