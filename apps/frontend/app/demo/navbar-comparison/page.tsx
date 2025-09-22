import { Bell, Sun, Moon, User, Wallet } from 'lucide-react';
import { ChainSelector } from '@/components/nav/ChainSelector';
import { WalletProviderIcon } from '@/components/nav/WalletProviderIcon';
import { UserManagementDropdown } from '@/components/nav/UserManagementDropdown';
import { NotificationBellSimple } from '@/components/notifications/NotificationBellSimple';
import { UnifiedThemeToggle } from '@/components/ui/UnifiedThemeToggle';

export default function NavbarComparison() {
  return (
    <div className="min-h-screen bg-gradient-to-br from-orange-50 to-purple-50 dark:from-slate-900 dark:to-slate-800 p-4">
      <div className="max-w-8xl mx-auto">
        <h1 className="text-4xl font-bold text-slate-900 dark:text-slate-100 mb-2">
          🔄 Navbar Enhancement Comparison
        </h1>
        <p className="text-slate-600 dark:text-slate-400 mb-8">
          Before and after implementation of the new navbar icon features
        </p>

        {/* Before Section */}
        <section className="mb-8">
          <h2 className="text-2xl font-semibold text-slate-800 dark:text-slate-200 mb-4 flex items-center gap-2">
            ❌ BEFORE: Limited Navbar
          </h2>
          <div className="bg-white/90 backdrop-blur-xl dark:bg-slate-900/95 border border-orange-100/50 dark:border-slate-700/50 rounded-2xl p-6">
            <div className="flex items-center justify-between">
              <div className="flex items-center gap-4">
                <span className="text-2xl font-bold bg-gradient-to-r from-blue-500 to-purple-600 bg-clip-text text-transparent">
                  EPSX
                </span>
                <nav className="flex items-center gap-2">
                  <span className="text-sm text-slate-600 dark:text-slate-400">Analytics</span>
                  <span className="text-sm text-slate-600 dark:text-slate-400">Settings</span>
                  <span className="text-sm text-slate-600 dark:text-slate-400">About</span>
                </nav>
              </div>
              
              {/* Old limited right actions */}
              <div className="flex items-center gap-3">
                <div className="flex items-center gap-1 bg-orange-50 dark:bg-slate-800 p-2 rounded-full">
                  <Bell className="h-4 w-4 text-orange-500" />
                </div>
                <div className="flex items-center gap-1 bg-orange-50 dark:bg-slate-800 p-2 rounded-full">
                  <Sun className="h-4 w-4 text-orange-500" />
                </div>
                <div className="flex items-center gap-2 bg-orange-50 dark:bg-slate-800 px-3 py-2 rounded-full">
                  <Wallet className="h-4 w-4 text-orange-500" />
                  <span className="text-xs text-orange-500">Connect</span>
                </div>
              </div>
            </div>
            
            <div className="mt-4 p-3 bg-red-50 dark:bg-red-900/20 rounded-lg">
              <p className="text-sm text-red-700 dark:text-red-400">
                <strong>Problems:</strong> No chain selection, no wallet provider info, no user management, generic Web3 auth
              </p>
            </div>
          </div>
        </section>

        {/* After Section */}
        <section className="mb-8">
          <h2 className="text-2xl font-semibold text-slate-800 dark:text-slate-200 mb-4 flex items-center gap-2">
            ✅ AFTER: Enhanced Navbar with 4 New Icons
          </h2>
          <div className="bg-white/90 backdrop-blur-xl dark:bg-slate-900/95 border border-orange-100/50 dark:border-slate-700/50 rounded-2xl p-6">
            <div className="flex items-center justify-between">
              <div className="flex items-center gap-4">
                <span className="text-2xl font-bold bg-gradient-to-r from-blue-500 to-purple-600 bg-clip-text text-transparent">
                  EPSX
                </span>
                <nav className="flex items-center gap-2">
                  <span className="text-sm text-slate-600 dark:text-slate-400">Analytics</span>
                  <span className="text-sm text-slate-600 dark:text-slate-400">Settings</span>
                  <span className="text-sm text-slate-600 dark:text-slate-400">About</span>
                </nav>
              </div>
              
              {/* New enhanced right actions */}
              <div className="flex items-center gap-3">
                <div className="relative">
                  <NotificationBellSimple compact={true} />
                  <span className="absolute -top-1 -right-1 text-xs bg-red-500 text-white rounded-full px-1">1</span>
                </div>
                <WalletProviderIcon compact={true} />
                <ChainSelector compact={true} />
                <UserManagementDropdown compact={true} />
                <UnifiedThemeToggle variant="minimal" />
              </div>
            </div>
            
            <div className="mt-4 p-3 bg-green-50 dark:bg-green-900/20 rounded-lg">
              <p className="text-sm text-green-700 dark:text-green-400">
                <strong>Improvements:</strong> Complete Web3 integration, user management, chain switching, wallet provider detection
              </p>
            </div>
          </div>
        </section>

        {/* Feature Breakdown */}
        <section className="mb-8">
          <h2 className="text-2xl font-semibold text-slate-800 dark:text-slate-200 mb-4">
            📋 New Features Breakdown
          </h2>
          <div className="grid md:grid-cols-2 lg:grid-cols-4 gap-4">
            {/* Feature 1 */}
            <div className="bg-white/80 backdrop-blur-xl dark:bg-slate-900/80 border border-orange-100/50 dark:border-slate-700/50 rounded-xl p-4">
              <div className="text-center mb-3">
                <div className="bg-blue-100 dark:bg-blue-900/30 p-3 rounded-full w-fit mx-auto mb-2">
                  🔔
                </div>
                <h3 className="font-semibold text-slate-800 dark:text-slate-200">Enhanced Notifications</h3>
              </div>
              <ul className="text-xs text-slate-600 dark:text-slate-400 space-y-1">
                <li>• Real-time badge count</li>
                <li>• Notification preview</li>
                <li>• Mobile sheet view</li>
                <li>• Auto-refresh (60s)</li>
              </ul>
            </div>

            {/* Feature 2 */}
            <div className="bg-white/80 backdrop-blur-xl dark:bg-slate-900/80 border border-orange-100/50 dark:border-slate-700/50 rounded-xl p-4">
              <div className="text-center mb-3">
                <div className="bg-orange-100 dark:bg-orange-900/30 p-3 rounded-full w-fit mx-auto mb-2">
                  🦊
                </div>
                <h3 className="font-semibold text-slate-800 dark:text-slate-200">Wallet Provider</h3>
              </div>
              <ul className="text-xs text-slate-600 dark:text-slate-400 space-y-1">
                <li>• Auto-detect provider</li>
                <li>• Address copy/paste</li>
                <li>• BSCScan explorer link</li>
                <li>• Connection status</li>
              </ul>
            </div>

            {/* Feature 3 */}
            <div className="bg-white/80 backdrop-blur-xl dark:bg-slate-900/80 border border-orange-100/50 dark:border-slate-700/50 rounded-xl p-4">
              <div className="text-center mb-3">
                <div className="bg-yellow-100 dark:bg-yellow-900/30 p-3 rounded-full w-fit mx-auto mb-2">
                  🔗
                </div>
                <h3 className="font-semibold text-slate-800 dark:text-slate-200">Chain Selection</h3>
              </div>
              <ul className="text-xs text-slate-600 dark:text-slate-400 space-y-1">
                <li>• BSC Mainnet/Testnet</li>
                <li>• Network switching</li>
                <li>• Visual indicators</li>
                <li>• Status monitoring</li>
              </ul>
            </div>

            {/* Feature 4 */}
            <div className="bg-white/80 backdrop-blur-xl dark:bg-slate-900/80 border border-orange-100/50 dark:border-slate-700/50 rounded-xl p-4">
              <div className="text-center mb-3">
                <div className="bg-purple-100 dark:bg-purple-900/30 p-3 rounded-full w-fit mx-auto mb-2">
                  👤
                </div>
                <h3 className="font-semibold text-slate-800 dark:text-slate-200">User Management</h3>
              </div>
              <ul className="text-xs text-slate-600 dark:text-slate-400 space-y-1">
                <li>• Progressive auth levels</li>
                <li>• Context-aware menus</li>
                <li>• Profile/Settings links</li>
                <li>• Upgrade prompts</li>
              </ul>
            </div>
          </div>
        </section>

        {/* Implementation Stats */}
        <section>
          <h2 className="text-2xl font-semibold text-slate-800 dark:text-slate-200 mb-4">
            📊 Implementation Statistics
          </h2>
          <div className="bg-white/80 backdrop-blur-xl dark:bg-slate-900/80 border border-orange-100/50 dark:border-slate-700/50 rounded-xl p-6">
            <div className="grid md:grid-cols-4 gap-6 text-center">
              <div>
                <div className="text-3xl font-bold text-blue-500 mb-1">4</div>
                <div className="text-sm text-slate-600 dark:text-slate-400">New Components</div>
              </div>
              <div>
                <div className="text-3xl font-bold text-green-500 mb-1">100%</div>
                <div className="text-sm text-slate-600 dark:text-slate-400">Mobile Responsive</div>
              </div>
              <div>
                <div className="text-3xl font-bold text-orange-500 mb-1">0</div>
                <div className="text-sm text-slate-600 dark:text-slate-400">Animations Added</div>
              </div>
              <div>
                <div className="text-3xl font-bold text-purple-500 mb-1">3</div>
                <div className="text-sm text-slate-600 dark:text-slate-400">Auth Levels</div>
              </div>
            </div>
          </div>
        </section>

        <div className="mt-8 text-center">
          <p className="text-lg font-medium text-slate-700 dark:text-slate-300 mb-2">
            🎯 Mission Accomplished!
          </p>
          <p className="text-sm text-slate-500 dark:text-slate-400">
            Navigate to <code className="bg-slate-100 dark:bg-slate-800 px-2 py-1 rounded">http://localhost:3000</code> to see the enhanced navbar in action
          </p>
        </div>
      </div>
    </div>
  );
}