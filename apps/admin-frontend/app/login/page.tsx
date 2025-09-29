'use client';

import { useRouter } from 'next/navigation';
import { AdminWalletAuth } from '@/components/auth/AdminWalletAuth';
import { Shield, Key, Zap, Users } from 'lucide-react';

export default function AdminLoginPage() {
  const router = useRouter();

  const handleAuthSuccess = (walletAddress: string) => {
    console.log('✅ Admin authentication successful:', walletAddress);
    router.push('/');
  };

  const handleAuthError = (error: string) => {
    console.error('❌ Admin authentication error:', error);
  };

  return (
    <div className="min-h-screen bg-gradient-to-br from-yellow-50 via-orange-50 to-pink-50 dark:from-gray-900 dark:via-purple-900 dark:to-gray-900 p-6">
      {/* Background Decorations */}
      <div className="fixed inset-0 overflow-hidden pointer-events-none">
        <div className="absolute top-20 left-20 w-32 h-32 bg-gradient-to-r from-yellow-400/20 to-orange-500/20 rounded-full blur-xl"></div>
        <div className="absolute top-40 right-32 w-24 h-24 bg-gradient-to-r from-pink-400/20 to-purple-500/20 rounded-full blur-lg"></div>
        <div className="absolute bottom-32 left-1/3 w-28 h-28 bg-gradient-to-r from-orange-400/15 to-yellow-500/15 rounded-full blur-xl"></div>
      </div>

      <div className="relative max-w-4xl mx-auto">
        {/* Header */}
        <div className="text-center mb-12">
          <div className="relative inline-block mb-8">
            <h1 className="text-5xl sm:text-6xl font-bold bg-gradient-to-r from-yellow-600 via-orange-600 via-pink-600 to-purple-600 bg-clip-text text-transparent">
              🔐 EPSX Admin Access
            </h1>
            <div className="absolute -top-2 -right-2 w-4 h-4 bg-gradient-to-r from-yellow-400 to-orange-500 rounded-full"></div>
          </div>
          <p className="text-xl text-gray-600 dark:text-gray-400 max-w-2xl mx-auto">
            Connect your Web3 wallet to access the EPSX administrative dashboard
          </p>
        </div>

        {/* Features Grid */}
        <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-4 gap-6 mb-12">
          {[
            {
              icon: Shield,
              title: "Secure Access",
              description: "Web3 wallet authentication with admin permissions",
              color: "from-blue-400 to-cyan-500"
            },
            {
              icon: Key,
              title: "Permission Control",
              description: "Granular access management and user permissions",
              color: "from-green-400 to-emerald-500"
            },
            {
              icon: Zap,
              title: "Real-time Data",
              description: "Live system metrics and performance monitoring",
              color: "from-purple-400 to-pink-500"
            },
            {
              icon: Users,
              title: "User Management",
              description: "Comprehensive wallet and user administration",
              color: "from-orange-400 to-red-500"
            }
          ].map((feature, index) => (
            <div key={index} className="bg-white/80 dark:bg-gray-800/80 backdrop-blur-sm rounded-2xl p-6 shadow-xl border-2 border-gray-200/50 dark:border-gray-700/50">
              <div className={`inline-flex p-3 rounded-full bg-gradient-to-r ${feature.color} mb-4`}>
                <feature.icon className="w-6 h-6 text-white" />
              </div>
              <h3 className="text-lg font-semibold text-gray-900 dark:text-gray-100 mb-2">
                {feature.title}
              </h3>
              <p className="text-sm text-gray-600 dark:text-gray-400">
                {feature.description}
              </p>
            </div>
          ))}
        </div>

        {/* Authentication Section */}
        <div className="bg-white/90 dark:bg-gray-800/90 backdrop-blur-sm rounded-3xl p-8 shadow-2xl border-2 border-gray-200/50 dark:border-gray-700/50">
          <div className="text-center mb-8">
            <h2 className="text-3xl font-bold text-gray-900 dark:text-gray-100 mb-4">
              Connect Admin Wallet
            </h2>
            <p className="text-gray-600 dark:text-gray-400">
              Sign in with your authorized Web3 wallet to access administrative features
            </p>
          </div>

          <div className="max-w-lg mx-auto">
            <AdminWalletAuth 
              onAuthSuccess={handleAuthSuccess}
              onAuthError={handleAuthError}
              className="w-full"
            />
          </div>

          <div className="mt-8 pt-6 border-t border-gray-200 dark:border-gray-700">
            <div className="text-center">
              <p className="text-sm text-gray-500 dark:text-gray-400 mb-4">
                🔒 Secure, decentralized authentication
              </p>
              <div className="flex items-center justify-center space-x-6 text-xs text-gray-400">
                <span>✅ Multi-chain support</span>
                <span>✅ Permission validation</span>
                <span>✅ Session management</span>
              </div>
            </div>
          </div>
        </div>

        {/* Help Section */}
        <div className="mt-12 text-center">
          <p className="text-sm text-gray-500 dark:text-gray-400">
            Need admin access? Contact your system administrator to grant wallet permissions.
          </p>
        </div>
      </div>
    </div>
  );
}