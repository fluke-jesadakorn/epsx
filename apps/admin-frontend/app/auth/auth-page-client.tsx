'use client';

import { getAndClearReturnUrlAction } from '@/lib/auth/auth-actions';
import { AuthModal } from '@/shared/components/auth';
import { UnifiedThemeToggle } from '@/shared/components/ui/theme-toggle';
import { logger } from '@/shared/utils/logger';
import {
    Activity,
    CheckCircle,
    LayoutDashboard,
    Lock,
    Shield,
    ShieldCheck,
} from 'lucide-react';
import Link from 'next/link';
import { useRouter, useSearchParams } from 'next/navigation';
import { useEffect, useRef, useState } from 'react';
import { useAccount } from 'wagmi';

const features = [
    { icon: ShieldCheck, title: 'Role-Based Access', desc: 'Granular permission controls for every admin role.' },
    { icon: Activity, title: 'Audit Logging', desc: 'Complete activity trail for compliance and security.' },
    { icon: Lock, title: 'Wallet Verification', desc: 'Web3-native authentication with SIWE protocol.' },
    { icon: LayoutDashboard, title: 'Admin Dashboard', desc: 'Full platform control from a single interface.' },
];

const benefits = [
    'Secure Web3 Admin Login',
    'No Account Credentials Needed',
    'Role-Based Access Control',
];

