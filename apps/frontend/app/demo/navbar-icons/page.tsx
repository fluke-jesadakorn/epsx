import { ChainSelector } from '@/components/nav/ChainSelector';
import { WalletProviderIcon } from '@/components/nav/WalletProviderIcon';
import { UserManagementDropdown } from '@/components/nav/UserManagementDropdown';
import { NotificationBellSimple } from '@/components/notifications/NotificationBellSimple';

export default function NavbarIconsDemo() {
  return (
    <div className="min-h-screen bg-gradient-to-br from-orange-50 to-purple-50 dark:from-slate-900 dark:to-slate-800 p-4">
      <div className="max-w-7xl mx-auto">
        <h1 className="text-4xl font-bold text-slate-900 dark:text-slate-100 mb-2">
          📍 Navbar Icons Demo
        </h1>
        <p className="text-slate-600 dark:text-slate-400 mb-8">
          Visual demonstration of the new navbar icon features
        </p>

        {/* Live Navbar Preview */}
        <section className="mb-12">
          <h2 className="text-2xl font-semibold text-slate-800 dark:text-slate-200 mb-4">
            🎯 Live Navbar Preview
          </h2>
          <div className="bg-white/90 backdrop-blur-xl dark:bg-slate-900/95 border border-orange-100/50 dark:border-slate-700/50 rounded-2xl p-8">
            <div className="flex items-center gap-8 justify-center mb-6">
              {/* Notification Bell */}
              <div className="text-center bg-blue-50 dark:bg-blue-900/20 p-6 rounded-xl border-2 border-blue-200/50 dark:border-blue-700/50 min-w-[140px]">
                <div className="mb-3">
                  <NotificationBellSimple compact={true} />
                </div>
                <h4 className="font-semibold text-blue-700 dark:text-blue-300 mb-1">Notifications</h4>
                <p className="text-xs text-blue-600 dark:text-blue-400">🔔 Real-time alerts</p>
                <p className="text-xs text-slate-500 mt-1">Click to view notifications</p>
              </div>

              {/* Wallet Provider Icon */}
              <div className="text-center bg-orange-50 dark:bg-orange-900/20 p-6 rounded-xl border-2 border-orange-200/50 dark:border-orange-700/50 min-w-[140px]">
                <div className="mb-3">
                  <WalletProviderIcon compact={true} />
                </div>
                <h4 className="font-semibold text-orange-700 dark:text-orange-300 mb-1">Wallet Provider</h4>
                <p className="text-xs text-orange-600 dark:text-orange-400">🦊 MetaMask/WalletConnect</p>
                <p className="text-xs text-slate-500 mt-1">Click to manage wallet</p>
              </div>

              {/* Chain Selector */}
              <div className="text-center bg-yellow-50 dark:bg-yellow-900/20 p-6 rounded-xl border-2 border-yellow-200/50 dark:border-yellow-700/50 min-w-[140px]">
                <div className="mb-3">
                  <ChainSelector compact={true} />
                </div>
                <h4 className="font-semibold text-yellow-700 dark:text-yellow-300 mb-1">Chain Selection</h4>
                <p className="text-xs text-yellow-600 dark:text-yellow-400">🔗 BSC Network Switch</p>
                <p className="text-xs text-slate-500 mt-1">Click to switch networks</p>
              </div>

              {/* User Management */}
              <div className="text-center bg-purple-50 dark:bg-purple-900/20 p-6 rounded-xl border-2 border-purple-200/50 dark:border-purple-700/50 min-w-[140px]">
                <div className="mb-3">
                  <UserManagementDropdown compact={true} />
                </div>
                <h4 className="font-semibold text-purple-700 dark:text-purple-300 mb-1">User Management</h4>
                <p className="text-xs text-purple-600 dark:text-purple-400">👤 Account & Settings</p>
                <p className="text-xs text-slate-500 mt-1">Click to access profile</p>
              </div>
            </div>
            
            <div className="text-center p-4 bg-green-50 dark:bg-green-900/20 rounded-lg border border-green-200/50 dark:border-green-700/50">
              <p className="text-sm font-medium text-green-700 dark:text-green-300 mb-1">
                ✅ All 4 new navbar icons are now active and functional!
              </p>
              <p className="text-xs text-green-600 dark:text-green-400">
                Each icon provides dropdown menus with comprehensive Web3 and user management features
              </p>
            </div>
          </div>
        </section>

        {/* Individual Component Demos */}
        <div className="grid lg:grid-cols-2 gap-8 mb-12">
          {/* Notification Bell Demo */}
          <div className="bg-white/80 backdrop-blur-xl dark:bg-slate-900/80 border border-orange-100/50 dark:border-slate-700/50 rounded-2xl p-8">
            <h3 className="text-2xl font-semibold text-slate-800 dark:text-slate-200 mb-6 flex items-center gap-3">
              🔔 Enhanced Notification Bell
            </h3>
            <div className="space-y-6">
              <div className="flex items-center gap-6 p-4 bg-blue-50 dark:bg-blue-900/20 rounded-xl">
                <NotificationBellSimple compact={true} />
                <div>
                  <span className="text-lg font-medium text-slate-700 dark:text-slate-300">Live Component</span>
                  <p className="text-sm text-slate-600 dark:text-slate-400">Click to see notifications in action</p>
                </div>
              </div>
              
              <div className="grid grid-cols-1 gap-4">
                <div className="p-4 bg-slate-50 dark:bg-slate-800 rounded-lg">
                  <h4 className="font-semibold text-slate-800 dark:text-slate-200 mb-2">✨ Key Features:</h4>
                  <ul className="space-y-2 text-sm text-slate-600 dark:text-slate-400">
                    <li className="flex items-start gap-2">
                      <span className="text-blue-500">•</span>
                      <span><strong>Real-time Badge:</strong> Live notification count with animated pulse effect</span>
                    </li>
                    <li className="flex items-start gap-2">
                      <span className="text-green-500">•</span>
                      <span><strong>Rich Preview:</strong> Full notification cards with timestamps and actions</span>
                    </li>
                    <li className="flex items-start gap-2">
                      <span className="text-orange-500">•</span>
                      <span><strong>Auto-Refresh:</strong> Polls backend every 60 seconds for new notifications</span>
                    </li>
                    <li className="flex items-start gap-2">
                      <span className="text-purple-500">•</span>
                      <span><strong>Mobile-First:</strong> Responsive sheet view for mobile devices</span>
                    </li>
                  </ul>
                </div>
                
                <div className="p-4 bg-blue-50 dark:bg-blue-900/20 rounded-lg">
                  <h4 className="font-semibold text-blue-700 dark:text-blue-300 mb-2">🔧 Technical Details:</h4>
                  <ul className="space-y-1 text-sm text-blue-600 dark:text-blue-400">
                    <li>• Server-side initial data loading</li>
                    <li>• Client-side polling with Bearer token auth</li>
                    <li>• Browser notification integration</li>
                    <li>• Optimistic UI updates</li>
                  </ul>
                </div>
              </div>
            </div>
          </div>

          {/* Wallet Provider Demo */}
          <div className="bg-white/80 backdrop-blur-xl dark:bg-slate-900/80 border border-orange-100/50 dark:border-slate-700/50 rounded-2xl p-8">
            <h3 className="text-2xl font-semibold text-slate-800 dark:text-slate-200 mb-6 flex items-center gap-3">
              🦊 Smart Wallet Provider Detection
            </h3>
            <div className="space-y-6">
              <div className="grid grid-cols-2 gap-4">
                <div className="flex items-center gap-4 p-4 bg-orange-50 dark:bg-orange-900/20 rounded-xl">
                  <WalletProviderIcon compact={true} />
                  <div>
                    <span className="text-sm font-medium text-slate-700 dark:text-slate-300">Compact</span>
                    <p className="text-xs text-slate-600 dark:text-slate-400">Navbar mode</p>
                  </div>
                </div>
                <div className="flex items-center gap-4 p-4 bg-orange-50 dark:bg-orange-900/20 rounded-xl">
                  <WalletProviderIcon compact={false} />
                  <div>
                    <span className="text-sm font-medium text-slate-700 dark:text-slate-300">Expanded</span>
                    <p className="text-xs text-slate-600 dark:text-slate-400">Mobile mode</p>
                  </div>
                </div>
              </div>
              
              <div className="grid grid-cols-1 gap-4">
                <div className="p-4 bg-slate-50 dark:bg-slate-800 rounded-lg">
                  <h4 className="font-semibold text-slate-800 dark:text-slate-200 mb-3">🔍 Wallet Detection Matrix:</h4>
                  <div className="grid grid-cols-2 gap-3 text-sm">
                    <div className="flex items-center gap-2">
                      <span className="text-orange-500">🦊</span>
                      <span className="text-slate-600 dark:text-slate-400">MetaMask Browser Extension</span>
                    </div>
                    <div className="flex items-center gap-2">
                      <span className="text-blue-500">🔗</span>
                      <span className="text-slate-600 dark:text-slate-400">WalletConnect Protocol</span>
                    </div>
                    <div className="flex items-center gap-2">
                      <span className="text-blue-600">🔵</span>
                      <span className="text-slate-600 dark:text-slate-400">Coinbase Wallet</span>
                    </div>
                    <div className="flex items-center gap-2">
                      <span className="text-purple-500">🌈</span>
                      <span className="text-slate-600 dark:text-slate-400">Rainbow Wallet</span>
                    </div>
                  </div>
                </div>
                
                <div className="p-4 bg-orange-50 dark:bg-orange-900/20 rounded-lg">
                  <h4 className="font-semibold text-orange-700 dark:text-orange-300 mb-2">🎯 Dropdown Features:</h4>
                  <ul className="space-y-2 text-sm text-orange-600 dark:text-orange-400">
                    <li className="flex items-start gap-2">
                      <span>📋</span>
                      <span><strong>One-Click Copy:</strong> Copy wallet address to clipboard</span>
                    </li>
                    <li className="flex items-start gap-2">
                      <span>🔍</span>
                      <span><strong>BSCScan Link:</strong> View wallet on blockchain explorer</span>
                    </li>
                    <li className="flex items-start gap-2">
                      <span>📊</span>
                      <span><strong>Connection Status:</strong> Real-time connection monitoring</span>
                    </li>
                    <li className="flex items-start gap-2">
                      <span>🎨</span>
                      <span><strong>Provider Branding:</strong> Dynamic icons and colors</span>
                    </li>
                  </ul>
                </div>
              </div>
            </div>
          </div>

          {/* Chain Selector Demo */}
          <div className="bg-white/80 backdrop-blur-xl dark:bg-slate-900/80 border border-orange-100/50 dark:border-slate-700/50 rounded-2xl p-8">
            <h3 className="text-2xl font-semibold text-slate-800 dark:text-slate-200 mb-6 flex items-center gap-3">
              🔗 Intelligent Chain Selection
            </h3>
            <div className="space-y-6">
              <div className="grid grid-cols-2 gap-4">
                <div className="flex items-center gap-4 p-4 bg-yellow-50 dark:bg-yellow-900/20 rounded-xl">
                  <ChainSelector compact={true} />
                  <div>
                    <span className="text-sm font-medium text-slate-700 dark:text-slate-300">Compact</span>
                    <p className="text-xs text-slate-600 dark:text-slate-400">Navbar mode</p>
                  </div>
                </div>
                <div className="flex items-center gap-4 p-4 bg-yellow-50 dark:bg-yellow-900/20 rounded-xl">
                  <ChainSelector compact={false} />
                  <div>
                    <span className="text-sm font-medium text-slate-700 dark:text-slate-300">Expanded</span>
                    <p className="text-xs text-slate-600 dark:text-slate-400">Mobile mode</p>
                  </div>
                </div>
              </div>
              
              <div className="grid grid-cols-1 gap-4">
                <div className="p-4 bg-slate-50 dark:bg-slate-800 rounded-lg">
                  <h4 className="font-semibold text-slate-800 dark:text-slate-200 mb-3">⛓️ Supported Networks:</h4>
                  <div className="grid grid-cols-2 gap-4">
                    <div className="p-3 bg-yellow-100 dark:bg-yellow-900/30 rounded-lg">
                      <div className="flex items-center gap-2 mb-2">
                        <span className="text-2xl">🟡</span>
                        <span className="font-medium text-yellow-800 dark:text-yellow-200">BSC Mainnet</span>
                      </div>
                      <ul className="text-xs text-yellow-700 dark:text-yellow-300 space-y-1">
                        <li>• Chain ID: 56</li>
                        <li>• Production network</li>
                        <li>• Real BNB transactions</li>
                        <li>• Live trading environment</li>
                      </ul>
                    </div>
                    <div className="p-3 bg-orange-100 dark:bg-orange-900/30 rounded-lg">
                      <div className="flex items-center gap-2 mb-2">
                        <span className="text-2xl">🟠</span>
                        <span className="font-medium text-orange-800 dark:text-orange-200">BSC Testnet</span>
                      </div>
                      <ul className="text-xs text-orange-700 dark:text-orange-300 space-y-1">
                        <li>• Chain ID: 97</li>
                        <li>• Development network</li>
                        <li>• Test BNB (free)</li>
                        <li>• Safe testing environment</li>
                      </ul>
                    </div>
                  </div>
                </div>
                
                <div className="p-4 bg-yellow-50 dark:bg-yellow-900/20 rounded-lg">
                  <h4 className="font-semibold text-yellow-700 dark:text-yellow-300 mb-2">⚡ Smart Features:</h4>
                  <ul className="space-y-2 text-sm text-yellow-600 dark:text-yellow-400">
                    <li className="flex items-start gap-2">
                      <span>🔄</span>
                      <span><strong>One-Click Switching:</strong> Instant network changes with wagmi</span>
                    </li>
                    <li className="flex items-start gap-2">
                      <span>📡</span>
                      <span><strong>Status Monitoring:</strong> Real-time connection and sync status</span>
                    </li>
                    <li className="flex items-start gap-2">
                      <span>⚠️</span>
                      <span><strong>Network Validation:</strong> Warns about unsupported networks</span>
                    </li>
                    <li className="flex items-start gap-2">
                      <span>🎯</span>
                      <span><strong>Environment Sync:</strong> Auto-detects mainnet/testnet config</span>
                    </li>
                  </ul>
                </div>
              </div>
            </div>
          </div>

          {/* User Management Demo */}
          <div className="bg-white/80 backdrop-blur-xl dark:bg-slate-900/80 border border-orange-100/50 dark:border-slate-700/50 rounded-2xl p-8">
            <h3 className="text-2xl font-semibold text-slate-800 dark:text-slate-200 mb-6 flex items-center gap-3">
              👤 Progressive User Management
            </h3>
            <div className="space-y-6">
              <div className="grid grid-cols-2 gap-4">
                <div className="flex items-center gap-4 p-4 bg-purple-50 dark:bg-purple-900/20 rounded-xl">
                  <UserManagementDropdown compact={true} />
                  <div>
                    <span className="text-sm font-medium text-slate-700 dark:text-slate-300">Compact</span>
                    <p className="text-xs text-slate-600 dark:text-slate-400">Navbar mode</p>
                  </div>
                </div>
                <div className="flex items-center gap-4 p-4 bg-purple-50 dark:bg-purple-900/20 rounded-xl">
                  <UserManagementDropdown compact={false} />
                  <div>
                    <span className="text-sm font-medium text-slate-700 dark:text-slate-300">Expanded</span>
                    <p className="text-xs text-slate-600 dark:text-slate-400">Mobile mode</p>
                  </div>
                </div>
              </div>
              
              <div className="grid grid-cols-1 gap-4">
                <div className="p-4 bg-slate-50 dark:bg-slate-800 rounded-lg">
                  <h4 className="font-semibold text-slate-800 dark:text-slate-200 mb-3">🔐 Authentication Levels:</h4>
                  <div className="grid grid-cols-3 gap-3">
                    <div className="p-3 bg-slate-100 dark:bg-slate-700 rounded-lg text-center">
                      <div className="flex items-center justify-center gap-1 mb-2">
                        <span className="text-slate-400">👁️</span>
                        <span className="text-xs font-medium text-slate-600 dark:text-slate-400">PUBLIC</span>
                      </div>
                      <p className="text-xs text-slate-500">Browse only</p>
                    </div>
                    <div className="p-3 bg-blue-100 dark:bg-blue-900/30 rounded-lg text-center">
                      <div className="flex items-center justify-center gap-1 mb-2">
                        <span className="text-blue-500">🔗</span>
                        <span className="text-xs font-medium text-blue-700 dark:text-blue-300">CONNECTED</span>
                      </div>
                      <p className="text-xs text-blue-600 dark:text-blue-400">Wallet linked</p>
                    </div>
                    <div className="p-3 bg-green-100 dark:bg-green-900/30 rounded-lg text-center">
                      <div className="flex items-center justify-center gap-1 mb-2">
                        <span className="text-green-500">🛡️</span>
                        <span className="text-xs font-medium text-green-700 dark:text-green-300">AUTHENTICATED</span>
                      </div>
                      <p className="text-xs text-green-600 dark:text-green-400">Full access</p>
                    </div>
                  </div>
                </div>
                
                <div className="p-4 bg-purple-50 dark:bg-purple-900/20 rounded-lg">
                  <h4 className="font-semibold text-purple-700 dark:text-purple-300 mb-3">🎯 Context-Aware Menu Items:</h4>
                  <div className="grid grid-cols-2 gap-4 text-sm">
                    <div>
                      <h5 className="font-medium text-purple-800 dark:text-purple-200 mb-2">Always Available:</h5>
                      <ul className="space-y-1 text-purple-600 dark:text-purple-400">
                        <li className="flex items-center gap-2"><span>👤</span>Profile</li>
                        <li className="flex items-center gap-2"><span>⚙️</span>Settings</li>
                      </ul>
                    </div>
                    <div>
                      <h5 className="font-medium text-purple-800 dark:text-purple-200 mb-2">Connected+ Only:</h5>
                      <ul className="space-y-1 text-purple-600 dark:text-purple-400">
                        <li className="flex items-center gap-2"><span>🔗</span>Wallet</li>
                        <li className="flex items-center gap-2"><span>💳</span>Billing</li>
                        <li className="flex items-center gap-2"><span>🔔</span>Notifications</li>
                        <li className="flex items-center gap-2"><span>🛡️</span>Security</li>
                      </ul>
                    </div>
                  </div>
                </div>
                
                <div className="p-4 bg-gradient-to-r from-purple-50 to-pink-50 dark:from-purple-900/20 dark:to-pink-900/20 rounded-lg">
                  <h4 className="font-semibold text-purple-700 dark:text-purple-300 mb-2">✨ Smart Upgrade Prompts:</h4>
                  <ul className="space-y-2 text-sm text-purple-600 dark:text-purple-400">
                    <li className="flex items-start gap-2">
                      <span>🔄</span>
                      <span><strong>Progressive Enhancement:</strong> Suggests next auth level upgrade</span>
                    </li>
                    <li className="flex items-start gap-2">
                      <span>🎯</span>
                      <span><strong>Feature Hints:</strong> Shows benefits of upgrading access</span>
                    </li>
                    <li className="flex items-start gap-2">
                      <span>📱</span>
                      <span><strong>Seamless Flow:</strong> One-click upgrade paths</span>
                    </li>
                  </ul>
                </div>
              </div>
            </div>
          </div>
        </div>

        {/* Authentication States Demo */}
        <section className="mb-12">
          <h2 className="text-2xl font-semibold text-slate-800 dark:text-slate-200 mb-4">
            🔐 Authentication States
          </h2>
          <div className="bg-white/80 backdrop-blur-xl dark:bg-slate-900/80 border border-orange-100/50 dark:border-slate-700/50 rounded-2xl p-6">
            <div className="grid md:grid-cols-3 gap-4">
              <div className="text-center p-4 bg-slate-50 dark:bg-slate-800 rounded-lg">
                <div className="flex items-center justify-center gap-2 mb-2">
                  <span className="text-slate-400">👁️</span>
                  <span className="text-sm font-medium">Public</span>
                </div>
                <p className="text-xs text-slate-500">Browse only access</p>
              </div>
              <div className="text-center p-4 bg-blue-50 dark:bg-blue-900/20 rounded-lg">
                <div className="flex items-center justify-center gap-2 mb-2">
                  <span className="text-blue-500">🔗</span>
                  <span className="text-sm font-medium">Connected</span>
                </div>
                <p className="text-xs text-slate-500">Wallet connected</p>
              </div>
              <div className="text-center p-4 bg-green-50 dark:bg-green-900/20 rounded-lg">
                <div className="flex items-center justify-center gap-2 mb-2">
                  <span className="text-green-500">🛡️</span>
                  <span className="text-sm font-medium">Authenticated</span>
                </div>
                <p className="text-xs text-slate-500">Full access</p>
              </div>
            </div>
          </div>
        </section>

        {/* Technical Details */}
        <section>
          <h2 className="text-2xl font-semibold text-slate-800 dark:text-slate-200 mb-4">
            ⚙️ Technical Implementation
          </h2>
          <div className="bg-white/80 backdrop-blur-xl dark:bg-slate-900/80 border border-orange-100/50 dark:border-slate-700/50 rounded-2xl p-6">
            <div className="grid md:grid-cols-2 gap-6">
              <div>
                <h4 className="font-medium text-slate-800 dark:text-slate-200 mb-2">Design Principles</h4>
                <ul className="text-sm text-slate-600 dark:text-slate-400 space-y-1">
                  <li>✅ Zero animation policy compliance</li>
                  <li>✅ Consistent orange theme styling</li>
                  <li>✅ 40px height buttons with rounded design</li>
                  <li>✅ Mobile-responsive hamburger menu</li>
                  <li>✅ SSR-safe hydration handling</li>
                </ul>
              </div>
              <div>
                <h4 className="font-medium text-slate-800 dark:text-slate-200 mb-2">Web3 Integration</h4>
                <ul className="text-sm text-slate-600 dark:text-slate-400 space-y-1">
                  <li>🔗 Wagmi hooks for wallet interaction</li>
                  <li>🌐 RainbowKit connector support</li>
                  <li>⛓️ BSC Mainnet/Testnet switching</li>
                  <li>🔐 Progressive authentication levels</li>
                  <li>📱 Mobile-optimized Web3 experience</li>
                </ul>
              </div>
            </div>
          </div>
        </section>

        <div className="mt-8 text-center">
          <p className="text-sm text-slate-500 dark:text-slate-400">
            🎉 All navbar icons are now implemented and functional!
          </p>
        </div>
      </div>
    </div>
  );
}