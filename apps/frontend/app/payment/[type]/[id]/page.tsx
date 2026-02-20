import { CreditCard, Gem, Key, Link2, Lock, Users, Zap } from 'lucide-react';
import { PaymentAuthGuard } from '@/app/payment/components/payment-auth-guard';
import { getCurrentUser, getDebugSessionInfo } from '@/lib/server-actions-user';
import { PaymentClient } from '../../payment-client';
import { getPublicPlansAction } from '@/app/actions/plans';

export const dynamic = 'force-dynamic';

import type { LucideIcon } from 'lucide-react';

type PaymentType = 'plan' | 'access-plan' | 'permission' | 'link';

interface ThemeConfig {
    gradient: string;
    decorGradient1: string;
    decorGradient2: string;
    decorGradient3: string;
    headingGradient: string;
    iconBg: string;
    Icon: LucideIcon;
    title: string;
    description: string;
}

interface PaymentDynamicPageProps {
    params: Promise<{
        type: string;
        id: string;
    }>;
}

/**
 * Get theme configuration based on payment type
 */
function getThemeConfig(type: PaymentType): ThemeConfig {
    switch (type) {
        case 'plan':
            return {
                gradient: 'from-purple-50 via-indigo-50 to-blue-50 dark:from-gray-900 dark:via-purple-900/20 dark:to-gray-800',
                decorGradient1: 'from-purple-400/30 to-indigo-500/30',
                decorGradient2: 'from-blue-400/30 to-cyan-500/30',
                decorGradient3: 'from-pink-400/30 to-purple-500/30',
                headingGradient: 'from-purple-600 via-indigo-600 to-blue-600',
                iconBg: 'from-purple-500 to-indigo-600',
                Icon: Gem,
                title: 'Upgrade Your Plan',
                description: 'Unlock powerful analytics, API access, and premium features',
            };
        case 'access-plan':
            return {
                gradient: 'from-emerald-50 via-teal-50 to-cyan-50 dark:from-gray-900 dark:via-emerald-900/20 dark:to-gray-800',
                decorGradient1: 'from-emerald-400/30 to-teal-500/30',
                decorGradient2: 'from-cyan-400/30 to-blue-500/30',
                decorGradient3: 'from-teal-400/30 to-emerald-500/30',
                headingGradient: 'from-emerald-600 via-teal-600 to-cyan-600',
                iconBg: 'from-emerald-500 to-teal-600',
                Icon: Users,
                title: 'Join Access Plan',
                description: 'Get access to shared permissions and plan-exclusive features',
            };
        case 'permission':
            return {
                gradient: 'from-amber-50 via-orange-50 to-rose-50 dark:from-gray-900 dark:via-amber-900/20 dark:to-gray-800',
                decorGradient1: 'from-amber-400/30 to-orange-500/30',
                decorGradient2: 'from-rose-400/30 to-pink-500/30',
                decorGradient3: 'from-orange-400/30 to-amber-500/30',
                headingGradient: 'from-amber-600 via-orange-600 to-rose-600',
                iconBg: 'from-amber-500 to-orange-600',
                Icon: Key,
                title: 'Unlock permission',
                description: 'Purchase specific access rights for advanced features',
            };
        case 'link':
        default:
            return {
                gradient: 'from-pink-50 via-purple-50 to-indigo-50 dark:from-gray-900 dark:via-pink-900/20 dark:to-gray-800',
                decorGradient1: 'from-pink-400/30 to-purple-500/30',
                decorGradient2: 'from-indigo-400/30 to-blue-500/30',
                decorGradient3: 'from-purple-400/30 to-pink-500/30',
                headingGradient: 'from-pink-600 via-purple-600 to-indigo-600',
                iconBg: 'from-pink-500 to-purple-600',
                Icon: Link2,
                title: 'Complete Payment',
                description: 'Secure blockchain-powered payment',
            };
    }
}

