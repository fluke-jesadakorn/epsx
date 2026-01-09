import { GlobalAuthGuard } from '@/components/auth/GlobalAuthGuard';
import { getCurrentUser } from '@/lib/server-actions';
import { getDebugSessionInfo } from '@/lib/server-actions-user';
import { PaymentClient } from '../../PaymentClient';

export const dynamic = 'force-dynamic';

interface PaymentPermissionPageProps {
    params: Promise<{
        id: string;
    }>;
}

export default async function PaymentPermissionPage({ params }: PaymentPermissionPageProps) {
    const user = await getCurrentUser();
    const debugInfo = !user ? await getDebugSessionInfo() : null;
    const { id } = await params;

    // Show auth guard for unauthenticated users
    if (!user) {
        return (
            <main className="min-h-screen bg-gradient-to-br from-amber-50 via-orange-50 to-rose-50 dark:from-gray-900 dark:via-amber-900/20 dark:to-gray-800 flex items-center justify-center relative overflow-hidden">
                {/* Decorative background */}
                <div className="absolute inset-0 pointer-events-none">
                    <div className="absolute top-10 left-10 w-32 h-32 bg-gradient-to-br from-amber-400/30 to-orange-500/30 rounded-full blur-xl" />
                    <div className="absolute top-40 right-20 w-24 h-24 bg-gradient-to-br from-rose-400/30 to-pink-500/30 rounded-full blur-xl" />
                    <div className="absolute bottom-20 left-20 w-40 h-40 bg-gradient-to-br from-orange-400/30 to-amber-500/30 rounded-full blur-xl" />
                </div>
                <div className="container mx-auto p-6 relative z-10">
                    <GlobalAuthGuard title="Permission Access" debugInfo={debugInfo} />
                </div>
            </main>
        );
    }

    return (
        <main className="min-h-screen bg-gradient-to-br from-amber-50 via-orange-50 to-rose-50 dark:from-gray-900 dark:via-amber-900/20 dark:to-gray-800 py-12 px-4 relative overflow-hidden">
            {/* Decorative background */}
            <div className="absolute inset-0 pointer-events-none">
                <div className="absolute top-10 left-10 w-32 h-32 bg-gradient-to-br from-amber-400/30 to-orange-500/30 rounded-full blur-xl" />
                <div className="absolute top-40 right-20 w-24 h-24 bg-gradient-to-br from-rose-400/30 to-pink-500/30 rounded-full blur-xl" />
                <div className="absolute bottom-20 left-20 w-40 h-40 bg-gradient-to-br from-orange-400/30 to-amber-500/30 rounded-full blur-xl" />
            </div>

            <div className="max-w-6xl mx-auto relative z-10">
                {/* Hero Header */}
                <div className="text-center mb-12">
                    <div className="inline-block mb-6">
                        <span className="text-6xl">🔑</span>
                    </div>
                    <h1 className="text-4xl lg:text-5xl font-black bg-gradient-to-r from-amber-600 via-orange-600 to-rose-600 bg-clip-text text-transparent mb-4">
                        Unlock Permission
                    </h1>
                    <p className="text-lg text-gray-700 dark:text-gray-300 max-w-2xl mx-auto">
                        Purchase specific access rights for advanced features
                    </p>
                </div>

                {/* Payment Flow */}
                <PaymentClient
                    paymentType="permission"
                    preselectedId={id}
                    title="Unlock Access"
                    description="Purchase access to this specific feature or capability"
                />
            </div>
        </main>
    );
}
