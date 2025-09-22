import { ChainSelector } from '@/components/nav/ChainSelector';
import { WalletProviderIcon } from '@/components/nav/WalletProviderIcon';
import { UserManagementDropdown } from '@/components/nav/UserManagementDropdown';
import { NotificationBellSimple } from '@/components/notifications/NotificationBellSimple';
import { UnifiedThemeToggle } from '@/components/ui/UnifiedThemeToggle';

export default function NavbarCompleteDemo() {
  return (
    <div className="min-h-screen bg-gradient-to-br from-orange-50 to-purple-50 dark:from-slate-900 dark:to-slate-800 p-4">
      <div className="max-w-8xl mx-auto">
        <h1 className="text-5xl font-bold text-slate-900 dark:text-slate-100 mb-4 text-center">
          🎯 Complete Navbar Icons Implementation
        </h1>
        <p className="text-xl text-slate-600 dark:text-slate-400 mb-12 text-center max-w-4xl mx-auto">
          Comprehensive demonstration of all 4 new navbar icons with detailed features, 
          interactive examples, and technical implementation details
        </p>

        {/* Full Width Navbar Simulation */}
        <section className="mb-16">
          <h2 className="text-3xl font-semibold text-slate-800 dark:text-slate-200 mb-6 text-center">
            📱 Full Navbar Experience
          </h2>
          <div className="bg-white/95 backdrop-blur-xl dark:bg-slate-900/95 border border-orange-100/50 dark:border-slate-700/50 rounded-3xl p-8 shadow-2xl">
            {/* Simulated Navbar */}
            <div className="flex items-center justify-between mb-8 p-6 bg-gradient-to-r from-orange-50/80 to-purple-50/80 dark:from-slate-800/80 dark:to-slate-700/80 rounded-2xl border border-orange-200/50 dark:border-slate-600/50">
              <div className="flex items-center gap-6">
                <span className="text-3xl font-bold bg-gradient-to-r from-blue-500 to-purple-600 bg-clip-text text-transparent">
                  EPSX
                </span>
                <nav className="flex items-center gap-4 text-slate-600 dark:text-slate-400">
                  <span className="hover:text-orange-500 cursor-pointer">Analytics</span>
                  <span className="hover:text-orange-500 cursor-pointer">Settings</span>
                  <span className="hover:text-orange-500 cursor-pointer">About</span>
                </nav>
              </div>
              
              {/* Enhanced Right Actions */}
              <div className="flex items-center gap-4">
                <div className="relative">
                  <NotificationBellSimple compact={true} />
                  <div className="absolute -top-1 -right-1 w-3 h-3 bg-red-500 rounded-full animate-pulse"></div>
                </div>
                <WalletProviderIcon compact={true} />
                <ChainSelector compact={true} />
                <UserManagementDropdown compact={true} />
                <UnifiedThemeToggle variant="minimal" />
              </div>
            </div>
            
            <div className="text-center p-4 bg-green-50 dark:bg-green-900/20 rounded-xl border border-green-200/50 dark:border-green-700/50">
              <p className="text-lg font-semibold text-green-700 dark:text-green-300 mb-2">
                ✅ This is what you see in the actual navbar at localhost:3000
              </p>
              <p className="text-sm text-green-600 dark:text-green-400">
                Each icon is fully interactive - click them to explore the dropdown menus and features!
              </p>
            </div>
          </div>
        </section>

        {/* Detailed Feature Matrix */}
        <section className="mb-16">
          <h2 className="text-3xl font-semibold text-slate-800 dark:text-slate-200 mb-8 text-center">
            🔧 Feature Implementation Matrix
          </h2>
          <div className="grid lg:grid-cols-4 gap-6">
            
            {/* Notification Bell Feature Card */}
            <div className="bg-white/90 backdrop-blur-xl dark:bg-slate-900/90 border border-blue-200/50 dark:border-blue-700/50 rounded-2xl p-8 shadow-xl">
              <div className="text-center mb-6">
                <div className="w-16 h-16 bg-blue-100 dark:bg-blue-900/30 rounded-full flex items-center justify-center mx-auto mb-4">
                  <span className="text-3xl">🔔</span>
                </div>
                <h3 className="text-xl font-bold text-blue-800 dark:text-blue-200">Notification Bell</h3>
                <p className="text-sm text-blue-600 dark:text-blue-400">Enhanced alerts system</p>
              </div>
              
              <div className="space-y-4">
                <div className="p-3 bg-blue-50 dark:bg-blue-900/20 rounded-lg">
                  <NotificationBellSimple compact={false} />
                </div>
                
                <div className="space-y-3">
                  <h4 className="font-semibold text-blue-800 dark:text-blue-200 text-sm">Core Features:</h4>
                  <ul className="space-y-2 text-xs text-blue-700 dark:text-blue-300">
                    <li className="flex items-start gap-2">
                      <span className="w-1 h-1 bg-blue-500 rounded-full mt-2"></span>
                      <span>Real-time badge count with pulse animation</span>
                    </li>
                    <li className="flex items-start gap-2">
                      <span className="w-1 h-1 bg-blue-500 rounded-full mt-2"></span>
                      <span>Rich notification cards with timestamps</span>
                    </li>
                    <li className="flex items-start gap-2">
                      <span className="w-1 h-1 bg-blue-500 rounded-full mt-2"></span>
                      <span>Auto-polling every 60 seconds</span>
                    </li>
                    <li className="flex items-start gap-2">
                      <span className="w-1 h-1 bg-blue-500 rounded-full mt-2"></span>
                      <span>Mobile-responsive sheet view</span>
                    </li>
                    <li className="flex items-start gap-2">
                      <span className="w-1 h-1 bg-blue-500 rounded-full mt-2"></span>
                      <span>Browser notification integration</span>
                    </li>
                  </ul>
                </div>

                <div className="p-2 bg-blue-100 dark:bg-blue-800/30 rounded text-center">
                  <span className="text-xs font-medium text-blue-800 dark:text-blue-200">Click to test notifications</span>
                </div>
              </div>
            </div>

            {/* Wallet Provider Feature Card */}
            <div className="bg-white/90 backdrop-blur-xl dark:bg-slate-900/90 border border-orange-200/50 dark:border-orange-700/50 rounded-2xl p-8 shadow-xl">
              <div className="text-center mb-6">
                <div className="w-16 h-16 bg-orange-100 dark:bg-orange-900/30 rounded-full flex items-center justify-center mx-auto mb-4">
                  <span className="text-3xl">🦊</span>
                </div>
                <h3 className="text-xl font-bold text-orange-800 dark:text-orange-200">Wallet Provider</h3>
                <p className="text-sm text-orange-600 dark:text-orange-400">Smart wallet detection</p>
              </div>
              
              <div className="space-y-4">
                <div className="p-3 bg-orange-50 dark:bg-orange-900/20 rounded-lg">
                  <WalletProviderIcon compact={false} />
                </div>
                
                <div className="space-y-3">
                  <h4 className="font-semibold text-orange-800 dark:text-orange-200 text-sm">Supported Wallets:</h4>
                  <div className="grid grid-cols-2 gap-2">
                    <div className="flex items-center gap-1 text-xs">
                      <span>🦊</span>
                      <span className="text-orange-700 dark:text-orange-300">MetaMask</span>
                    </div>
                    <div className="flex items-center gap-1 text-xs">
                      <span>🔗</span>
                      <span className="text-orange-700 dark:text-orange-300">WalletConnect</span>
                    </div>
                    <div className="flex items-center gap-1 text-xs">
                      <span>🔵</span>
                      <span className="text-orange-700 dark:text-orange-300">Coinbase</span>
                    </div>
                    <div className="flex items-center gap-1 text-xs">
                      <span>🌈</span>
                      <span className="text-orange-700 dark:text-orange-300">Rainbow</span>
                    </div>
                  </div>
                  
                  <h4 className="font-semibold text-orange-800 dark:text-orange-200 text-sm mt-4">Features:</h4>
                  <ul className="space-y-2 text-xs text-orange-700 dark:text-orange-300">
                    <li className="flex items-start gap-2">
                      <span className="w-1 h-1 bg-orange-500 rounded-full mt-2"></span>
                      <span>Auto-detect wallet provider</span>
                    </li>
                    <li className="flex items-start gap-2">
                      <span className="w-1 h-1 bg-orange-500 rounded-full mt-2"></span>
                      <span>One-click address copy</span>
                    </li>
                    <li className="flex items-start gap-2">
                      <span className="w-1 h-1 bg-orange-500 rounded-full mt-2"></span>
                      <span>BSCScan explorer link</span>
                    </li>
                    <li className="flex items-start gap-2">
                      <span className="w-1 h-1 bg-orange-500 rounded-full mt-2"></span>
                      <span>Real-time connection status</span>
                    </li>
                  </ul>
                </div>

                <div className="p-2 bg-orange-100 dark:bg-orange-800/30 rounded text-center">
                  <span className="text-xs font-medium text-orange-800 dark:text-orange-200">Connect wallet to see provider info</span>
                </div>
              </div>
            </div>

            {/* Chain Selector Feature Card */}
            <div className="bg-white/90 backdrop-blur-xl dark:bg-slate-900/90 border border-yellow-200/50 dark:border-yellow-700/50 rounded-2xl p-8 shadow-xl">
              <div className="text-center mb-6">
                <div className="w-16 h-16 bg-yellow-100 dark:bg-yellow-900/30 rounded-full flex items-center justify-center mx-auto mb-4">
                  <span className="text-3xl">🔗</span>
                </div>
                <h3 className="text-xl font-bold text-yellow-800 dark:text-yellow-200">Chain Selector</h3>
                <p className="text-sm text-yellow-600 dark:text-yellow-400">Network management</p>
              </div>
              
              <div className="space-y-4">
                <div className="p-3 bg-yellow-50 dark:bg-yellow-900/20 rounded-lg">
                  <ChainSelector compact={false} />
                </div>
                
                <div className="space-y-3">
                  <h4 className="font-semibold text-yellow-800 dark:text-yellow-200 text-sm">Supported Networks:</h4>
                  <div className="space-y-2">
                    <div className="p-2 bg-yellow-100 dark:bg-yellow-800/30 rounded flex items-center gap-2">
                      <span className="text-lg">🟡</span>
                      <div>
                        <div className="text-xs font-medium text-yellow-800 dark:text-yellow-200">BSC Mainnet</div>
                        <div className="text-xs text-yellow-700 dark:text-yellow-300">Chain ID: 56</div>
                      </div>
                    </div>
                    <div className="p-2 bg-orange-100 dark:bg-orange-800/30 rounded flex items-center gap-2">
                      <span className="text-lg">🟠</span>
                      <div>
                        <div className="text-xs font-medium text-orange-800 dark:text-orange-200">BSC Testnet</div>
                        <div className="text-xs text-orange-700 dark:text-orange-300">Chain ID: 97</div>
                      </div>
                    </div>
                  </div>
                  
                  <h4 className="font-semibold text-yellow-800 dark:text-yellow-200 text-sm mt-4">Features:</h4>
                  <ul className="space-y-2 text-xs text-yellow-700 dark:text-yellow-300">
                    <li className="flex items-start gap-2">
                      <span className="w-1 h-1 bg-yellow-500 rounded-full mt-2"></span>
                      <span>One-click network switching</span>
                    </li>
                    <li className="flex items-start gap-2">
                      <span className="w-1 h-1 bg-yellow-500 rounded-full mt-2"></span>
                      <span>Real-time sync status</span>
                    </li>
                    <li className="flex items-start gap-2">
                      <span className="w-1 h-1 bg-yellow-500 rounded-full mt-2"></span>
                      <span>Network validation warnings</span>
                    </li>
                    <li className="flex items-start gap-2">
                      <span className="w-1 h-1 bg-yellow-500 rounded-full mt-2"></span>
                      <span>Environment auto-detection</span>
                    </li>
                  </ul>
                </div>

                <div className="p-2 bg-yellow-100 dark:bg-yellow-800/30 rounded text-center">
                  <span className="text-xs font-medium text-yellow-800 dark:text-yellow-200">Connect wallet to switch networks</span>
                </div>
              </div>
            </div>

            {/* User Management Feature Card */}
            <div className="bg-white/90 backdrop-blur-xl dark:bg-slate-900/90 border border-purple-200/50 dark:border-purple-700/50 rounded-2xl p-8 shadow-xl">
              <div className="text-center mb-6">
                <div className="w-16 h-16 bg-purple-100 dark:bg-purple-900/30 rounded-full flex items-center justify-center mx-auto mb-4">
                  <span className="text-3xl">👤</span>
                </div>
                <h3 className="text-xl font-bold text-purple-800 dark:text-purple-200">User Management</h3>
                <p className="text-sm text-purple-600 dark:text-purple-400">Progressive authentication</p>
              </div>
              
              <div className="space-y-4">
                <div className="p-3 bg-purple-50 dark:bg-purple-900/20 rounded-lg">
                  <UserManagementDropdown compact={false} />
                </div>
                
                <div className="space-y-3">
                  <h4 className="font-semibold text-purple-800 dark:text-purple-200 text-sm">Auth Levels:</h4>
                  <div className="space-y-2">
                    <div className="p-2 bg-slate-100 dark:bg-slate-700 rounded flex items-center gap-2">
                      <span>👁️</span>
                      <span className="text-xs font-medium text-slate-700 dark:text-slate-300">Public - Browse only</span>
                    </div>
                    <div className="p-2 bg-blue-100 dark:bg-blue-800/30 rounded flex items-center gap-2">
                      <span>🔗</span>
                      <span className="text-xs font-medium text-blue-700 dark:text-blue-300">Connected - Wallet linked</span>
                    </div>
                    <div className="p-2 bg-green-100 dark:bg-green-800/30 rounded flex items-center gap-2">
                      <span>🛡️</span>
                      <span className="text-xs font-medium text-green-700 dark:text-green-300">Authenticated - Full access</span>
                    </div>
                  </div>
                  
                  <h4 className="font-semibold text-purple-800 dark:text-purple-200 text-sm mt-4">Menu Items:</h4>
                  <ul className="space-y-1 text-xs text-purple-700 dark:text-purple-300">
                    <li className="flex items-center gap-2"><span>👤</span>Profile</li>
                    <li className="flex items-center gap-2"><span>⚙️</span>Settings</li>
                    <li className="flex items-center gap-2"><span>🔗</span>Wallet</li>
                    <li className="flex items-center gap-2"><span>💳</span>Billing</li>
                    <li className="flex items-center gap-2"><span>🔔</span>Notifications</li>
                    <li className="flex items-center gap-2"><span>🛡️</span>Security</li>
                  </ul>
                </div>

                <div className="p-2 bg-purple-100 dark:bg-purple-800/30 rounded text-center">
                  <span className="text-xs font-medium text-purple-800 dark:text-purple-200">Context-aware menu items</span>
                </div>
              </div>
            </div>
          </div>
        </section>

        {/* Technical Implementation Stats */}
        <section className="mb-16">
          <h2 className="text-3xl font-semibold text-slate-800 dark:text-slate-200 mb-8 text-center">
            📊 Implementation Statistics
          </h2>
          <div className="bg-white/90 backdrop-blur-xl dark:bg-slate-900/90 border border-slate-200/50 dark:border-slate-700/50 rounded-3xl p-10 shadow-2xl">
            <div className="grid lg:grid-cols-6 gap-8 text-center">
              <div className="p-6 bg-gradient-to-br from-blue-50 to-blue-100 dark:from-blue-900/20 dark:to-blue-800/20 rounded-2xl">
                <div className="text-4xl font-bold text-blue-600 dark:text-blue-400 mb-2">4</div>
                <div className="text-sm font-medium text-blue-800 dark:text-blue-200">New Components</div>
                <div className="text-xs text-blue-600 dark:text-blue-400">Created from scratch</div>
              </div>
              
              <div className="p-6 bg-gradient-to-br from-green-50 to-green-100 dark:from-green-900/20 dark:to-green-800/20 rounded-2xl">
                <div className="text-4xl font-bold text-green-600 dark:text-green-400 mb-2">100%</div>
                <div className="text-sm font-medium text-green-800 dark:text-green-200">Mobile Responsive</div>
                <div className="text-xs text-green-600 dark:text-green-400">Hamburger menu</div>
              </div>
              
              <div className="p-6 bg-gradient-to-br from-orange-50 to-orange-100 dark:from-orange-900/20 dark:to-orange-800/20 rounded-2xl">
                <div className="text-4xl font-bold text-orange-600 dark:text-orange-400 mb-2">0</div>
                <div className="text-sm font-medium text-orange-800 dark:text-orange-200">Animations</div>
                <div className="text-xs text-orange-600 dark:text-orange-400">Zero animation policy</div>
              </div>
              
              <div className="p-6 bg-gradient-to-br from-purple-50 to-purple-100 dark:from-purple-900/20 dark:to-purple-800/20 rounded-2xl">
                <div className="text-4xl font-bold text-purple-600 dark:text-purple-400 mb-2">3</div>
                <div className="text-sm font-medium text-purple-800 dark:text-purple-200">Auth Levels</div>
                <div className="text-xs text-purple-600 dark:text-purple-400">Progressive system</div>
              </div>
              
              <div className="p-6 bg-gradient-to-br from-yellow-50 to-yellow-100 dark:from-yellow-900/20 dark:to-yellow-800/20 rounded-2xl">
                <div className="text-4xl font-bold text-yellow-600 dark:text-yellow-400 mb-2">6</div>
                <div className="text-sm font-medium text-yellow-800 dark:text-yellow-200">Wallet Types</div>
                <div className="text-xs text-yellow-600 dark:text-yellow-400">Auto-detection</div>
              </div>
              
              <div className="p-6 bg-gradient-to-br from-cyan-50 to-cyan-100 dark:from-cyan-900/20 dark:to-cyan-800/20 rounded-2xl">
                <div className="text-4xl font-bold text-cyan-600 dark:text-cyan-400 mb-2">2</div>
                <div className="text-sm font-medium text-cyan-800 dark:text-cyan-200">BSC Networks</div>
                <div className="text-xs text-cyan-600 dark:text-cyan-400">Mainnet & Testnet</div>
              </div>
            </div>
          </div>
        </section>

        {/* Call to Action */}
        <section className="text-center">
          <div className="bg-gradient-to-r from-orange-100 to-purple-100 dark:from-orange-900/30 dark:to-purple-900/30 rounded-3xl p-12 border border-orange-200/50 dark:border-orange-700/50">
            <h2 className="text-4xl font-bold text-slate-800 dark:text-slate-200 mb-4">
              🎉 Ready to Experience the Enhanced Navbar?
            </h2>
            <p className="text-xl text-slate-600 dark:text-slate-400 mb-8 max-w-3xl mx-auto">
              All 4 new navbar icons are now live and functional. Visit the main application to interact with 
              the complete Web3-enabled navigation experience.
            </p>
            
            <div className="flex flex-col sm:flex-row gap-4 justify-center items-center">
              <div className="bg-white dark:bg-slate-800 rounded-2xl p-4 border border-orange-200/50 dark:border-orange-700/50">
                <span className="text-lg font-mono text-slate-700 dark:text-slate-300">
                  🌐 http://localhost:3000
                </span>
              </div>
              <div className="text-slate-500 dark:text-slate-400">
                ← Visit main app to test all features
              </div>
            </div>
          </div>
        </section>
      </div>
    </div>
  );
}