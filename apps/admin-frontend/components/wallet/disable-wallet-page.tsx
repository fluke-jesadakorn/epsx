'use client';

import { disableWalletAction } from '@/app/wallet-management/actions';
import { Breadcrumb } from '@/components/ui/breadcrumb';
import { Button } from '@/components/ui/button';
import { Checkbox } from '@/components/ui/checkbox';
import { Label } from '@/components/ui/label';
import {
    Select,
    SelectContent,
    SelectItem,
    SelectTrigger,
    SelectValue,
} from '@/components/ui/select';
import { Textarea } from '@/components/ui/textarea';
import { cn } from '@/lib/utils';
import {
    AlertTriangle,
    ArrowLeft,
    BarChart3,
    Clock,
    Coins,
    CreditCard,
    FileText,
    Shield,
    ShieldOff,
    TrendingUp,
    Wallet,
} from 'lucide-react';
import { useRouter } from 'next/navigation';
import { useState } from 'react';
import { toast } from 'sonner';
import type { DisableReasonCategory, Platform } from './types';

interface DisableWalletPageProps {
    walletAddress: string;
}

const DURATIONS = [
    { value: '1', label: '24 hours', desc: 'Short-term suspension' },
    { value: '7', label: '7 days', desc: 'Week-long review' },
    { value: '30', label: '30 days', desc: 'Investigation period' },
    { value: '90', label: '90 days', desc: 'Extended suspension' },
    { value: 'until_manual', label: 'Indefinite', desc: 'Until manually re-enabled' },
];

const REASONS: { value: DisableReasonCategory; label: string; icon: typeof AlertTriangle }[] = [
    { value: 'suspicious_activity', label: 'Suspicious Activity', icon: Shield },
    { value: 'tos_violation', label: 'Terms of Service Violation', icon: FileText },
    { value: 'pending_verification', label: 'Pending Verification', icon: Clock },
    { value: 'user_request', label: 'User Request', icon: Wallet },
    { value: 'other', label: 'Other', icon: FileText },
];

const PLATFORMS: { value: Platform; label: string; icon: typeof BarChart3; color: string }[] = [
    { value: 'analytics', label: 'EPSX Analytics', icon: BarChart3, color: 'from-blue-500 to-cyan-500' },
    { value: 'pay', label: 'EPSX Pay', icon: CreditCard, color: 'from-emerald-500 to-teal-500' },
    { value: 'token', label: 'EPSX Token', icon: Coins, color: 'from-amber-500 to-orange-500' },
    { value: 'markets', label: 'EPSX Markets', icon: TrendingUp, color: 'from-purple-500 to-pink-500' },
];

