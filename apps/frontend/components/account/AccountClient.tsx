'use client';

import {
  Badge,
  Button
} from '@/components/ui';
import { Alert, AlertDescription } from '@/components/ui/alert';
import { useApiClient } from '@/shared/hooks/useApiClient';
import {
  AlertCircle,
  ArrowRight,
  Bell,
  CheckCircle,
  CreditCard,
  Lock,
  Shield,
  Sparkles
} from 'lucide-react';
import Link from 'next/link';
import { useRouter } from 'next/navigation';
import { useEffect, useState } from 'react';
import { useAccount } from 'wagmi';
import { AccessOverview } from './AccessOverview';
import { PaymentHistoryTab } from './PaymentHistoryTab';

interface NotificationPreferences {
  trading: boolean;
  security: boolean;
  account: boolean;
  system: boolean;
  marketing: boolean;
}

interface UserProfile {
  wallet_address: string;
  created_at: string;
  last_login: string;
  auth_method: string;
}

interface NotificationPreferencesResponse {
  preferences: NotificationPreferences;
}

export function AccountClient() {
  const router = useRouter();
  const { base } = useApiClient({ platform: 'frontend' });
  const { address } = useAccount();
  const [notificationPrefs, setNotificationPrefs] = useState<NotificationPreferences>({
    trading: true,
    security: true,
    account: true,
    system: false,
    marketing: false
  });
  const [userProfile, setUserProfile] = useState<UserProfile | null>(null);

  const [prefsLoading, setPrefsLoading] = useState(true);
  const [prefsError, setPrefsError] = useState<string | null>(null);
  const [prefsSuccess, setPrefsSuccess] = useState<string | null>(null);

  // Load notification preferences and user info on component mount
  useEffect(() => {
    loadNotificationPreferences();
    loadUserInfo();
  }, [base]);

  const loadUserInfo = async () => {
    try {
      const response = await base.get<{ success: boolean, data: UserProfile }>('/api/users/profile');
      if (response && response.success && response.data?.success && response.data.data) {
        setUserProfile(response.data.data);
      }
    } catch (error) {
      console.error('Error loading user info:', error);
    }
  };

  const loadNotificationPreferences = async () => {
    try {
      setPrefsLoading(true);
      const response = await base.get<{ success: boolean, data: NotificationPreferencesResponse }>('/api/notifications/preferences');
      if (response && response.success && response.data?.success && response.data.data?.preferences) {
        setNotificationPrefs(response.data.data.preferences);
      }
    } catch (error) {
      console.error('Error loading notification preferences:', error);
    } finally {
      setPrefsLoading(false);
    }
  };

  const updateNotificationPrefs = async (prefs: NotificationPreferences) => {
    try {
      setPrefsError(null);
      setPrefsSuccess(null);
      const response = await base.post('/api/notifications/preferences', prefs);
      if (response && response.success) {
        setNotificationPrefs(prefs);
        setPrefsSuccess('Notification preferences updated successfully!');
        setTimeout(() => setPrefsSuccess(null), 3000);
      } else {
        throw new Error('Failed to update preferences');
      }
    } catch (error) {
      console.error('Error updating notification preferences:', error);
      setPrefsError('Failed to update preferences. Please try again.');
    }
  };

  return (
    <div className="min-h-screen bg-gradient-to-br from-yellow-50 via-orange-50 to-pink-50 dark:from-gray-900 dark:via-purple-900 dark:to-gray-900 p-3 sm:p-6 pb-20">
      {/* Background Decorations */}
      <div className="fixed inset-0 overflow-hidden pointer-events-none">
        <div className="absolute top-20 left-20 w-32 h-32 bg-gradient-to-r from-yellow-400/20 to-orange-500/20 rounded-full blur-xl"></div>
        <div className="absolute top-40 right-32 w-24 h-24 bg-gradient-to-r from-pink-400/20 to-purple-500/20 rounded-full blur-lg"></div>
        <div className="absolute bottom-32 left-1/3 w-28 h-28 bg-gradient-to-r from-orange-400/15 to-yellow-500/15 rounded-full blur-xl"></div>
      </div>

      <div className="relative max-w-7xl mx-auto px-1 sm:px-4 lg:px-6 space-y-8 sm:space-y-12">
        {/* Page Header */}
        <div className="text-center mb-12">
          <div className="relative inline-block">
            <h1 className="text-4xl sm:text-5xl font-bold bg-gradient-to-r from-yellow-600 via-orange-600 via-pink-600 to-purple-600 bg-clip-text text-transparent mb-4 flex items-center justify-center gap-3">
              <span>👤</span> Account Settings
            </h1>
            <div className="absolute -top-2 -right-2 w-4 h-4 bg-gradient-to-r from-yellow-400 to-orange-500 rounded-full"></div>
          </div>
          <p className="text-base sm:text-lg text-gray-700 dark:text-gray-300 max-w-2xl mx-auto font-medium">
            Manage your account access, payments, and preferences with ease
          </p>
        </div>

        {/* Analytics/Stats Cards Section */}
        <div className="grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-4 gap-4 sm:gap-6">
          {/* Wallet Address Card */}
          <div className="bg-white/80 dark:bg-gray-800/80 backdrop-blur-sm rounded-2xl sm:rounded-3xl p-5 sm:p-6 shadow-xl border-2 border-blue-300/50 dark:border-blue-700/50 hover:shadow-2xl transition-all duration-300 group">
            <div className="flex items-center justify-between mb-4 text-2xl sm:text-3xl">
              <span className="group-hover:scale-110 transition-transform">👛</span>
              <Badge variant="outline" className="text-xs font-semibold bg-blue-50/50 dark:bg-blue-900/20 text-blue-600 border-blue-200">Wallet</Badge>
            </div>
            <div className="space-y-1">
              <div className="text-sm font-medium text-gray-500 dark:text-gray-400">Current Address</div>
              <div className="text-sm font-mono font-bold text-gray-900 dark:text-gray-100 truncate">
                {userProfile?.wallet_address || address || 'Not Connected'}
              </div>
            </div>
          </div>

          {/* Member Status Card */}
          <div className="bg-white/80 dark:bg-gray-800/80 backdrop-blur-sm rounded-2xl sm:rounded-3xl p-5 sm:p-6 shadow-xl border-2 border-green-300/50 dark:border-green-700/50 hover:shadow-2xl transition-all duration-300 group">
            <div className="flex items-center justify-between mb-4 text-2xl sm:text-3xl">
              <span className="group-hover:scale-110 transition-transform">✅</span>
              <Badge variant="outline" className="text-xs font-semibold bg-green-50/50 dark:bg-green-900/20 text-green-600 border-green-200">Active</Badge>
            </div>
            <div className="space-y-1">
              <div className="text-sm font-medium text-gray-500 dark:text-gray-400">Member Since</div>
              <div className="text-lg font-bold text-gray-900 dark:text-gray-100">
                {userProfile?.created_at ? new Date(userProfile.created_at).toLocaleDateString(undefined, { month: 'long', year: 'numeric' }) : 'Join Now'}
              </div>
            </div>
          </div>

          {/* Renewal Status Card */}
          <div className="bg-white/80 dark:bg-gray-800/80 backdrop-blur-sm rounded-2xl sm:rounded-3xl p-5 sm:p-6 shadow-xl border-2 border-orange-300/50 dark:border-orange-700/50 hover:shadow-2xl transition-all duration-300 group">
            <div className="flex items-center justify-between mb-4 text-2xl sm:text-3xl">
              <span className="group-hover:scale-110 transition-transform">⏰</span>
              <Badge variant="outline" className="text-xs font-semibold bg-orange-50/50 dark:bg-orange-900/20 text-orange-600 border-orange-200">Status</Badge>
            </div>
            <div className="space-y-1">
              <div className="text-sm font-medium text-gray-500 dark:text-gray-400">Recent Payment</div>
              <div className="text-lg font-bold text-gray-900 dark:text-gray-100">N/A</div>
            </div>
          </div>

          {/* Security Card */}
          <div className="bg-white/80 dark:bg-gray-800/80 backdrop-blur-sm rounded-2xl sm:rounded-3xl p-5 sm:p-6 shadow-xl border-2 border-purple-300/50 dark:border-purple-700/50 hover:shadow-2xl transition-all duration-300 group">
            <div className="flex items-center justify-between mb-4 text-2xl sm:text-3xl">
              <span className="group-hover:scale-110 transition-transform">🛡️</span>
              <Badge variant="outline" className="text-xs font-semibold bg-purple-50/50 dark:bg-purple-900/20 text-purple-600 border-purple-200">Secure</Badge>
            </div>
            <div className="space-y-1">
              <div className="text-sm font-medium text-gray-500 dark:text-gray-400">Method</div>
              <div className="text-lg font-bold text-gray-900 dark:text-gray-100 capitalize">Web3 Vault</div>
            </div>
          </div>
        </div>

        {/* Quick Actions Grid */}
        <div className="grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-3 gap-4 sm:gap-6">
          <Link href="/support" className="block group">
            <div className="relative overflow-hidden rounded-2xl sm:rounded-3xl bg-gradient-to-r from-blue-400/20 via-cyan-400/20 to-blue-400/20 p-0.5 hover:scale-105 transition-all duration-300">
              <div className="relative bg-white/95 dark:bg-gray-900/95 backdrop-blur-xl rounded-2xl sm:rounded-3xl p-5 sm:p-6">
                <div className="absolute top-4 right-4 w-4 h-4 bg-gradient-to-r from-blue-400 to-cyan-500 rounded-full blur-sm opacity-60"></div>
                <h3 className="text-lg sm:text-xl font-bold bg-gradient-to-r from-blue-400 to-cyan-500 bg-clip-text text-transparent mb-2 flex items-center gap-2">
                  <span className="text-xl">🛟</span> Support Center
                </h3>
                <p className="text-sm text-gray-700 dark:text-gray-300 mb-4">Need help? Connect with our team</p>
                <div className="flex items-center justify-between">
                  <div className="px-3 py-1 bg-gradient-to-r from-blue-400 to-cyan-500 text-white rounded-full text-xs font-semibold">Contact</div>
                  <ArrowRight className="w-4 h-4 text-gray-400 group-hover:translate-x-1 transition-transform" />
                </div>
              </div>
            </div>
          </Link>

          <Link href="/privacy" className="block group">
            <div className="relative overflow-hidden rounded-2xl sm:rounded-3xl bg-gradient-to-r from-green-400/20 via-emerald-400/20 to-green-400/20 p-0.5 hover:scale-105 transition-all duration-300">
              <div className="relative bg-white/95 dark:bg-gray-900/95 backdrop-blur-xl rounded-2xl sm:rounded-3xl p-5 sm:p-6">
                <div className="absolute top-4 right-4 w-4 h-4 bg-gradient-to-r from-green-400 to-emerald-500 rounded-full blur-sm opacity-60"></div>
                <h3 className="text-lg sm:text-xl font-bold bg-gradient-to-r from-green-400 to-emerald-500 bg-clip-text text-transparent mb-2 flex items-center gap-2">
                  <span className="text-xl">🔒</span> Privacy Control
                </h3>
                <p className="text-sm text-gray-700 dark:text-gray-300 mb-4">Manage your data and visibility settings</p>
                <div className="flex items-center justify-between">
                  <div className="px-3 py-1 bg-gradient-to-r from-green-400 to-emerald-500 text-white rounded-full text-xs font-semibold">Settings</div>
                  <ArrowRight className="w-4 h-4 text-gray-400 group-hover:translate-x-1 transition-transform" />
                </div>
              </div>
            </div>
          </Link>

          <Link href="/notifications" className="block group">
            <div className="relative overflow-hidden rounded-2xl sm:rounded-3xl bg-gradient-to-r from-orange-400/20 via-pink-400/20 to-orange-400/20 p-0.5 hover:scale-105 transition-all duration-300">
              <div className="relative bg-white/95 dark:bg-gray-900/95 backdrop-blur-xl rounded-2xl sm:rounded-3xl p-5 sm:p-6">
                <div className="absolute top-4 right-4 w-4 h-4 bg-gradient-to-r from-orange-400 to-pink-500 rounded-full blur-sm opacity-60"></div>
                <h3 className="text-lg sm:text-xl font-bold bg-gradient-to-r from-orange-400 to-pink-500 bg-clip-text text-transparent mb-2 flex items-center gap-2">
                  <span className="text-xl">🔔</span> Recent Activity
                </h3>
                <p className="text-sm text-gray-700 dark:text-gray-300 mb-4">Check your latest logs and alerts</p>
                <div className="flex items-center justify-between">
                  <div className="px-3 py-1 bg-gradient-to-r from-orange-400 to-pink-500 text-white rounded-full text-xs font-semibold">View Logs</div>
                  <ArrowRight className="w-4 h-4 text-gray-400 group-hover:translate-x-1 transition-transform" />
                </div>
              </div>
            </div>
          </Link>
        </div>

        {/* Access & Plans Section */}
        <div className="bg-white/70 dark:bg-gray-800/60 backdrop-blur-xl rounded-[2rem] sm:rounded-[3rem] p-6 sm:p-8 lg:p-10 shadow-2xl border-2 border-indigo-200/50 dark:border-indigo-800/50">
          <div className="flex items-center gap-3 mb-8">
            <div className="p-3 bg-indigo-100 dark:bg-indigo-900/30 rounded-2xl">
              <Shield className="w-6 h-6 text-indigo-600 dark:text-indigo-400" />
            </div>
            <h2 className="text-2xl sm:text-3xl font-bold text-gray-900 dark:text-white">Access & Plans</h2>
          </div>
          <AccessOverview />
        </div>

        {/* Payments Section */}
        <div className="bg-white/70 dark:bg-gray-800/60 backdrop-blur-xl rounded-[2rem] sm:rounded-[3rem] p-6 sm:p-8 lg:p-10 shadow-2xl border-2 border-blue-200/50 dark:border-blue-800/50">
          <div className="flex items-center gap-3 mb-8">
            <div className="p-3 bg-blue-100 dark:bg-blue-900/30 rounded-2xl">
              <CreditCard className="w-6 h-6 text-blue-600 dark:text-blue-400" />
            </div>
            <h2 className="text-2xl sm:text-3xl font-bold text-gray-900 dark:text-white">Transaction History</h2>
          </div>
          <PaymentHistoryTab />
        </div>

        {/* Notification Settings Section */}
        <div className="bg-white/70 dark:bg-gray-800/60 backdrop-blur-xl rounded-[2rem] sm:rounded-[3rem] p-6 sm:p-8 lg:p-10 shadow-2xl border-2 border-purple-200/50 dark:border-purple-800/50">
          <div className="flex items-center gap-3 mb-8">
            <div className="p-3 bg-purple-100 dark:bg-purple-900/30 rounded-2xl">
              <Bell className="w-6 h-6 text-purple-600 dark:text-purple-400" />
            </div>
            <h2 className="text-2xl sm:text-3xl font-bold text-gray-900 dark:text-white">Notification Preferences</h2>
          </div>

          <div className="grid lg:grid-cols-12 gap-8">
            <div className="lg:col-span-4 space-y-4">
              <p className="text-gray-600 dark:text-gray-400 text-base leading-relaxed">
                Choose exactly what you want to be notified about. We'll send alerts via web push to keep you updated.
              </p>
              <div className="flex flex-col gap-3 pt-2">
                <Button variant="secondary" className="w-full justify-between group hover:border-purple-300 dark:hover:border-purple-700 font-bold" asChild>
                  <Link href="/notifications">
                    <span>Browse All Alerts</span>
                    <ArrowRight className="w-4 h-4 transition-transform group-hover:translate-x-1 flex-shrink-0" />
                  </Link>
                </Button>
                <Button variant="secondary" className="w-full justify-between group font-bold" asChild>
                  <Link href="/notifications/preferences">
                    <span>Advanced Settings</span>
                    <Sparkles className="w-4 h-4 flex-shrink-0" />
                  </Link>
                </Button>
              </div>
            </div>

            <div className="lg:col-span-8">
              {/* Error/Success Alerts */}
              {(prefsError || prefsSuccess) && (
                <div className="mb-6 space-y-3">
                  {prefsError && (
                    <Alert variant="destructive" className="bg-red-50/50 dark:bg-red-900/20 border-red-200 dark:border-red-800">
                      <AlertCircle className="h-4 w-4" />
                      <AlertDescription>{prefsError}</AlertDescription>
                    </Alert>
                  )}
                  {prefsSuccess && (
                    <Alert className="bg-green-50/50 dark:bg-green-900/20 border-green-200 dark:border-green-800">
                      <CheckCircle className="h-4 w-4 text-green-600" />
                      <AlertDescription className="text-green-700 dark:text-green-300">{prefsSuccess}</AlertDescription>
                    </Alert>
                  )}
                </div>
              )}

              <div className="grid sm:grid-cols-2 gap-4">
                {[
                  { id: 'trading', label: 'Trading Alerts', desc: 'Price movements & portfolio', icon: '💹' },
                  { id: 'security', label: 'Security Alerts', desc: 'Auth & security warnings', icon: '🛡️' },
                  { id: 'account', label: 'Account Updates', desc: 'Profile & subscription', icon: '👤' },
                  { id: 'system', label: 'System Status', desc: 'Maintenance & features', icon: '⚙️' },
                  { id: 'marketing', label: 'Promotions', desc: 'News & special offers', icon: '🎁' }
                ].map((item) => (
                  <label key={item.id} className="cursor-pointer group">
                    <div className="flex items-center justify-between p-4 rounded-2xl bg-white/50 dark:bg-gray-900/40 border-2 border-gray-100 dark:border-gray-800 group-hover:border-purple-200 dark:group-hover:border-purple-800/50 transition-all duration-200">
                      <div className="flex gap-4">
                        <div className="text-2xl mt-1">{item.icon}</div>
                        <div>
                          <div className="font-bold text-gray-900 dark:text-gray-100 group-hover:text-purple-600 dark:group-hover:text-purple-400 transition-colors">{item.label}</div>
                          <div className="text-xs text-gray-500 dark:text-gray-400">{item.desc}</div>
                        </div>
                      </div>
                      <input
                        type="checkbox"
                        checked={notificationPrefs[item.id as keyof NotificationPreferences]}
                        onChange={(e) => updateNotificationPrefs({
                          ...notificationPrefs,
                          [item.id]: e.target.checked
                        })}
                        className="h-5 w-5 rounded-lg border-2 border-gray-300 text-purple-600 focus:ring-purple-500 cursor-pointer"
                      />
                    </div>
                  </label>
                ))}
              </div>
            </div>
          </div>
        </div>

        {/* Data & Privacy Section (Simplified) */}
        <div className="flex flex-col sm:flex-row items-center justify-between gap-6 p-8 rounded-[2rem] bg-indigo-600 text-white shadow-xl overflow-hidden relative">
          <div className="relative z-10 space-y-2 text-center sm:text-left">
            <h3 className="text-xl font-bold flex items-center gap-2 justify-center sm:justify-start">
              <Lock className="w-5 h-5" /> Privacy & Data Security
            </h3>
            <p className="text-indigo-100 text-sm max-w-lg">
              Your account data is secured with industrial-grade encryption and protocol-level security.
            </p>
          </div>
          <Button className="relative z-10 bg-white text-indigo-600 hover:bg-white/90 font-bold px-8 rounded-xl" asChild>
            <Link href="/privacy">Read Policy</Link>
          </Button>

          {/* Background visuals for the banner */}
          <div className="absolute top-0 right-0 -mr-16 -mt-16 h-48 w-48 rounded-full bg-white/10 blur-3xl"></div>
          <div className="absolute bottom-0 left-0 -ml-16 -mb-16 h-32 w-32 rounded-full bg-indigo-400/20 blur-2xl"></div>
        </div>
      </div>
    </div>
  );
}