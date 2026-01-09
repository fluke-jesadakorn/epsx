import { GlobalAuthGuard } from '@/components/auth/GlobalAuthGuard';
import { getCurrentUser } from '@/lib/server-actions';
import { getDebugSessionInfo } from '@/lib/server-actions-user';
import { PaymentClient } from '../../PaymentClient';

export const dynamic = 'force-dynamic';

interface PaymentPlanPageProps {
    params: Promise<{
        id: string;
    }>;
}

export default async function PaymentPlanPage({ params }: PaymentPlanPageProps) {
    const user = await getCurrentUser();
    const debugInfo = !user ? await getDebugSessionInfo() : null;
    const { id } = await params;

    // Show auth guard for unauthenticated users
    if (!user) {
        return (
            <main className="min-h-screen bg-gradient-to-br from-purple-50 via-indigo-50 to-blue-50 dark:from-gray-900 dark:via-purple-900/20 dark:to-gray-800 flex items-center justify-center relative overflow-hidden">
                {/* Decorative background */}
                <div className="absolute inset-0 pointer-events-none">
                    <div className="absolute top-10 left-10 w-32 h-32 bg-gradient-to-br from-purple-400/30 to-indigo-500/30 rounded-full blur-xl" />
                    <div className="absolute top-40 right-20 w-24 h-24 bg-gradient-to-br from-blue-400/30 to-cyan-500/30 rounded-full blur-xl" />
                    <div className="absolute bottom-20 left-20 w-40 h-40 bg-gradient-to-br from-pink-400/30 to-purple-500/30 rounded-full blur-xl" />
                </div>
                <div className="container mx-auto p-6 relative z-10">
                    <GlobalAuthGuard title="Payment Portal" debugInfo={debugInfo} />
                </div>
            </main>
        );
    }

    return (
        <main className="min-h-screen bg-gradient-to-br from-purple-50 via-indigo-50 to-blue-50 dark:from-gray-900 dark:via-purple-900/20 dark:to-gray-800 py-12 px-4 relative overflow-hidden">
            {/* Decorative background */}
            <div className="absolute inset-0 pointer-events-none">
                <div className="absolute top-10 left-10 w-32 h-32 bg-gradient-to-br from-purple-400/30 to-indigo-500/30 rounded-full blur-xl" />
                <div className="absolute top-40 right-20 w-24 h-24 bg-gradient-to-br from-blue-400/30 to-cyan-500/30 rounded-full blur-xl" />
                <div className="absolute bottom-20 left-20 w-40 h-40 bg-gradient-to-br from-pink-400/30 to-purple-500/30 rounded-full blur-xl" />
            </div>

            <div className="max-w-6xl mx-auto relative z-10">
                {/* Hero Header */}
                <div className="text-center mb-12">
                    <div className="inline-block mb-6">
                        <span className="text-6xl">💎</span>
                    </div>
                    <h1 className="text-4xl lg:text-5xl font-black bg-gradient-to-r from-purple-600 via-indigo-600 to-blue-600 bg-clip-text text-transparent mb-4">
                        Upgrade Your Plan
                    </h1>
                    <p className="text-lg text-gray-700 dark:text-gray-300 max-w-2xl mx-auto">
                        Unlock powerful analytics, API access, and premium features with blockchain-secured payments
                    </p>
                </div>

                {/* Payment Flow */}
                <PaymentClient
                    paymentType="plan"
                    preselectedId={id}
                    title="Choose Your Plan"
                    description="Select a plan to unlock premium features and analytics"
                />
            </div>
        </main>
    );
}