// eslint-disable-next-line max-lines-per-function, complexity
export function DisableWalletPage({ walletAddress }: DisableWalletPageProps) {
    const router = useRouter();
    const [isLoading, setIsLoading] = useState(false);

    // Form state
    const [duration, setDuration] = useState('until_manual');
    const [reason, setReason] = useState<DisableReasonCategory>('suspicious_activity');
    const [details, setDetails] = useState('');
    const [platforms, setPlatforms] = useState<Platform[]>(['analytics', 'pay', 'token', 'markets']);
    const [blockLogin, setBlockLogin] = useState(true);
    const [pauseSubs, setPauseSubs] = useState(false);
    const [notify, setNotify] = useState(false);
    const [error, setError] = useState('');

    const shortAddr = `${walletAddress.slice(0, 6)}...${walletAddress.slice(-4)}`;

    const togglePlatform = (p: Platform) => {
        setPlatforms(prev =>
            prev.includes(p) ? prev.filter(x => x !== p) : [...prev, p]
        );
    };

    const handleSubmit = async () => {
        setError('');
        if (!details.trim()) {
            setError('Please provide details about why this wallet is being disabled.');
            return;
        }
        if (platforms.length === 0) {
            setError('Select at least one platform.');
            return;
        }

        setIsLoading(true);
        try {
            await disableWalletAction(walletAddress, {
                duration_days: duration === 'until_manual' ? null : Number.parseInt(duration, 10),
                reason_category: reason,
                reason_details: details.trim(),
                affected_platforms: platforms,
                block_login: blockLogin,
                pause_subscriptions: pauseSubs,
                notify_user: notify,
            });
            toast.success('Wallet disabled successfully');
            router.push('/wallet-management/wallets');
        } catch (err) {
            setError(err instanceof Error ? err.message : 'Failed to disable wallet');
        } finally {
            setIsLoading(false);
        }
    };

    const selectedDuration = DURATIONS.find(d => d.value === duration);

    return (
        <div className="space-y-6">
            {/* Breadcrumb */}
            <Breadcrumb
                variant="minimal"
                showHome={false}
                items={[
                    { label: 'Wallet Management', href: '/wallet-management/wallets', icon: Wallet },
                    { label: 'Wallets', href: '/wallet-management/wallets' },
                    { label: shortAddr, href: `/wallet-management/${encodeURIComponent(walletAddress)}` },
                    { label: 'Disable Wallet' },
                ]}
            />

            {/* Header */}
            <div className="flex items-center justify-between">
                <div className="flex items-center gap-4">
                    <button
                        onClick={() => router.back()}
                        className="p-2.5 rounded-xl bg-white dark:bg-white/[0.04] border border-gray-200 dark:border-slate-700 hover:bg-black/[0.05] dark:hover:bg-white/10 transition-colors"
                    >
                        <ArrowLeft className="h-5 w-5 text-slate-400" />
                    </button>
                    <div>
                        <h1 className="flex items-center gap-3 text-2xl sm:text-3xl font-bold">
                            <span className="p-2 rounded-xl bg-amber-500/10 border border-amber-500/20">
                                <ShieldOff className="h-6 w-6 text-amber-500" />
                            </span>
                            <span className="bg-gradient-to-r from-amber-400 to-red-400 bg-clip-text text-transparent">
                                Disable Wallet
                            </span>
                        </h1>
                        <p className="text-sm text-muted-foreground mt-1">
                            Restrict access for{' '}
                            <code className="px-1.5 py-0.5 bg-gray-100 dark:bg-slate-800 rounded text-xs font-mono text-slate-300 border border-gray-200 dark:border-slate-700">
                                {walletAddress}
                            </code>
                        </p>
                    </div>
                </div>
            </div>

            {/* Content Grid */}
            <div className="grid grid-cols-1 lg:grid-cols-3 gap-6">
                {/* Left: Form */}
                <div className="lg:col-span-2 space-y-6">
                    {/* Duration */}
                    <section className="rounded-2xl border border-gray-200 dark:border-slate-700 bg-white/[0.02] p-6">
                        <div className="flex items-center gap-2 mb-4">
                            <Clock className="h-5 w-5 text-[#1fc7d4]" />
                            <h2 className="text-lg font-semibold">Disable Duration</h2>
                        </div>
                        <div className="grid grid-cols-2 sm:grid-cols-3 lg:grid-cols-5 gap-3">
                            {DURATIONS.map((opt) => (
                                <button
                                    key={opt.value}
                                    onClick={() => setDuration(opt.value)}
                                    className={cn(
                                        'flex flex-col items-center gap-1 px-3 py-3 rounded-xl border transition-all text-center',
                                        duration === opt.value
                                            ? 'border-amber-500/50 bg-amber-500/10 text-amber-400 shadow-lg shadow-amber-500/5'
                                            : 'border-gray-200 dark:border-slate-700 bg-white/[0.02] text-slate-400 hover:border-gray-200 dark:border-slate-700 hover:bg-white/[0.04]'
                                    )}
                                >
                                    <span className="text-sm font-bold">{opt.label}</span>
                                    <span className="text-[10px] opacity-60">{opt.desc}</span>
                                </button>
                            ))}
                        </div>
                    </section>

                    {/* Platforms */}
                    <section className="rounded-2xl border border-gray-200 dark:border-slate-700 bg-white/[0.02] p-6">
                        <div className="flex items-center justify-between mb-4">
                            <div className="flex items-center gap-2">
                                <BarChart3 className="h-5 w-5 text-[#7645d9]" />
                                <h2 className="text-lg font-semibold">Affected Platforms</h2>
                            </div>
                            <button
                                onClick={() => setPlatforms(
                                    platforms.length === PLATFORMS.length
                                        ? []
                                        : PLATFORMS.map(p => p.value)
                                )}
                                className="text-xs text-[#1fc7d4] hover:underline"
                            >
                                {platforms.length === PLATFORMS.length ? 'Deselect All' : 'Select All'}
                            </button>
                        </div>
                        <div className="grid grid-cols-2 gap-3">
                            {PLATFORMS.map((p) => {
                                const Icon = p.icon;
                                const selected = platforms.includes(p.value);
                                return (
                                    <label
                                        key={p.value}
                                        className={cn(
                                            'flex items-center gap-3 px-4 py-3 rounded-xl border cursor-pointer transition-all',
                                            selected
                                                ? 'border-amber-500/30 bg-amber-500/5'
                                                : 'border-gray-200 dark:border-slate-700 bg-white/[0.02] hover:border-gray-200 dark:border-slate-700'
                                        )}
                                    >
                                        <Checkbox
                                            checked={selected}
                                            onCheckedChange={() => togglePlatform(p.value)}
                                        />
                                        <div className={cn('p-1.5 rounded-lg bg-gradient-to-br', p.color)}>
                                            <Icon className="h-3.5 w-3.5 text-white" />
                                        </div>
                                        <span className="text-sm font-medium">{p.label}</span>
                                    </label>
                                );
                            })}
                        </div>
                    </section>

                    {/* Reason */}
                    <section className="rounded-2xl border border-gray-200 dark:border-slate-700 bg-white/[0.02] p-6 space-y-4">
                        <div className="flex items-center gap-2 mb-2">
                            <FileText className="h-5 w-5 text-[#ed4b9e]" />
                            <h2 className="text-lg font-semibold">Reason & Details</h2>
                        </div>

                        <div>
                            <Label className="text-sm text-muted-foreground">Category</Label>
                            <Select value={reason} onValueChange={(v) => setReason(v as DisableReasonCategory)}>
                                <SelectTrigger className="mt-1.5 bg-white/[0.02] border-gray-200 dark:border-slate-700">
                                    <SelectValue />
                                </SelectTrigger>
                                <SelectContent>
                                    {REASONS.map((r) => {
                                        const RIcon = r.icon;
                                        return (
                                            <SelectItem key={r.value} value={r.value}>
                                                <span className="flex items-center gap-2">
                                                    <RIcon className="h-3.5 w-3.5" />
                                                    {r.label}
                                                </span>
                                            </SelectItem>
                                        );
                                    })}
                                </SelectContent>
                            </Select>
                        </div>

                        <div>
                            <Label className="text-sm text-muted-foreground">Details (Required)</Label>
                            <Textarea
                                value={details}
                                onChange={(e) => setDetails(e.target.value)}
                                placeholder="Explain why this wallet is being disabled. Be specific for audit purposes..."
                                className="mt-1.5 min-h-[120px] bg-white/[0.02] border-gray-200 dark:border-slate-700 resize-none"
                            />
                        </div>
                    </section>

                    {/* Additional Options */}
                    <section className="rounded-2xl border border-gray-200 dark:border-slate-700 bg-white/[0.02] p-6">
                        <div className="flex items-center gap-2 mb-4">
                            <Shield className="h-5 w-5 text-slate-400" />
                            <h2 className="text-lg font-semibold">Additional Actions</h2>
                        </div>
                        <div className="space-y-3">
                            <label className="flex items-center gap-3 px-4 py-3 rounded-xl border border-gray-200 dark:border-slate-700 bg-white/[0.02] cursor-pointer hover:border-gray-200 dark:border-slate-700 transition-colors">
                                <Checkbox checked={blockLogin} onCheckedChange={(c) => setBlockLogin(c === true)} />
                                <div>
                                    <span className="text-sm font-medium">Block login across all platforms</span>
                                    <p className="text-xs text-muted-foreground mt-0.5">Prevents the wallet from authenticating on any EPSX platform</p>
                                </div>
                            </label>
                            <label className="flex items-center gap-3 px-4 py-3 rounded-xl border border-gray-200 dark:border-slate-700 bg-white/[0.02] cursor-pointer hover:border-gray-200 dark:border-slate-700 transition-colors">
                                <Checkbox checked={pauseSubs} onCheckedChange={(c) => setPauseSubs(c === true)} />
                                <div>
                                    <span className="text-sm font-medium">Pause active subscriptions</span>
                                    <p className="text-xs text-muted-foreground mt-0.5">Billing will be paused until wallet is re-enabled</p>
                                </div>
                            </label>
                            <label className="flex items-center gap-3 px-4 py-3 rounded-xl border border-gray-200 dark:border-slate-700 bg-white/[0.02] cursor-pointer hover:border-gray-200 dark:border-slate-700 transition-colors">
                                <Checkbox checked={notify} onCheckedChange={(c) => setNotify(c === true)} />
                                <div>
                                    <span className="text-sm font-medium">Send notification to user</span>
                                    <p className="text-xs text-muted-foreground mt-0.5">Email notification if the user has a registered email address</p>
                                </div>
                            </label>
                        </div>
                    </section>
                </div>

                {/* Right: Summary */}
                <div className="space-y-6">
                    {/* Summary Card */}
                    <div className="rounded-2xl border border-amber-500/20 bg-amber-500/5 p-6 sticky top-6">
                        <h3 className="text-sm font-bold uppercase tracking-wider text-amber-400 mb-4">Action Summary</h3>

                        <div className="space-y-4">
                            <div>
                                <span className="text-xs text-muted-foreground">Wallet</span>
                                <p className="font-mono text-sm text-slate-200 mt-0.5">{shortAddr}</p>
                            </div>

                            <div>
                                <span className="text-xs text-muted-foreground">Duration</span>
                                <p className="text-sm font-medium text-slate-200 mt-0.5">
                                    {selectedDuration?.label ?? 'Not selected'}
                                </p>
                            </div>

                            <div>
                                <span className="text-xs text-muted-foreground">Reason</span>
                                <p className="text-sm font-medium text-slate-200 mt-0.5">
                                    {REASONS.find(r => r.value === reason)?.label ?? '-'}
                                </p>
                            </div>

                            <div>
                                <span className="text-xs text-muted-foreground">Platforms ({platforms.length})</span>
                                <div className="flex flex-wrap gap-1.5 mt-1">
                                    {platforms.map(p => {
                                        const conf = PLATFORMS.find(x => x.value === p);
                                        if (!conf) { return null; }
                                        const PIcon = conf.icon;
                                        return (
                                            <span key={p} className="inline-flex items-center gap-1 px-2 py-0.5 rounded-md bg-white dark:bg-white/[0.04] text-xs text-slate-300 border border-gray-200 dark:border-slate-700">
                                                <PIcon className="h-3 w-3" /> {conf.label.replace('EPSX ', '')}
                                            </span>
                                        );
                                    })}
                                </div>
                            </div>

                            <div className="border-t border-gray-200 dark:border-slate-700 pt-3 space-y-1.5">
                                <div className="flex items-center gap-2 text-xs">
                                    <div className={cn('w-2 h-2 rounded-full', blockLogin ? 'bg-red-400' : 'bg-slate-600')} />
                                    <span className={blockLogin ? 'text-slate-200' : 'text-slate-500'}>Block login</span>
                                </div>
                                <div className="flex items-center gap-2 text-xs">
                                    <div className={cn('w-2 h-2 rounded-full', pauseSubs ? 'bg-amber-400' : 'bg-slate-600')} />
                                    <span className={pauseSubs ? 'text-slate-200' : 'text-slate-500'}>Pause subscriptions</span>
                                </div>
                                <div className="flex items-center gap-2 text-xs">
                                    <div className={cn('w-2 h-2 rounded-full', notify ? 'bg-blue-400' : 'bg-slate-600')} />
                                    <span className={notify ? 'text-slate-200' : 'text-slate-500'}>Notify user</span>
                                </div>
                            </div>
                        </div>

                        {/* Warning */}
                        <div className="mt-5 p-3 rounded-xl bg-amber-500/10 border border-amber-500/20">
                            <div className="flex gap-2">
                                <AlertTriangle className="h-4 w-4 text-amber-400 shrink-0 mt-0.5" />
                                <p className="text-xs text-amber-300/90 leading-relaxed">
                                    This wallet will lose access to the selected platforms immediately. An admin must manually re-enable access if the duration is set to indefinite.
                                </p>
                            </div>
                        </div>

                        {/* Error */}
                        {error !== '' && (
                            <div className="mt-3 p-3 rounded-xl bg-red-500/10 border border-red-500/20">
                                <p className="text-xs text-red-400">{error}</p>
                            </div>
                        )}

                        {/* Actions */}
                        <div className="mt-5 space-y-2">
                            <Button
                                onClick={() => { void handleSubmit(); }}
                                disabled={isLoading}
                                className="w-full bg-gradient-to-r from-amber-600 to-red-600 hover:from-amber-500 hover:to-red-500 text-white font-bold shadow-lg shadow-amber-500/20"
                            >
                                {isLoading ? (
                                    <span className="flex items-center gap-2">
                                        <span className="h-4 w-4 border-2 border-white/30 border-t-white rounded-full animate-spin" />
                                        Disabling...
                                    </span>
                                ) : (
                                    <span className="flex items-center gap-2">
                                        <ShieldOff className="h-4 w-4" />
                                        Disable Wallet
                                    </span>
                                )}
                            </Button>
                            <Button
                                variant="ghost"
                                onClick={() => router.back()}
                                disabled={isLoading}
                                className="w-full text-slate-400 hover:text-white"
                            >
                                Cancel
                            </Button>
                        </div>
                    </div>
                </div>
            </div>
        </div>
    );
}
