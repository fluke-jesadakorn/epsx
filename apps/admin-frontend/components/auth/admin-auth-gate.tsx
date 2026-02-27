'use client';

import '@/shared/components/auth/auth.css';
import { AuthStatusDisplay, ConnectStep, SignStep, SwitchChainStep } from '@/shared/components/auth/auth-modal-components';
import { useAuthModalLogic } from '@/shared/components/auth/hooks/use-auth-modal-logic';
import { TurnstileWidget } from '@/shared/components/turnstile-widget';
import { BarChart3, Key, Lock, Shield, Users } from 'lucide-react';
import { useRouter } from 'next/navigation';
import { useEffect, useState } from 'react';
import { useAccount } from 'wagmi';

const FEATURES = [
    { icon: Shield, title: 'Secure Access', desc: 'Admin-only via Web3 wallet' },
    { icon: Users, title: 'User Management', desc: 'Full control over platform users' },
    { icon: Key, title: 'Permissions', desc: 'Granular permission management' },
    { icon: BarChart3, title: 'Analytics', desc: 'Advanced analytics & reporting' },
];

function BrandPanel() {
    return (
        <div className="relative z-10 hidden w-3/5 flex-col justify-center overflow-hidden p-20 xl:p-24 lg:flex">
            <div className="mb-12">
                <div className="mb-8 flex items-center gap-3">
                    <div className="rounded-2xl bg-gradient-to-br from-[#7645d9] to-[#1fc7d4] p-3 shadow-2xl shadow-purple-500/20 ring-1 ring-white/20 transition-transform hover:scale-105">
                        <Lock className="h-8 w-8 text-white" />
                    </div>
                    <span className="text-4xl font-black italic uppercase tracking-tighter">EPSX</span>
                </div>
                <h1 className="text-5xl font-bold leading-tight tracking-tight xl:text-7xl">
                    Admin{' '}
                    <span className="bg-gradient-to-r from-[#7645d9] via-[#1fc7d4] to-[#7645d9] bg-clip-text text-transparent">
                        Control
                    </span>
                    <br />
                    Panel
                </h1>
                <p className="mt-6 max-w-xl text-lg leading-relaxed text-slate-400">
                    Restricted access. Connect your admin wallet to manage users, permissions, and platform analytics.
                </p>
            </div>

            <div className="grid max-w-2xl gap-8 sm:grid-cols-2">
                {FEATURES.map((feature) => (
                    <div key={feature.title} className="group flex gap-4">
                        <div className="flex h-12 w-12 shrink-0 items-center justify-center rounded-xl border border-border/20 bg-white/5 transition-all group-hover:border-purple-500/20 group-hover:bg-purple-500/10">
                            <feature.icon className="h-6 w-6 text-[#7645d9] transition-transform group-hover:scale-110" />
                        </div>
                        <div>
                            <h3 className="text-lg font-bold">{feature.title}</h3>
                            <p className="mt-1 text-sm leading-snug text-slate-500">{feature.desc}</p>
                        </div>
                    </div>
                ))}
            </div>
        </div>
    );
}

