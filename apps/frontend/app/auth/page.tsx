'use client';

import { WalletConnectAuth } from '@/components/auth/wallet-connect-auth';
import { useSharedAuth } from '@/shared/components/auth';
import { UnifiedThemeToggle } from '@/shared/components/ui/theme-toggle';
import { CheckCircle, Cpu, Database, Globe, Lock, ShieldCheck, Zap } from 'lucide-react';
import Link from 'next/link';
import { useRouter, useSearchParams } from 'next/navigation';
import { Suspense, useEffect } from 'react';
import { toast } from 'sonner';
import { useAccount } from 'wagmi';

const features = [
    { icon: Database, title: "Data Accuracy", desc: "Institutional-grade precision for every metric." },
    { icon: Zap, title: "Real-time Edge", desc: "Stay ahead of the curve with instant updates." },
    { icon: ShieldCheck, title: "Secure Ownership", desc: "Your data, your identity, through Web3." },
    { icon: Globe, title: "Global Coverage", desc: "Comprehensive coverage across all data sources." }
];

const benefits = [
    "Secure Web3 Login Flow",
    "No Account Credentials Needed",
    "Decentralized Data Privacy"
];

function AuthContent() {
    const router = useRouter();
    const searchParams = useSearchParams();
    const returnUrl = searchParams.get('return_url') ?? '/';
    const { isAuthenticated, user } = useSharedAuth();
    const { isConnected } = useAccount();

    // Auto-redirect only when both authenticated AND wallet connected
    // Prevents bounce-back when cookies exist but wagmi session expired
    useEffect(() => {
        if (isAuthenticated && user && isConnected) {
            router.push(returnUrl);
            router.refresh();
        }
    }, [isAuthenticated, user, isConnected, returnUrl, router]);

    const handleAuthSuccess = (_walletAddress: string) => {
        toast.success('Authenticated successfully');
    };

    return (
        <div className="relative flex min-h-screen w-full flex-col overflow-hidden bg-background lg:flex-row">
            {/* Theme Toggle */}
            <div className="absolute top-4 right-4 z-20">
                <UnifiedThemeToggle variant="minimal" size="sm" showTooltip={false} />
            </div>

            {/* Animated Background */}
            <div className="absolute inset-0 z-0 overflow-hidden pointer-events-none">
                <div className="absolute top-[-10%] left-[-10%] h-[60%] w-[60%] rounded-full bg-orange-600/10 blur-[120px] animate-pulse" />
                <div className="absolute bottom-[-10%] right-[-10%] h-[60%] w-[60%] rounded-full bg-purple-600/10 blur-[120px] animate-pulse" style={{ animationDelay: '1s' }} />
                <div className="absolute top-[20%] right-[10%] h-[40%] w-[40%] rounded-full bg-blue-600/10 blur-[100px] animate-pulse" style={{ animationDelay: '2s' }} />
            </div>

            {/* Left Side: Desktop Brand */}
            <div className="relative z-10 hidden w-full flex-col justify-center p-8 text-foreground dark:text-white lg:flex lg:w-3/5 xl:p-20 overflow-hidden">
                <div className="mb-12 animate-fade-in">
                    <div className="flex items-center gap-3 mb-8">
                        <div className="rounded-2xl bg-gradient-to-br from-orange-500 to-purple-600 p-3 shadow-2xl shadow-orange-500/20 ring-1 ring-white/20 transition-transform hover:scale-105">
                            <Cpu className="h-8 w-8 text-white" />
                        </div>
                        <span className="text-4xl font-black tracking-tighter uppercase italic">EPSX</span>
                    </div>

                    <h1 className="text-5xl font-bold leading-tight xl:text-7xl tracking-tight">
                        Precision <span className="text-transparent bg-clip-text bg-gradient-to-r from-orange-400 via-yellow-400 to-orange-500 animate-gradient">Analytics</span> <br />
                        For Modern Teams
                    </h1>
                    <p className="mt-6 text-lg xl:text-xl text-slate-400 max-w-xl leading-relaxed">
                        Join the next generation of data intelligence. Real-time metrics, predictive modeling, and institutional-grade insights at your fingertips.
                    </p>
                </div>

                <div className="grid gap-8 sm:grid-cols-2 max-w-2xl">
                    {features.map((feature, i) => (
                        <div key={i} className="flex gap-4 group animate-slide-up" style={{ animationDelay: `${i * 100}ms` }}>
                            <div className="flex h-12 w-12 shrink-0 items-center justify-center rounded-xl bg-white dark:bg-white/5 border border-gray-200 dark:border-slate-700 ring-1 ring-white/5 backdrop-blur-sm group-hover:bg-orange-500/10 group-hover:border-orange-500/20 transition-all duration-300">
                                <feature.icon className="h-6 w-6 text-orange-400 group-hover:scale-110 transition-transform" />
                            </div>
                            <div>
                                <h3 className="font-bold text-lg text-foreground/90 dark:text-white/90">{feature.title}</h3>
                                <p className="text-slate-500 text-sm mt-1 leading-snug">{feature.desc}</p>
                            </div>
                        </div>
                    ))}
                </div>

                <div className="mt-16 flex items-center gap-6 py-6 border-t border-gray-200 dark:border-slate-700 max-w-md">
                    <div className="flex -space-x-2">
                        {[1, 2, 3, 4].map((i) => (
                            <div key={i} className="h-9 w-9 rounded-full border-2 border-background bg-gray-100 dark:bg-slate-800 flex items-center justify-center text-[10px] font-black tracking-widest text-slate-400 ring-1 ring-white/10 overflow-hidden transition-transform hover:scale-110">
                                <div className="w-full h-full bg-gradient-to-br from-slate-700 to-white dark:to-slate-900 flex items-center justify-center">
                                    {String.fromCharCode(64 + i)}
                                </div>
                            </div>
                        ))}
                    </div>
                    <p className="text-sm text-slate-400 font-medium">
                        Powering <span className="text-foreground dark:text-white font-bold text-base px-1">2,500+</span> teams worldwide
                    </p>
                </div>
            </div>

            {/* Right Side: Auth Form */}
            <div className="relative z-10 flex w-full items-center justify-center p-4 sm:p-6 lg:w-2/5 lg:bg-white/[0.02] lg:backdrop-blur-3xl lg:border-l lg:border-gray-200 dark:border-slate-700">
                <div className="w-full max-w-md">
                    {/* Mobile Header */}
                    <div className="lg:hidden mb-8 sm:mb-10 text-center text-foreground dark:text-white mt-4 sm:mt-6 animate-fade-in">
                        <div className="flex items-center justify-center gap-2 mb-4 sm:mb-6">
                            <div className="rounded-xl sm:rounded-2xl bg-gradient-to-br from-orange-500 to-purple-600 p-2 sm:p-3 shadow-xl shadow-orange-500/20 ring-1 ring-white/20">
                                <Cpu className="h-8 w-8 sm:h-10 sm:w-10 text-white" />
                            </div>
                            <span className="text-2xl sm:text-3xl font-black italic tracking-tighter uppercase">EPSX</span>
                        </div>
                        <h2 className="text-3xl sm:text-4xl font-bold tracking-tight mb-2">Welcome Back</h2>
                        <p className="text-slate-400 mt-2 text-base sm:text-lg px-4">Connect your wallet to access the platform</p>
                    </div>

                    {/* Auth Card */}
                    <div className="card-insight-enhanced group relative overflow-hidden bg-white/90 dark:bg-slate-950/60 border-gray-200 dark:border-slate-700 p-6 sm:p-8 lg:p-10 shadow-[0_24px_48px_-12px_rgba(0,0,0,0.5)] backdrop-blur-3xl rounded-2xl sm:rounded-3xl animate-slide-up">
                        {/* Shimmer Effect */}
                        <div className="absolute top-0 left-0 w-full h-px bg-gradient-to-r from-transparent via-orange-500/50 to-transparent -translate-x-full animate-[shimmer_3s_infinite]" />

                        <div className="relative">
                            {/* Desktop Title */}
                            <div className="mb-8 lg:mb-10 text-center hidden lg:block">
                                <div className="mx-auto mb-6 flex h-16 w-16 lg:h-20 lg:w-20 items-center justify-center rounded-2xl lg:rounded-[2rem] bg-orange-500/10 ring-1 ring-orange-500/20 shadow-[0_0_40px_-10px_rgba(249,115,22,0.3)] transition-transform hover:scale-105">
                                    <Lock className="h-8 w-8 lg:h-10 lg:w-10 text-orange-500" />
                                </div>
                                <h2 className="text-2xl lg:text-3xl font-bold text-foreground dark:text-white mb-2 tracking-tight">Welcome to EPSX</h2>
                                <p className="text-slate-500 font-medium text-sm lg:text-base">Secure authentication via Web3</p>
                            </div>

                            {/* Mobile Icon */}
                            <div className="mb-6 text-center lg:hidden">
                                <div className="mx-auto mb-4 flex h-16 w-16 items-center justify-center rounded-2xl bg-orange-500/10 ring-1 ring-orange-500/20 shadow-[0_0_40px_-10px_rgba(249,115,22,0.3)]">
                                    <Lock className="h-8 w-8 text-orange-500" />
                                </div>
                            </div>

                            <div className="space-y-5 sm:space-y-6">
                                <WalletConnectAuth
                                    onAuthSuccess={handleAuthSuccess}
                                    className="w-full justify-center py-5 sm:py-6 text-lg sm:text-xl font-bold shadow-2xl shadow-orange-500/10 hover:shadow-orange-500/20 active:scale-[0.98] transition-all rounded-xl sm:rounded-2xl border-none bg-gradient-to-r from-orange-500 to-amber-500 hover:from-orange-600 hover:to-amber-600"
                                />

                                <div className="pt-5 sm:pt-6 border-t border-gray-200 dark:border-slate-700">
                                    <div className="grid grid-cols-1 gap-3 sm:gap-4">
                                        {benefits.map((text, i) => (
                                            <div key={i} className="flex items-center gap-3 text-xs sm:text-sm text-slate-400 font-medium">
                                                <div className="flex h-5 w-5 shrink-0 items-center justify-center rounded-full bg-orange-500/10 ring-1 ring-orange-500/20">
                                                    <CheckCircle className="h-3 w-3 text-orange-500" />
                                                </div>
                                                {text}
                                            </div>
                                        ))}
                                    </div>
                                </div>

                                {/* Mobile Features */}
                                <div className="lg:hidden pt-5 sm:pt-6 border-t border-gray-200 dark:border-slate-700">
                                    <div className="grid grid-cols-2 gap-3 sm:gap-4">
                                        {features.map((feature, i) => (
                                            <div key={i} className="flex flex-col items-center text-center p-3 sm:p-4 rounded-xl bg-white/[0.02] border border-gray-200 dark:border-slate-700 hover:bg-white/[0.04] transition-colors">
                                                <div className="flex h-10 w-10 items-center justify-center rounded-lg bg-orange-500/10 ring-1 ring-orange-500/20 mb-2">
                                                    <feature.icon className="h-5 w-5 text-orange-400" />
                                                </div>
                                                <h4 className="font-semibold text-xs text-foreground/90 dark:text-white/90">{feature.title}</h4>
                                            </div>
                                        ))}
                                    </div>
                                </div>
                            </div>

                            <div className="mt-8 sm:mt-10 lg:mt-12 text-center">
                                <p className="text-[10px] sm:text-xs text-slate-500/80 leading-relaxed max-w-[280px] mx-auto px-4">
                                    By connecting, you agree to our{' '}
                                    <a href="/terms" className="text-muted-foreground hover:text-foreground underline underline-offset-2 transition-colors">Terms</a>
                                    {' '}and{' '}
                                    <a href="/privacy" className="text-muted-foreground hover:text-foreground underline underline-offset-2 transition-colors">Privacy</a>.
                                </p>
                            </div>
                        </div>
                    </div>

                    {/* Status Indicator */}
                    <div className="mt-6 sm:mt-8 flex items-center justify-center gap-2 text-[9px] sm:text-[10px] uppercase tracking-[0.2em] font-bold text-slate-600">
                        <div className="h-1 w-1 rounded-full bg-green-500 animate-pulse" />
                        <span className="hidden sm:inline">Network Secure & Operational</span>
                        <span className="sm:hidden">Secure Connection</span>
                    </div>

                    {/* Manual redirect fallback */}
                    <div className="mt-4 text-center">
                        <Link href="/" className="text-xs text-slate-500 hover:text-orange-500 transition-colors underline underline-offset-2">
                            Go to Homepage
                        </Link>
                    </div>
                </div>
            </div>
        </div>
    );
}

export default function AuthPage() {
    return (
        <Suspense fallback={
            <div className="min-h-screen bg-background flex items-center justify-center">
                <div className="animate-spin rounded-full h-8 w-8 border-t-2 border-orange-500"></div>
            </div>
        }>
            <AuthContent />
        </Suspense>
    );
}
