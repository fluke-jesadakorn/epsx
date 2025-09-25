'use client';

import {
  Bell,
  ChevronRight,
  Globe,
  Palette,
  RotateCcw,
  Save,
  Settings,
  Shield,
  Sparkles,
  Zap,
} from 'lucide-react';
import * as React from 'react';
import { UnifiedAdminClient } from '@/lib/api/unified-admin-client';

interface SettingsDashboardProps {
  initialSystemConfig: any;
  initialGeneralSettings: any;
  initialNotificationSettings: any;
  initialSecuritySettings: any;
  initialFeatureFlags: any;
  initialEnvironmentConfig: any;
}

export const SettingsDashboard: React.FC<SettingsDashboardProps> = ({
  initialSystemConfig,
  initialGeneralSettings,
  initialNotificationSettings,
  initialSecuritySettings,
  initialFeatureFlags: _initialFeatureFlags,
  initialEnvironmentConfig: _initialEnvironmentConfig
}) => {
  const [activeView, setActiveView] = React.useState('general');
  const [settings, setSettings] = React.useState({
    general: initialGeneralSettings,
    notifications: initialNotificationSettings,
    security: initialSecuritySettings,
    system: initialSystemConfig
  });

  const settingsViews = [
    {
      id: 'general',
      label: 'General Settings',
      icon: Settings,
      description: 'Basic system configuration',
    },
    {
      id: 'notifications',
      label: 'Notifications',
      icon: Bell,
      description: 'Alert and notification preferences',
    },
    {
      id: 'security',
      label: 'Security',
      icon: Shield,
      description: 'Authentication and security settings',
    },
    {
      id: 'appearance',
      label: 'Appearance',
      icon: Palette,
      description: 'Theme and display options',
    },
  ];

  const handleSettingChange = (category: string, key: string, value: any) => {
    setSettings(prev => ({
      ...prev,
      [category]: {
        ...(prev as any)[category],
        [key]: value
      }
    }));
  };

  const handleSaveSettings = async () => {
    try {
      for (const [category, categorySettings] of Object.entries(settings)) {
        // await AdminApiService.updateSettings({
        //   category,
        //   settings: categorySettings,
        //   updatedBy: 'current-admin-id'
        // });
      }
      alert('Settings saved successfully!');
    } catch (error) {
      console.error('Error saving settings:', error);
      alert('Failed to save settings');
    }
  };

  const renderContent = () => {
    switch (activeView) {
      case 'general':
        return (
          <div>
            <div className="bg-white/80 dark:bg-gray-800/80 backdrop-blur-sm rounded-3xl shadow-2xl border border-yellow-200/50 dark:border-purple-500/30 overflow-hidden">
              {/* Card Header */}
              <div className="bg-gradient-to-r from-yellow-400 via-orange-500 to-pink-500 p-6">
                <div className="flex items-center gap-4">
                  <div className="h-12 w-12 bg-white/20 rounded-2xl flex items-center justify-center backdrop-blur-sm">
                    <Globe className="h-6 w-6 text-white" />
                  </div>
                  <div>
                    <h2 className="text-2xl font-bold text-white">🌍 System Configuration</h2>
                    <p className="text-white/80">Configure your platform basics</p>
                  </div>
                </div>
              </div>
              
              {/* Card Content */}
              <div className="p-8 space-y-6">
                <div>
                  <label className="flex items-center gap-2 text-lg font-semibold text-gray-900 dark:text-white mb-3">
                    <Sparkles className="h-5 w-5 text-yellow-500" />
                    System Name
                  </label>
                  <input
                    type="text"
                    value={settings.general?.systemName || settings.system?.name || 'EPSX Admin Console'}
                    onChange={(e) => handleSettingChange('general', 'systemName', e.target.value)}
                    className="w-full px-4 py-3 border-2 border-yellow-200 dark:border-purple-500/30 rounded-2xl bg-white/50 dark:bg-gray-700/50 backdrop-blur-sm text-gray-900 dark:text-white focus:ring-4 focus:ring-orange-200 focus:border-orange-400  text-lg"
                    placeholder="Enter your system name..."
                  />
                </div>
                
                <div>
                  <label className="flex items-center gap-2 text-lg font-semibold text-gray-900 dark:text-white mb-3">
                    <Zap className="h-5 w-5 text-orange-500" />
                    Admin Email
                  </label>
                  <input
                    type="email"
                    value={settings.general?.adminEmail || settings.system?.adminEmail || 'admin@epsx.com'}
                    onChange={(e) => handleSettingChange('general', 'adminEmail', e.target.value)}
                    className="w-full px-4 py-3 border-2 border-yellow-200 dark:border-purple-500/30 rounded-2xl bg-white/50 dark:bg-gray-700/50 backdrop-blur-sm text-gray-900 dark:text-white focus:ring-4 focus:ring-orange-200 focus:border-orange-400  text-lg"
                    placeholder="Enter admin email address..."
                  />
                </div>
                
                <div className="flex items-center justify-between p-6 bg-gradient-to-r from-yellow-50/50 to-orange-50/50 dark:from-purple-900/20 dark:to-gray-900/20 rounded-2xl border border-yellow-200/30 dark:border-purple-500/20">
                  <div>
                    <div className="flex items-center gap-2 font-bold text-lg text-gray-900 dark:text-white mb-2">
                      <Shield className="h-5 w-5 text-red-500" />
                      🚧 Maintenance Mode
                    </div>
                    <div className="text-gray-600 dark:text-gray-400">
                      Temporarily disable public access to the platform
                    </div>
                  </div>
                  <label className="relative inline-flex items-center cursor-pointer">
                    <input 
                      type="checkbox" 
                      className="sr-only peer"
                      checked={settings.general?.maintenanceMode || settings.system?.maintenanceMode || false}
                      onChange={(e) => handleSettingChange('general', 'maintenanceMode', e.target.checked)}
                    />
                    <div className="relative w-14 h-7 bg-gray-200 peer-focus:outline-none peer-focus:ring-4 peer-focus:ring-orange-300 dark:peer-focus:ring-orange-800 rounded-full peer dark:bg-gray-700 peer-checked:after:translate-x-full rtl:peer-checked:after:-translate-x-full peer-checked:after:border-white after:content-[''] after:absolute after:top-0.5 after:start-[4px] after:bg-white after:border-gray-300 after:border after:rounded-full after:h-6 after:w-6  dark:border-gray-600 peer-checked:bg-gradient-to-r peer-checked:from-yellow-400 peer-checked:to-orange-500"></div>
                  </label>
                </div>
              </div>
            </div>
          </div>
        );
      case 'notifications':
        return (
          <div>
            <div className="bg-white/80 dark:bg-gray-800/80 backdrop-blur-sm rounded-3xl shadow-2xl border border-yellow-200/50 dark:border-purple-500/30 overflow-hidden">
              {/* Card Header */}
              <div className="bg-gradient-to-r from-blue-400 via-purple-500 to-pink-500 p-6">
                <div className="flex items-center gap-4">
                  <div className="h-12 w-12 bg-white/20 rounded-2xl flex items-center justify-center backdrop-blur-sm">
                    <Bell className="h-6 w-6 text-white" />
                  </div>
                  <div>
                    <h2 className="text-2xl font-bold text-white">🔔 Notification Settings</h2>
                    <p className="text-white/80">Manage your alert preferences</p>
                  </div>
                </div>
              </div>
              
              {/* Card Content */}
              <div className="p-8">
                <div className="grid gap-6">
                  {Object.entries(settings.notifications || { emailNotifications: true, pushNotifications: false, smsNotifications: true, securityAlerts: true }).map(([key, value]) => (
                    <div key={key} className="flex items-center justify-between p-6 bg-gradient-to-r from-yellow-50/50 to-orange-50/50 dark:from-purple-900/20 dark:to-gray-900/20 rounded-2xl border border-yellow-200/30 dark:border-purple-500/20 hover:shadow-lg ">
                      <div className="flex items-center gap-4">
                        <div className="h-10 w-10 bg-gradient-to-r from-yellow-400 to-orange-500 rounded-2xl flex items-center justify-center">
                          <Bell className="h-5 w-5 text-white" />
                        </div>
                        <div>
                          <div className="font-bold text-lg text-gray-900 dark:text-white capitalize">
                            {key.replace(/([A-Z])/g, ' $1').replace(/^./, str => str.toUpperCase())}
                          </div>
                          <div className="text-gray-600 dark:text-gray-400">
                            Enable {key.toLowerCase().replace(/([A-Z])/g, ' $1')} for this platform
                          </div>
                        </div>
                      </div>
                      <label className="relative inline-flex items-center cursor-pointer">
                        <input 
                          type="checkbox" 
                          className="sr-only peer"
                          checked={Boolean(value)}
                          onChange={(e) => handleSettingChange('notifications', key, e.target.checked)}
                        />
                        <div className="relative w-14 h-7 bg-gray-200 peer-focus:outline-none peer-focus:ring-4 peer-focus:ring-purple-300 dark:peer-focus:ring-purple-800 rounded-full peer dark:bg-gray-700 peer-checked:after:translate-x-full rtl:peer-checked:after:-translate-x-full peer-checked:after:border-white after:content-[''] after:absolute after:top-0.5 after:start-[4px] after:bg-white after:border-gray-300 after:border after:rounded-full after:h-6 after:w-6  dark:border-gray-600 peer-checked:bg-gradient-to-r peer-checked:from-blue-400 peer-checked:to-purple-500"></div>
                      </label>
                    </div>
                  ))}
                </div>
              </div>
            </div>
          </div>
        );
      case 'security':
        return (
          <div>
            <div className="bg-white/80 dark:bg-gray-800/80 backdrop-blur-sm rounded-3xl shadow-2xl border border-yellow-200/50 dark:border-purple-500/30 overflow-hidden">
              {/* Card Header */}
              <div className="bg-gradient-to-r from-red-400 via-pink-500 to-purple-600 p-6">
                <div className="flex items-center gap-4">
                  <div className="h-12 w-12 bg-white/20 rounded-2xl flex items-center justify-center backdrop-blur-sm">
                    <Shield className="h-6 w-6 text-white" />
                  </div>
                  <div>
                    <h2 className="text-2xl font-bold text-white">🔒 Security Settings</h2>
                    <p className="text-white/80">Protect your admin access</p>
                  </div>
                </div>
              </div>
              
              {/* Card Content */}
              <div className="p-8 space-y-6">
                <div>
                  <label className="flex items-center gap-2 text-lg font-semibold text-gray-900 dark:text-white mb-3">
                    <Zap className="h-5 w-5 text-red-500" />
                    ⏱️ Session Timeout (minutes)
                  </label>
                  <input
                    type="number"
                    value={settings.security?.sessionTimeout || 30}
                    onChange={(e) => handleSettingChange('security', 'sessionTimeout', parseInt(e.target.value))}
                    className="w-full px-4 py-3 border-2 border-yellow-200 dark:border-purple-500/30 rounded-2xl bg-white/50 dark:bg-gray-700/50 backdrop-blur-sm text-gray-900 dark:text-white focus:ring-4 focus:ring-red-200 focus:border-red-400  text-lg"
                    placeholder="Enter session timeout in minutes..."
                    min="5"
                    max="480"
                  />
                </div>
                
                <div className="flex items-center justify-between p-6 bg-gradient-to-r from-red-50/50 to-pink-50/50 dark:from-red-900/20 dark:to-purple-900/20 rounded-2xl border border-red-200/30 dark:border-purple-500/20">
                  <div className="flex items-center gap-4">
                    <div className="h-10 w-10 bg-gradient-to-r from-red-400 to-pink-500 rounded-2xl flex items-center justify-center">
                      <Shield className="h-5 w-5 text-white" />
                    </div>
                    <div>
                      <div className="font-bold text-lg text-gray-900 dark:text-white">
                        🛡️ Two-Factor Authentication
                      </div>
                      <div className="text-gray-600 dark:text-gray-400">
                        Require 2FA for all admin access (Highly Recommended)
                      </div>
                    </div>
                  </div>
                  <label className="relative inline-flex items-center cursor-pointer">
                    <input 
                      type="checkbox" 
                      className="sr-only peer"
                      checked={settings.security?.twoFactorAuth || false}
                      onChange={(e) => handleSettingChange('security', 'twoFactorAuth', e.target.checked)}
                    />
                    <div className="relative w-14 h-7 bg-gray-200 peer-focus:outline-none peer-focus:ring-4 peer-focus:ring-red-300 dark:peer-focus:ring-red-800 rounded-full peer dark:bg-gray-700 peer-checked:after:translate-x-full rtl:peer-checked:after:-translate-x-full peer-checked:after:border-white after:content-[''] after:absolute after:top-0.5 after:start-[4px] after:bg-white after:border-gray-300 after:border after:rounded-full after:h-6 after:w-6  dark:border-gray-600 peer-checked:bg-gradient-to-r peer-checked:from-red-400 peer-checked:to-pink-500"></div>
                  </label>
                </div>
              </div>
            </div>
          </div>
        );
      case 'appearance':
        return (
          <div>
            <div className="bg-white/80 dark:bg-gray-800/80 backdrop-blur-sm rounded-3xl shadow-2xl border border-yellow-200/50 dark:border-purple-500/30 overflow-hidden">
              {/* Card Header */}
              <div className="bg-gradient-to-r from-purple-400 via-pink-500 to-red-500 p-6">
                <div className="flex items-center gap-4">
                  <div className="h-12 w-12 bg-white/20 rounded-2xl flex items-center justify-center backdrop-blur-sm">
                    <Palette className="h-6 w-6 text-white" />
                  </div>
                  <div>
                    <h2 className="text-2xl font-bold text-white">🎨 Appearance Settings</h2>
                    <p className="text-white/80">Customize your visual experience</p>
                  </div>
                </div>
              </div>
              
              {/* Card Content */}
              <div className="p-8 space-y-6">
                <div>
                  <label className="flex items-center gap-2 text-lg font-semibold text-gray-900 dark:text-white mb-3">
                    <Sparkles className="h-5 w-5 text-purple-500" />
                    🌓 Theme Mode
                  </label>
                  <select
                    value={settings.general?.theme || 'light'}
                    onChange={(e) => handleSettingChange('general', 'theme', e.target.value)}
                    className="w-full px-4 py-3 border-2 border-yellow-200 dark:border-purple-500/30 rounded-2xl bg-white/50 dark:bg-gray-700/50 backdrop-blur-sm text-gray-900 dark:text-white focus:ring-4 focus:ring-purple-200 focus:border-purple-400  text-lg"
                  >
                    <option value="light">☀️ Light Mode</option>
                    <option value="dark">🌙 Dark Mode</option>
                    <option value="auto">🔄 Auto (System)</option>
                  </select>
                </div>
                
                <div>
                  <label className="flex items-center gap-2 text-lg font-semibold text-gray-900 dark:text-white mb-3">
                    <Palette className="h-5 w-5 text-pink-500" />
                    🌈 Primary Accent Color
                  </label>
                  <div className="flex items-center gap-4">
                    <input
                      type="color"
                      value={settings.general?.primaryColor || '#FF8C00'}
                      onChange={(e) => handleSettingChange('general', 'primaryColor', e.target.value)}
                      className="h-16 w-32 border-2 border-yellow-200 dark:border-purple-500/30 rounded-2xl bg-white/50 dark:bg-gray-700/50 backdrop-blur-sm focus:ring-4 focus:ring-purple-200 focus:border-purple-400  cursor-pointer"
                    />
                    <div className="flex-1">
                      <div className="text-lg font-semibold text-gray-900 dark:text-white">
                        Selected Color: {settings.general?.primaryColor || '#FF8C00'}
                      </div>
                      <div className="text-gray-600 dark:text-gray-400">
                        This will affect buttons, links, and accent elements
                      </div>
                    </div>
                  </div>
                </div>
                
                {/* Color Presets */}
                <div>
                  <label className="flex items-center gap-2 text-lg font-semibold text-gray-900 dark:text-white mb-4">
                    <Zap className="h-5 w-5 text-yellow-500" />
                    🎭 Quick Presets
                  </label>
                  <div className="grid grid-cols-4 gap-4">
                    {[
                      { name: 'PancakeSwap', color: '#FF8C00', gradient: 'from-yellow-400 to-orange-500' },
                      { name: 'Ocean Blue', color: '#0EA5E9', gradient: 'from-blue-400 to-blue-600' },
                      { name: 'Forest', color: '#10B981', gradient: 'from-green-400 to-green-600' },
                      { name: 'Sunset', color: '#F59E0B', gradient: 'from-orange-400 to-red-500' },
                    ].map((preset) => (
                      <button
                        key={preset.name}
                        onClick={() => handleSettingChange('general', 'primaryColor', preset.color)}
                        className={`p-4 rounded-2xl bg-gradient-to-r ${preset.gradient} text-white font-semibold   text-center shadow-lg`}
                      >
                        <div className="text-sm">{preset.name}</div>
                        <div className="text-xs opacity-80">{preset.color}</div>
                      </button>
                    ))}
                  </div>
                </div>
              </div>
            </div>
          </div>
        );
      default:
        return (
          <div>
            <div className="bg-white/80 dark:bg-gray-800/80 backdrop-blur-sm rounded-3xl shadow-2xl border border-yellow-200/50 dark:border-purple-500/30 p-12 text-center">
              <div className="h-16 w-16 bg-gradient-to-r from-yellow-400 to-orange-500 rounded-3xl flex items-center justify-center mx-auto mb-6 ">
                <Settings className="h-8 w-8 text-white" />
              </div>
              <h3 className="text-2xl font-bold bg-gradient-to-r from-yellow-600 to-orange-600 bg-clip-text text-transparent mb-4">
                {settingsViews.find((v) => v.id === activeView)?.label}
              </h3>
              <p className="text-xl text-gray-600 dark:text-gray-400">
                🔄 Loading sweet settings data...
              </p>
            </div>
          </div>
        );
    }
  };

  return (
    <div className="min-h-screen bg-gradient-to-br from-yellow-50 via-orange-50 to-pink-50 dark:from-gray-900 dark:via-purple-900 dark:to-gray-900 p-6">
      {/* Background Decorations */}
      <div className="fixed inset-0 overflow-hidden pointer-events-none">
        <div className="absolute top-20 left-20 w-32 h-32 bg-gradient-to-r from-yellow-400/20 to-orange-500/20 rounded-full blur-xl "></div>
        <div className="absolute top-40 right-32 w-24 h-24 bg-gradient-to-r from-pink-400/20 to-purple-500/20 rounded-full blur-xl  delay-1000"></div>
        <div className="absolute bottom-32 left-40 w-40 h-40 bg-gradient-to-r from-blue-400/20 to-teal-500/20 rounded-full blur-xl  delay-2000"></div>
      </div>
      
      <div className="relative z-10 max-w-7xl mx-auto space-y-8">
        {/* Hero Header */}
        <div className="text-center mb-12">
          <div className="flex items-center justify-center gap-4 mb-6">
            <div className="h-16 w-16 bg-gradient-to-r from-yellow-400 to-orange-500 rounded-3xl flex items-center justify-center shadow-2xl ">
              <Settings className="h-8 w-8 text-white" />
            </div>
            <div className="text-left">
              <h1 className="text-4xl font-bold bg-gradient-to-r from-yellow-600 via-orange-600 to-pink-600 bg-clip-text text-transparent">
                ⚙️ System Settings
              </h1>
              <p className="text-xl text-gray-600 dark:text-gray-400 mt-2">
                Configure your sweet admin experience
              </p>
            </div>
          </div>
          
          {/* Action Buttons */}
          <div className="flex items-center justify-center gap-4">
            <button className="flex items-center gap-2 px-6 py-3 bg-white/70 dark:bg-gray-800/70 backdrop-blur-sm border-2 border-yellow-200/50 dark:border-purple-500/30 rounded-2xl hover:bg-white/90 dark:hover:bg-gray-800/90   shadow-lg">
              <RotateCcw className="h-5 w-5 text-gray-700 dark:text-gray-300" />
              <span className="font-semibold text-gray-700 dark:text-gray-300">Reset All</span>
            </button>
            <button 
              onClick={handleSaveSettings}
              className="flex items-center gap-2 px-8 py-3 bg-gradient-to-r from-yellow-400 via-orange-500 to-pink-500 text-white rounded-2xl font-semibold hover:from-yellow-500 hover:to-pink-600   shadow-xl"
            >
              <Save className="h-5 w-5" />
              <span>💾 Save Changes</span>
            </button>
          </div>
        </div>

        {/* Settings Navigation Cards */}
        <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-4 gap-6 mb-12">
          {settingsViews.map((view, index) => {
            const Icon = view.icon;
            const isActive = activeView === view.id;

            return (
              <button
                key={view.id}
                onClick={() => setActiveView(view.id)}
                className={`
                  group p-6 rounded-3xl text-left  border-2 relative overflow-hidden
                  ${
                    isActive
                      ? 'bg-gradient-to-br from-yellow-400 via-orange-500 to-pink-500 text-white border-orange-300 shadow-2xl scale-105'
                      : 'bg-white/70 dark:bg-gray-800/70 backdrop-blur-sm text-gray-700 dark:text-gray-300 hover:bg-white/90 dark:hover:bg-gray-800/90 border-yellow-200/50 dark:border-purple-500/30  shadow-xl'
                  }
                `}
                style={{
                  animationDelay: `${index * 100}ms`,
                }}
              >
                {/* Floating decorations */}
                <div className="absolute -top-2 -right-2 h-12 w-12 bg-white/10 rounded-full "></div>
                <div className="absolute top-1/2 -left-4 h-8 w-8 bg-yellow-400/10 rounded-full "></div>
                
                <div className="relative z-10">
                  <div className="flex items-center gap-4 mb-4">
                    <div
                      className={`
                      p-3 rounded-2xl  shadow-lg
                      ${
                        isActive
                          ? 'bg-white/20 backdrop-blur-sm'
                          : 'bg-gradient-to-r from-yellow-400 to-orange-500 group-hover:from-yellow-500 group-hover:to-orange-600'
                      }
                    `}
                    >
                      <Icon
                        className={`h-6 w-6  ${isActive ? 'text-white' : 'text-white'}`}
                      />
                    </div>
                    <ChevronRight
                      className={`h-5 w-5  ${isActive ? 'text-white rotate-90' : 'text-orange-500 group-hover:text-orange-600'}`}
                    />
                  </div>
                  <div>
                    <div
                      className={`font-bold text-lg mb-2 ${isActive ? 'text-white' : 'text-gray-900 dark:text-white'}`}
                    >
                      {view.label}
                    </div>
                    <div
                      className={`text-sm ${isActive ? 'text-white/80' : 'text-gray-600 dark:text-gray-400'}`}
                    >
                      {view.description}
                    </div>
                  </div>
                </div>
              </button>
            );
          })}
        </div>

        {/* Content */}
        {renderContent()}
      </div>
    </div>
  );
};