function AuthPanel() {
    const router = useRouter();
    const [mounted, setMounted] = useState(false);
    useEffect(() => { setMounted(true); }, []);

    // Wait for wagmi to finish reconnecting before opening the auth flow.
    // Without this, the hook auto-signs immediately while the connector is
    // still initializing → "Wallet still initializing" error.
    const { status } = useAccount();
    const isOpen = mounted && (status === 'connected' || status === 'disconnected');

    const {
        step, error, isSigning, isConnecting, isSwitching,
        address, connectors, connect,
        handleSwitchChain, handleSign, handleRetry, handleDisconnect,
        turnstileToken, handleTurnstileSuccess, handleTurnstileError, handleTurnstileExpire,
    } = useAuthModalLogic({
        isOpen,
        variant: 'admin',
        onSuccess: () => { router.refresh(); },
        onClose: () => {},
    });

    return (
        <div className="relative z-10 flex w-full items-center justify-center p-4 sm:p-6 lg:w-2/5 lg:border-l lg:border-border/20 lg:bg-white/[0.02] lg:backdrop-blur-3xl">
            {isOpen && turnstileToken === null && (
                <div aria-hidden="true" style={{ position: 'absolute', opacity: 0, pointerEvents: 'none', width: 0, height: 0, overflow: 'hidden' }}>
                    <TurnstileWidget
                        onSuccess={handleTurnstileSuccess}
                        onError={handleTurnstileError}
                        onExpire={handleTurnstileExpire}
                        action="auth"
                    />
                </div>
            )}

            <div className="w-full max-w-md">
                <div className="mb-8 text-center lg:hidden">
                    <div className="mb-4 flex items-center justify-center gap-2">
                        <div className="rounded-xl bg-gradient-to-br from-[#7645d9] to-[#1fc7d4] p-2 shadow-xl ring-1 ring-white/20">
                            <Lock className="h-8 w-8 text-white" />
                        </div>
                        <span className="text-2xl font-black italic uppercase tracking-tighter">EPSX</span>
                    </div>
                    <h2 className="text-3xl font-bold tracking-tight">Admin Access</h2>
                    <p className="mt-2 text-slate-400">Verify your admin permissions</p>
                </div>

                <div className="relative overflow-hidden rounded-3xl border border-border/20 bg-card p-8 shadow-[0_24px_48px_-12px_rgba(0,0,0,0.5)] backdrop-blur-3xl lg:p-10">
                    <div className="absolute top-0 left-0 h-[3px] w-full bg-gradient-to-r from-[#7645d9] to-[#1fc7d4]" />

                    <div className="mb-8 hidden text-center lg:block">
                        <div className="mx-auto mb-6 flex h-20 w-20 items-center justify-center rounded-[2rem] bg-purple-500/10 ring-1 ring-purple-500/20 shadow-[0_0_40px_-10px_rgba(118,69,217,0.3)] transition-transform hover:scale-105">
                            <Lock className="h-10 w-10 text-[#7645d9]" />
                        </div>
                        <h2 className="mb-2 text-3xl font-bold tracking-tight">Admin Access</h2>
                        <p className="text-sm text-muted-foreground">Verify your admin permissions</p>
                    </div>

                    {!mounted && (
                        <div className="auth-step">
                            <div className="auth-step-header">
                                <span className="auth-step-number">1</span>
                                <span className="auth-step-label">Select Wallet</span>
                            </div>
                            <div className="auth-wallets" />
                        </div>
                    )}
                    {mounted && (!isOpen || step === 'connect') && (
                        <ConnectStep connectors={connectors} connect={connect} isConnecting={isConnecting} error={error} />
                    )}
                    {isOpen && step === 'switch-chain' && (
                        <SwitchChainStep variant="admin" handleSwitchChain={() => { void handleSwitchChain(); }} isSwitching={isSwitching} />
                    )}
                    {isOpen && step === 'sign' && typeof address === 'string' && address !== '' && (
                        <SignStep address={address} handleSign={() => { void handleSign(); }} handleDisconnect={handleDisconnect} isSigning={isSigning} />
                    )}
                    {isOpen && (step === 'authenticating' || step === 'success' || step === 'error') && (
                        <AuthStatusDisplay step={step} error={error} handleRetry={handleRetry} handleDisconnect={handleDisconnect} />
                    )}
                </div>

                <p className="mt-6 text-center text-xs text-slate-600">
                    Only wallets with admin permissions can access.
                </p>
            </div>
        </div>
    );
}

export function AdminAuthGate() {
    return (
        <div className="fixed inset-0 z-50 flex min-h-screen w-full overflow-hidden bg-background">
            <div className="pointer-events-none absolute inset-0 z-0 overflow-hidden">
                <div className="absolute top-[-10%] left-[-10%] h-[60%] w-[60%] animate-pulse rounded-full bg-purple-600/10 blur-[120px]" />
                <div className="absolute bottom-[-10%] right-[-10%] h-[60%] w-[60%] animate-pulse rounded-full bg-cyan-600/10 blur-[120px]" style={{ animationDelay: '1s' }} />
                <div className="absolute top-[20%] right-[10%] h-[40%] w-[40%] animate-pulse rounded-full bg-indigo-600/10 blur-[100px]" style={{ animationDelay: '2s' }} />
            </div>

            <BrandPanel />
            <AuthPanel />
        </div>
    );
}