export default function AuthPageClient() {
    const router = useRouter();
    const searchParams = useSearchParams();
    const [showModal, setShowModal] = useState(false);
    const redirectingRef = useRef(false);
    const reason = searchParams.get('reason');
    const { status, isConnected } = useAccount();
    const [timedOut, setTimedOut] = useState(false);
    const wagmiReady = status !== 'reconnecting' && status !== 'connecting';
    const isWagmiReady = wagmiReady || timedOut;

    // Timeout: if wagmi reconnection takes >3s, unblock the UI
    useEffect(() => {
        if (wagmiReady) { return; }
        const t = setTimeout(() => { setTimedOut(true); }, 3000);
        return () => { clearTimeout(t); };
    }, [wagmiReady]);

    // Auto-open modal if wallet already connected
    useEffect(() => {
        if (isWagmiReady && isConnected && !showModal) {
            setShowModal(true);
        }
    }, [isWagmiReady, isConnected]); // eslint-disable-line react-hooks/exhaustive-deps

    const handleAuthSuccess = async () => {
        try {
            if (redirectingRef.current) {return;}
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

    return (
        <div className="fixed inset-0 z-50 flex w-full flex-col overflow-y-auto bg-background lg:flex-row">
            {/* Theme Toggle */}
            <div className="absolute top-4 right-4 z-20">
                <UnifiedThemeToggle variant="minimal" size="sm" showTooltip={false} />
            </div>

            {/* Animated background blobs */}
            <div className="pointer-events-none absolute inset-0 z-0 overflow-hidden">
                <div className="absolute left-[-10%] top-[-10%] h-[60%] w-[60%] animate-pulse rounded-full bg-purple-600/10 blur-[120px]" />
                <div className="absolute bottom-[-10%] right-[-10%] h-[60%] w-[60%] animate-pulse rounded-full bg-orange-600/10 blur-[120px]" style={{ animationDelay: '1s' }} />
                <div className="absolute right-[10%] top-[20%] h-[40%] w-[40%] animate-pulse rounded-full bg-purple-800/10 blur-[100px]" style={{ animationDelay: '2s' }} />
            </div>

            {/* Left Panel - Desktop branding */}
            <div className="relative z-10 hidden w-full flex-col justify-center overflow-hidden p-8 text-foreground dark:text-foreground lg:flex lg:w-3/5 xl:p-20">
                <div className="mb-12 animate-auth-fade-in">
                    <div className="mb-8 flex items-center gap-3">
                        <div className="rounded-2xl bg-gradient-to-br from-purple-500 to-orange-500 p-3 shadow-2xl shadow-purple-500/20 ring-1 ring-white/20 transition-transform hover:scale-105">
                            <Shield className="h-8 w-8 text-white" />
                        </div>
                        <span className="text-4xl font-black uppercase italic tracking-tighter">EPSX ADMIN</span>
                    </div>

                    <h1 className="text-5xl font-bold leading-tight tracking-tight xl:text-7xl">
                        Admin{' '}
                        <span className="animate-auth-gradient bg-gradient-to-r from-purple-400 via-orange-400 to-purple-500 bg-clip-text text-transparent">
                            Control Center
                        </span>
                    </h1>
                    <p className="mt-6 max-w-xl text-lg leading-relaxed text-slate-400 xl:text-xl">
                        Manage the entire EPSX platform. Role-based access, audit logging, wallet administration, and complete operational control.
                    </p>
                </div>

                <div className="grid max-w-2xl gap-8 sm:grid-cols-2">
                    {features.map((f, i) => (
                        <div key={i} className="group flex gap-4 animate-auth-slide-up" style={{ animationDelay: `${i * 100}ms` }}>
                            <div className="flex h-12 w-12 shrink-0 items-center justify-center rounded-xl border border-border/20 bg-white dark:bg-white/[0.04] ring-1 ring-white/5 transition-all duration-300 group-hover:border-purple-500/20 group-hover:bg-purple-500/10">
                                <f.icon className="h-6 w-6 text-purple-400 transition-transform group-hover:scale-110" />
                            </div>
                            <div>
                                <h3 className="text-lg font-bold text-foreground/90 dark:text-foreground/90">{f.title}</h3>
                                <p className="mt-1 text-sm leading-snug text-slate-500">{f.desc}</p>
                            </div>
                        </div>
                    ))}
                </div>
            </div>

            {/* Right Panel - Auth card */}
            <div className="relative z-10 flex w-full items-center justify-center p-4 sm:p-6 lg:w-2/5 lg:border-l lg:border-border/20 lg:bg-white/[0.02] lg:backdrop-blur-3xl">
                <div className="w-full max-w-md">
                    {/* Mobile header */}
                    <div className="mb-8 mt-4 animate-auth-fade-in text-center text-foreground dark:text-foreground sm:mb-10 sm:mt-6 lg:hidden">
                        <div className="mb-4 flex items-center justify-center gap-2 sm:mb-6">
                            <div className="rounded-xl bg-gradient-to-br from-purple-500 to-orange-500 p-2 shadow-xl shadow-purple-500/20 ring-1 ring-white/20 sm:rounded-2xl sm:p-3">
                                <Shield className="h-8 w-8 text-white sm:h-10 sm:w-10" />
                            </div>
                            <span className="text-2xl font-black uppercase italic tracking-tighter sm:text-3xl">EPSX ADMIN</span>
                        </div>
                        <h2 className="mb-2 text-3xl font-bold tracking-tight sm:text-4xl">Admin Portal</h2>
                        <p className="mt-2 px-4 text-base text-slate-400 sm:text-lg">Connect your wallet to access admin controls</p>
                    </div>

                    {/* Auth card with glassmorphism */}
                    <div className="group relative animate-auth-slide-up overflow-hidden rounded-2xl border border-border/20 bg-white/90 dark:bg-background/60 p-6 shadow-[0_24px_48px_-12px_rgba(0,0,0,0.5)] sm:rounded-3xl sm:p-8 lg:p-10">
                        {/* Shimmer line */}
                        <div className="absolute left-0 top-0 h-px w-full -translate-x-full bg-gradient-to-r from-transparent via-purple-500/50 to-transparent animate-[auth-shimmer_3s_infinite]" />

                        <div className="relative">
                            {/* Desktop title */}
                            <div className="mb-8 hidden text-center lg:mb-10 lg:block">
                                <div className="mx-auto mb-6 flex h-16 w-16 items-center justify-center rounded-2xl bg-purple-500/10 shadow-[0_0_40px_-10px_rgba(139,92,246,0.3)] ring-1 ring-purple-500/20 transition-transform hover:scale-105 lg:h-20 lg:w-20 lg:rounded-[2rem]">
                                    <Lock className="h-8 w-8 text-purple-400 lg:h-10 lg:w-10" />
                                </div>
                                <h2 className="mb-2 text-2xl font-bold tracking-tight text-foreground dark:text-foreground lg:text-3xl">Admin Portal</h2>
                                <p className="text-sm font-medium text-slate-500 lg:text-base">Secure authentication via Web3</p>
                            </div>

                            {/* Mobile icon */}
                            <div className="mb-6 text-center lg:hidden">
                                <div className="mx-auto mb-4 flex h-16 w-16 items-center justify-center rounded-2xl bg-purple-500/10 shadow-[0_0_40px_-10px_rgba(139,92,246,0.3)] ring-1 ring-purple-500/20">
                                    <Lock className="h-8 w-8 text-purple-400" />
                                </div>
                            </div>

                            <div className="space-y-5 sm:space-y-6">
                                {/* Reason banner */}
                                {reason != null && reason !== '' && (
                                    <div className="rounded-xl border border-red-500/20 bg-red-500/10 px-4 py-3 text-center text-sm text-red-300">
                                        {reason === 'no-session' && 'Your session has expired. Please sign in again.'}
                                        {reason === 'no-admin-permissions' && 'Admin permissions required.'}
                                    </div>
                                )}

                                {/* Connect Wallet button */}
                                <button
                                    type="button"
                                    onClick={() => { setShowModal(true); }}
                                    disabled={!isWagmiReady}
                                    className="flex w-full items-center justify-center gap-3 rounded-xl bg-gradient-to-r from-purple-500 to-orange-500 py-5 text-lg font-bold text-white shadow-2xl shadow-purple-500/10 transition-all hover:from-purple-600 hover:to-orange-600 hover:shadow-purple-500/20 active:scale-[0.98] disabled:opacity-70 disabled:cursor-not-allowed sm:rounded-2xl sm:py-6 sm:text-xl"
                                >
                                    {!isWagmiReady ? (
                                        <>
                                            <div className="h-5 w-5 animate-spin rounded-full border-2 border-white/30 border-t-white" />
                                            Loading...
                                        </>
                                    ) : (
                                        <>
                                            <Lock className="h-5 w-5" />
                                            Connect Wallet
                                        </>
                                    )}
                                </button>

                                {/* Benefits */}
                                <div className="border-t border-border/20 pt-5 sm:pt-6">
                                    <div className="grid grid-cols-1 gap-3 sm:gap-4">
                                        {benefits.map((text, i) => (
                                            <div key={i} className="flex items-center gap-3 text-xs font-medium text-slate-400 sm:text-sm">
                                                <div className="flex h-5 w-5 shrink-0 items-center justify-center rounded-full bg-purple-500/10 ring-1 ring-purple-500/20">
                                                    <CheckCircle className="h-3 w-3 text-purple-400" />
                                                </div>
                                                {text}
                                            </div>
                                        ))}
                                    </div>
                                </div>

                                {/* Mobile features grid */}
                                <div className="border-t border-border/20 pt-5 sm:pt-6 lg:hidden">
                                    <div className="grid grid-cols-2 gap-3 sm:gap-4">
                                        {features.map((f, i) => (
                                            <div key={i} className="flex flex-col items-center rounded-xl border border-border/20 bg-white/[0.02] p-3 text-center transition-colors hover:bg-white/[0.04] sm:p-4">
                                                <div className="mb-2 flex h-10 w-10 items-center justify-center rounded-lg bg-purple-500/10 ring-1 ring-purple-500/20">
                                                    <f.icon className="h-5 w-5 text-purple-400" />
                                                </div>
                                                <h4 className="text-xs font-semibold text-foreground/90 dark:text-foreground/90">{f.title}</h4>
                                            </div>
                                        ))}
                                    </div>
                                </div>
                            </div>

                            <div className="mt-8 text-center sm:mt-10 lg:mt-12">
                                <p className="mx-auto max-w-[280px] px-4 text-[10px] leading-relaxed text-slate-500/80 sm:text-xs">
                                    By connecting, you agree to our{' '}
                                    <a href="/terms" className="text-muted-foreground underline underline-offset-2 transition-colors hover:text-foreground">Terms</a>
                                    {' '}and{' '}
                                    <a href="/privacy" className="text-muted-foreground underline underline-offset-2 transition-colors hover:text-foreground">Privacy</a>.
                                </p>
                            </div>
                        </div>
                    </div>

                    {/* AuthModal portal */}
                    <AuthModal
                        isOpen={showModal}
                        onClose={() => { setShowModal(false); }}
                        variant="admin"
                        onSuccess={() => { void handleAuthSuccess(); }}
                    />

                    {/* Status indicator */}
                    <div className="mt-6 flex items-center justify-center gap-2 text-[9px] font-bold uppercase tracking-[0.2em] text-slate-600 sm:mt-8 sm:text-[10px]">
                        <div className="h-1 w-1 animate-pulse rounded-full bg-green-500" />
                        <span className="hidden sm:inline">Admin Network Secure</span>
                        <span className="sm:hidden">Secure Connection</span>
                    </div>

                    {/* Manual redirect fallback */}
                    <div className="mt-4 text-center">
                        <Link href="/" className="text-xs text-slate-500 hover:text-purple-500 transition-colors underline underline-offset-2">
                            Go to Dashboard
                        </Link>
                    </div>
                </div>
            </div>
        </div>
    );
}
