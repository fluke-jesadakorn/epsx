'use client';

import { BarChart2, Lock, LogIn, TrendingUp, Zap } from 'lucide-react';
import { useSharedAuth } from './Provider';

export interface AuthBannerProps {
    message?: string;
    buttonText?: string;
    description?: string;
    onSignIn?: () => void;
}

export function AuthBanner({
    message = 'Unlock Full Analytics Access',
    buttonText = 'Sign In Free',
    description = 'Get access to all rankings and premium features',
    onSignIn
}: AuthBannerProps) {
    const { isAuthenticated, openSignInModal } = useSharedAuth();

    if (isAuthenticated) {
        return null;
    }

    return (
        <div className="relative mb-6 overflow-hidden rounded-2xl border border-purple-500/30 bg-gradient-to-r from-purple-900/40 via-purple-800/30 to-pink-900/40 backdrop-blur-sm">
            {/* Decorative glow */}
            <div className="pointer-events-none absolute -top-10 -right-10 h-40 w-40 rounded-full bg-purple-500/20 blur-3xl" />
            <div className="pointer-events-none absolute -bottom-6 -left-6 h-24 w-24 rounded-full bg-pink-500/20 blur-2xl" />

            <div className="relative flex flex-col gap-4 p-5 sm:flex-row sm:items-center sm:justify-between">
                {/* Left: icon + text + benefits */}
                <div className="flex items-start gap-4">
                    <div className="flex h-12 w-12 shrink-0 items-center justify-center rounded-xl bg-gradient-to-br from-purple-500 to-pink-500 shadow-lg shadow-purple-500/30">
                        <Lock className="h-6 w-6 text-white" />
                    </div>
                    <div>
                        <p className="text-base font-bold text-white">{message}</p>
                        <p className="mt-0.5 text-sm text-purple-300/80">{description}</p>
                        <div className="mt-2 flex flex-wrap gap-3">
                            <span className="flex items-center gap-1 text-xs text-purple-300/70">
                                <TrendingUp className="h-3 w-3 text-purple-400" />
                                Top 100 stock rankings
                            </span>
                            <span className="flex items-center gap-1 text-xs text-purple-300/70">
                                <BarChart2 className="h-3 w-3 text-purple-400" />
                                Real-time EPS data
                            </span>
                            <span className="flex items-center gap-1 text-xs text-purple-300/70">
                                <Zap className="h-3 w-3 text-purple-400" />
                                AI-powered insights
                            </span>
                        </div>
                    </div>
                </div>

                {/* Right: CTA */}
                <button
                    onClick={onSignIn ?? openSignInModal}
                    className="group inline-flex shrink-0 items-center gap-2 rounded-xl bg-gradient-to-r from-purple-500 to-pink-500 px-6 py-3 text-sm font-semibold text-white shadow-lg shadow-purple-500/30 transition-all duration-200 hover:-translate-y-0.5 hover:shadow-xl hover:shadow-purple-500/40 focus:outline-none"
                >
                    <LogIn className="h-4 w-4 transition-transform group-hover:scale-110" />
                    {buttonText}
                </button>
            </div>
        </div>
    );
}

export default AuthBanner;