export default async function PaymentDynamicPage({ params }: PaymentDynamicPageProps) {
    const user = await getCurrentUser();
    const debugInfo = !user ? await getDebugSessionInfo() : null;
    const { type, id } = await params;

    // Validate payment type
    // Handle 'group' for legacy support by mapping to 'access-plan' if needed,
    // but the route itself is what matters. Since this is [type], 'group' will come in as type.

    const validTypes: PaymentType[] = ['plan', 'access-plan', 'permission', 'link'];
    let paymentType = validTypes.includes(type as PaymentType)
        ? (type as PaymentType)
        : 'plan';

    // Legacy support for accessing via /payment/group/[id] directly if it wasn't redirected
    if (type === 'group') {
        paymentType = 'access-plan';
    }

    // Get theme configuration
    const theme = getThemeConfig(paymentType);

    // Fetch plans on server to avoid client-side loading
    const plansResult = await getPublicPlansAction();
    const initialPlans = plansResult.success && plansResult.data ? plansResult.data : [];

    // Show auth guard for unauthenticated users
    if (!user) {
        return (
            <main className={`min-h-screen bg-[#0F1115] flex items-center justify-center relative overflow-hidden`}>
                <div className="container mx-auto p-6 relative z-10">
                    <PaymentAuthGuard />
                </div>
            </main>
        );
    }

    return (
        <main className={`min-h-screen bg-gradient-to-br ${theme.gradient} py-12 px-4 relative overflow-hidden`}>
            {/* Decorative background */}
            <div className="absolute inset-0 pointer-events-none">
                <div className={`absolute top-10 left-10 w-32 h-32 bg-gradient-to-br ${theme.decorGradient1} rounded-full blur-xl`} />
                <div className={`absolute top-40 right-20 w-24 h-24 bg-gradient-to-br ${theme.decorGradient2} rounded-full blur-xl`} />
                <div className={`absolute bottom-20 left-20 w-40 h-40 bg-gradient-to-br ${theme.decorGradient3} rounded-full blur-xl`} />
            </div>

            <div className="max-w-6xl mx-auto relative z-10">
                {/* Hero Header */}
                <div className="text-center mb-12">
                    <div className={`inline-flex items-center justify-center w-20 h-20 rounded-full bg-gradient-to-br ${theme.iconBg} mb-6`}>
                        <theme.Icon className="w-10 h-10 text-white" />
                    </div>
                    <h1 className={`text-4xl lg:text-5xl font-black bg-gradient-to-r ${theme.headingGradient} bg-clip-text text-transparent mb-4`}>
                        {theme.title}
                    </h1>
                    <p className="text-lg text-gray-700 dark:text-gray-300 max-w-2xl mx-auto">
                        {theme.description}
                    </p>
                </div>

                {/* Payment Flow */}
                <PaymentClient
                    paymentType={paymentType === 'link' ? 'plan' : paymentType}
                    preselectedId={id}
                    title={theme.title}
                    description={theme.description}
                    initialPlans={initialPlans}
                />

                {/* Security Footer */}
                <div className="mt-16 text-center">
                    <div className="inline-flex items-center gap-4 px-6 py-3 bg-white/60 dark:bg-gray-800/60 backdrop-blur-sm rounded-xl border border-gray-200/50 dark:border-gray-700/50 shadow-lg">
                        <div className="flex items-center gap-2 text-sm text-gray-600 dark:text-gray-400">
                            <Lock className="w-4 h-4 text-green-500" />
                            <span>Blockchain Secured</span>
                        </div>
                        <div className="w-px h-4 bg-gray-300 dark:bg-gray-600" />
                        <div className="flex items-center gap-2 text-sm text-gray-600 dark:text-gray-400">
                            <Zap className="w-4 h-4 text-blue-500" />
                            <span>Instant Activation</span>
                        </div>
                        <div className="w-px h-4 bg-gray-300 dark:bg-gray-600" />
                        <div className="flex items-center gap-2 text-sm text-gray-600 dark:text-gray-400">
                            <CreditCard className="w-4 h-4 text-purple-500" />
                            <span>USDT/USDC/DAI</span>
                        </div>
                    </div>
                </div>
            </div>
        </main>
    );
}
