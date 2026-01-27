'use client';

import {
  Bell,
  Clock,
  Globe,
  Loader2,
  Palette,
  RefreshCw,
  RotateCcw,
  Save,
  Settings,
  Shield,
  Zap
} from 'lucide-react';
import * as React from 'react';
import { toast } from 'sonner';

import { useSettings } from '@/components/providers/SettingsProvider';
import { PageTabs, type TabItem } from '@/components/shared/page-layout';
import { settingsApi } from '@/lib/api/settings-client';
import type { SystemSettings } from '@/types/settings';
import { DEFAULT_SETTINGS } from '@/types/settings';

interface SettingsDashboardProps {
  initialSystemConfig?: unknown;
  initialGeneralSettings?: unknown;
  initialNotificationSettings?: unknown;
  initialSecuritySettings?: unknown;
  initialFeatureFlags?: unknown;
  initialEnvironmentConfig?: unknown;
}

/**
 * Modernized Settings Dashboard with PancakeSwap aesthetic
 */
export const SettingsDashboard: React.FC<SettingsDashboardProps> = () => {
  const [activeView, setActiveView] = React.useState('general');
  const [settings, setSettings] = React.useState<SystemSettings>(DEFAULT_SETTINGS);
  const [loading, setLoading] = React.useState(true);
  const [saving, setSaving] = React.useState(false);
  const [resetting, setResetting] = React.useState(false);
  const [hasChanges, setHasChanges] = React.useState(false);
  const [originalSettings, setOriginalSettings] = React.useState<SystemSettings>(DEFAULT_SETTINGS);
  const { applySettings: applyGlobalSettings } = useSettings();

  // Load settings on mount
  React.useEffect(() => {
    const loadSettings = async () => {
      try {
        setLoading(true);
        const data = await settingsApi.getAll();
        setSettings(data);
        setOriginalSettings(data);
      } catch (error) {
        console.error('Failed to load settings:', error);
        toast.error('Failed to load settings. Using defaults.');
        setSettings(DEFAULT_SETTINGS);
        setOriginalSettings(DEFAULT_SETTINGS);
      } finally {
        setLoading(false);
      }
    };

    loadSettings();
  }, []);

  // Track changes
  React.useEffect(() => {
    setHasChanges(JSON.stringify(settings) !== JSON.stringify(originalSettings));
  }, [settings, originalSettings]);

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
      description: 'Session and access settings',
    },
    {
      id: 'appearance',
      label: 'Appearance',
      icon: Palette,
      description: 'Theme and display options',
    },
  ];

  const handleSettingChange = <T extends keyof SystemSettings>(
    category: T,
    key: keyof SystemSettings[T],
    value: SystemSettings[T][typeof key]
  ) => {
    setSettings(prev => ({
      ...prev,
      [category]: {
        ...prev[category],
        [key]: value
      }
    }));
  };

  const handleSaveSettings = async () => {
    try {
      setSaving(true);
      const response = await settingsApi.update(settings);

      if (response.success) {
        setOriginalSettings(settings);
        applyGlobalSettings(settings); // Apply theme and accent color in real-time
        toast.success(`Settings saved! (${response.updated_count} updated)`);
      } else {
        toast.error(response.message || 'Failed to save settings');
      }
    } catch (error) {
      console.error('Error saving settings:', error);
      toast.error('Failed to save settings. Please try again.');
    } finally {
      setSaving(false);
    }
  };

  const handleResetSettings = async () => {
    if (!confirm('Reset all settings to default values? This cannot be undone.')) {
      return;
    }

    try {
      setResetting(true);
      const defaultSettings = await settingsApi.reset();
      setSettings(defaultSettings);
      setOriginalSettings(defaultSettings);
      applyGlobalSettings(defaultSettings); // Apply theme and accent color in real-time
      toast.success('Settings reset to defaults!');
    } catch (error) {
      console.error('Error resetting settings:', error);
      toast.error('Failed to reset settings. Please try again.');
    } finally {
      setResetting(false);
    }
  };

  const tabs: TabItem[] = [
    { id: 'general', label: 'Nodes', prefix: '🌍', gradient: 'primary' },
    { id: 'notifications', label: 'Signals', prefix: '🔔', gradient: 'info' },
    { id: 'security', label: 'Vault', prefix: '🔒', gradient: 'warning' },
    { id: 'appearance', label: 'Optics', prefix: '🎨', gradient: 'purple' },
  ];

  const renderContent = () => {
    if (loading) {
      return (
        <div className="flex items-center justify-center py-40">
          <div className="relative">
            <div className="absolute inset-0 blur-2xl bg-cyan-500/20 rounded-full animate-pulse"></div>
            <Loader2 className="h-16 w-16 animate-spin text-[#1fc7d4] relative z-10" />
            <p className="mt-6 text-[10px] font-black uppercase tracking-[0.3em] text-cyan-400 text-center animate-pulse">
              Syncing Nexus...
            </p>
          </div>
        </div>
      );
    }

    switch (activeView) {
      case 'general':
        return (
          <div className="space-y-10 animate-in fade-in slide-in-from-bottom-4 duration-500">
            {/* System Node Section */}
            <div className="relative overflow-hidden rounded-[40px] bg-slate-900/40 backdrop-blur-2xl border border-white/5 shadow-2xl">
              <div className="bg-gradient-to-r from-cyan-500/10 to-transparent p-12">
                <div className="flex items-center gap-8">
                  <div className="w-16 h-16 rounded-[24px] bg-cyan-500 flex items-center justify-center shadow-lg shadow-cyan-500/20">
                    <Globe className="h-8 w-8 text-white" />
                  </div>
                  <div>
                    <h2 className="text-3xl font-black text-foreground uppercase tracking-tight mb-2">System Configuration</h2>
                    <p className="text-sm font-bold text-muted-foreground uppercase opacity-50 tracking-widest">Platform Core Parameters</p>
                  </div>
                </div>
              </div>

              <div className="px-12 pb-16 space-y-12">
                {/* System Name */}
                <div className="space-y-4">
                  <label className="text-[10px] font-black uppercase tracking-[0.2em] text-muted-foreground ml-2">
                    System Designation
                  </label>
                  <div className="relative group">
                    <input
                      type="text"
                      value={settings.general.systemName}
                      onChange={(e) => handleSettingChange('general', 'systemName', e.target.value)}
                      className="w-full h-16 bg-white/5 border border-white/10 rounded-2xl px-6 font-black text-lg transition-all focus:border-cyan-500/50 focus:bg-white/[0.08] outline-none"
                    />
                    <div className="absolute right-6 top-1/2 -translate-y-1/2 opacity-20 pointer-events-none">
                      <Zap className="w-6 h-6 text-cyan-400" />
                    </div>
                  </div>
                </div>

                {/* Admin Email */}
                <div className="space-y-4">
                  <label className="text-[10px] font-black uppercase tracking-[0.2em] text-muted-foreground ml-2">
                    Authority Email Channel
                  </label>
                  <div className="relative group">
                    <input
                      type="email"
                      value={settings.general.adminEmail}
                      onChange={(e) => handleSettingChange('general', 'adminEmail', e.target.value)}
                      className="w-full h-16 bg-white/5 border border-white/10 rounded-2xl px-6 font-black text-lg transition-all focus:border-cyan-500/50 focus:bg-white/[0.08] outline-none"
                    />
                    <div className="absolute right-6 top-1/2 -translate-y-1/2 opacity-20 pointer-events-none">
                      <RefreshCw className="w-6 h-6 text-cyan-400" />
                    </div>
                  </div>
                </div>

                {/* Maintenance Toggle */}
                <div className="flex items-center justify-between p-8 rounded-[32px] bg-red-500/5 border border-red-500/10 hover:bg-red-500/10 transition-all">
                  <div className="flex items-center gap-6">
                    <div className="w-12 h-12 rounded-2xl bg-red-500/20 flex items-center justify-center">
                      <Shield className="w-6 h-6 text-red-500" />
                    </div>
                    <div>
                      <div className="text-sm font-black text-foreground uppercase tracking-tight mb-1">Maintenance Lock</div>
                      <div className="text-[10px] font-bold text-muted-foreground uppercase opacity-50">Isolate Network from Public Operations</div>
                    </div>
                  </div>
                  <button
                    onClick={() => handleSettingChange('general', 'maintenanceMode', !settings.general.maintenanceMode)}
                    className={`relative w-24 h-12 rounded-full transition-all duration-300 ${settings.general.maintenanceMode ? 'bg-red-500' : 'bg-white/5'
                      }`}
                  >
                    <div className={`absolute top-2 left-2 w-8 h-8 rounded-full bg-white transition-transform duration-300 ${settings.general.maintenanceMode ? 'translate-x-[48px]' : 'translate-x-0'
                      }`} />
                  </button>
                </div>
              </div>
            </div>
          </div>
        );
      case 'notifications':
        return (
          <div className="space-y-10 animate-in fade-in slide-in-from-bottom-4 duration-500">
            <div className="relative overflow-hidden rounded-[40px] bg-slate-900/40 backdrop-blur-2xl border border-white/5 shadow-2xl">
              <div className="bg-gradient-to-r from-purple-500/10 to-transparent p-12">
                <div className="flex items-center gap-8">
                  <div className="w-16 h-16 rounded-[24px] bg-purple-500 flex items-center justify-center shadow-lg shadow-purple-500/20">
                    <Bell className="h-8 w-8 text-white" />
                  </div>
                  <div>
                    <h2 className="text-3xl font-black text-foreground uppercase tracking-tight mb-2">Signal Processing</h2>
                    <p className="text-sm font-bold text-muted-foreground uppercase opacity-50 tracking-widest">Network Alert Preferences</p>
                  </div>
                </div>
              </div>

              <div className="px-12 pb-16 grid grid-cols-1 md:grid-cols-2 gap-8">
                {Object.entries(settings.notifications).map(([key, value]) => (
                  <div
                    key={key}
                    className="flex items-center justify-between p-8 rounded-[32px] bg-white/5 border border-white/5 hover:bg-white/[0.08] transition-all"
                  >
                    <div className="flex items-center gap-6">
                      <div className="w-12 h-12 rounded-2xl bg-purple-500/20 flex items-center justify-center">
                        <Zap className="w-6 h-6 text-purple-400" />
                      </div>
                      <div>
                        <div className="text-sm font-black text-foreground uppercase tracking-tight mb-1">
                          {key.replace(/([A-Z])/g, ' $1').replace(/^./, str => str.toUpperCase())}
                        </div>
                        <div className="text-[10px] font-bold text-muted-foreground uppercase opacity-50">Active Broadcast Channel</div>
                      </div>
                    </div>
                    <button
                      onClick={() => handleSettingChange('notifications', key as keyof typeof settings.notifications, !value)}
                      className={`relative w-20 h-10 rounded-full transition-all duration-300 ${value ? 'bg-purple-500' : 'bg-white/5'
                        }`}
                    >
                      <div className={`absolute top-1.5 left-1.5 w-7 h-7 rounded-full bg-white transition-transform duration-300 ${value ? 'translate-x-[40px]' : 'translate-x-0'
                        }`} />
                    </button>
                  </div>
                ))}
              </div>
            </div>
          </div>
        );
      case 'security':
        return (
          <div className="space-y-10 animate-in fade-in slide-in-from-bottom-4 duration-500">
            <div className="relative overflow-hidden rounded-[40px] bg-slate-900/40 backdrop-blur-2xl border border-white/5 shadow-2xl">
              <div className="bg-gradient-to-r from-amber-500/10 to-transparent p-12">
                <div className="flex items-center gap-8">
                  <div className="w-16 h-16 rounded-[24px] bg-amber-500 flex items-center justify-center shadow-lg shadow-amber-500/20">
                    <Shield className="h-8 w-8 text-white" />
                  </div>
                  <div>
                    <h2 className="text-3xl font-black text-foreground uppercase tracking-tight mb-2">Vault Protocols</h2>
                    <p className="text-sm font-bold text-muted-foreground uppercase opacity-50 tracking-widest">Authentication and Access Controls</p>
                  </div>
                </div>
              </div>

              <div className="px-12 pb-16 space-y-12">
                <div className="space-y-4">
                  <label className="text-[10px] font-black uppercase tracking-[0.2em] text-muted-foreground ml-2">
                    Auto-Lock Duration (Minutes)
                  </label>
                  <div className="relative group max-w-md">
                    <input
                      type="number"
                      value={settings.security.sessionTimeout}
                      onChange={(e) => handleSettingChange('security', 'sessionTimeout', parseInt(e.target.value) || 30)}
                      className="w-full h-16 bg-white/5 border border-white/10 rounded-2xl px-6 font-black text-lg transition-all focus:border-amber-500/50 focus:bg-white/[0.08] outline-none"
                    />
                    <div className="absolute right-6 top-1/2 -translate-y-1/2 opacity-20 pointer-events-none">
                      <Clock className="w-6 h-6 text-amber-400" />
                    </div>
                  </div>
                  <p className="text-[10px] font-bold text-muted-foreground uppercase opacity-30 ml-2">Recommended: 15-60 Minutes for Optimal Security</p>
                </div>
              </div>
            </div>
          </div>
        );
      case 'appearance':
        return (
          <div className="space-y-10 animate-in fade-in slide-in-from-bottom-4 duration-500">
            <div className="relative overflow-hidden rounded-[40px] bg-slate-900/40 backdrop-blur-2xl border border-white/5 shadow-2xl">
              <div className="bg-gradient-to-r from-pink-500/10 to-transparent p-12">
                <div className="flex items-center gap-8">
                  <div className="w-16 h-16 rounded-[24px] bg-pink-500 flex items-center justify-center shadow-lg shadow-pink-500/20">
                    <Palette className="h-8 w-8 text-white" />
                  </div>
                  <div>
                    <h2 className="text-3xl font-black text-foreground uppercase tracking-tight mb-2">Optical Customization</h2>
                    <p className="text-sm font-bold text-muted-foreground uppercase opacity-50 tracking-widest">Visual Feedback and Interface Styling</p>
                  </div>
                </div>
              </div>

              <div className="px-12 pb-16 space-y-12">
                {/* Theme Mode */}
                <div className="space-y-4">
                  <label className="text-[10px] font-black uppercase tracking-[0.2em] text-muted-foreground ml-2">
                    Luminosity Mode
                  </label>
                  <div className="grid grid-cols-1 sm:grid-cols-3 gap-6">
                    {['light', 'dark', 'auto'].map((mode) => (
                      <button
                        key={mode}
                        onClick={() => handleSettingChange('appearance', 'theme', mode as any)}
                        className={`p-6 rounded-[24px] border transition-all text-center ${settings.appearance.theme === mode
                          ? 'bg-pink-500/10 border-pink-500 shadow-lg shadow-pink-500/10'
                          : 'bg-white/5 border-white/5 hover:border-white/10'
                          }`}
                      >
                        <div className="font-black text-sm uppercase tracking-widest">
                          {mode === 'light' ? '☀️ Daylight' : mode === 'dark' ? '🌙 Eclipse' : '🔄 Neural'}
                        </div>
                      </button>
                    ))}
                  </div>
                </div>

                {/* Accent Color */}
                <div className="space-y-4">
                  <label className="text-[10px] font-black uppercase tracking-[0.2em] text-muted-foreground ml-2">
                    Interface Accent Chroma
                  </label>
                  <div className="flex flex-col sm:flex-row items-center gap-10 p-8 rounded-[32px] bg-white/5 border border-white/5">
                    <div className="relative w-24 h-24 rounded-[32px] overflow-hidden group shadow-2xl">
                      <input
                        type="color"
                        value={settings.appearance.primaryColor}
                        onChange={(e) => handleSettingChange('appearance', 'primaryColor', e.target.value)}
                        className="absolute inset-0 w-[150%] h-[150%] -translate-x-1/4 -translate-y-1/4 cursor-pointer"
                      />
                      <div className="absolute inset-0 flex items-center justify-center pointer-events-none bg-black/40 opacity-0 group-hover:opacity-100 transition-opacity">
                        <RefreshCw className="w-6 h-6 text-white" />
                      </div>
                    </div>
                    <div className="flex-1 text-center sm:text-left">
                      <div className="text-xl font-black uppercase tracking-tight mb-1">{settings.appearance.primaryColor}</div>
                      <div className="text-[10px] font-bold text-muted-foreground uppercase opacity-50">Active Interface Pigment</div>
                    </div>

                    <div className="grid grid-cols-4 gap-3">
                      {[
                        { name: 'PancakeSwap', color: '#1fc7d4' },
                        { name: 'Eclipse', color: '#7645d9' },
                        { name: 'Magma', color: '#ffb237' },
                        { name: 'Crimson', color: '#ed4b9e' },
                      ].map((preset) => (
                        <button
                          key={preset.name}
                          onClick={() => handleSettingChange('appearance', 'primaryColor', preset.color)}
                          className="w-10 h-10 rounded-full border-2 border-white/10 hover:scale-110 transition-transform shadow-lg"
                          style={{ backgroundColor: preset.color }}
                          title={preset.name}
                        />
                      ))}
                    </div>
                  </div>
                </div>
              </div>
            </div>
          </div>
        );
      default:
        return null;
    }
  };

  return (
    <div className="space-y-10">
      {/* Global Control Bar */}
      <div className="flex flex-col sm:flex-row items-center justify-between gap-6 p-6 rounded-[32px] bg-slate-900/40 backdrop-blur-2xl border border-white/5 shadow-xl">
        <PageTabs
          tabs={tabs}
          activeTab={activeView}
          onTabChange={setActiveView}
          className="bg-transparent border-none p-0 backdrop-blur-none shadow-none"
        />

        <div className="flex items-center gap-4">
          <button
            onClick={handleResetSettings}
            disabled={resetting || loading}
            className="flex items-center gap-3 px-8 py-4 rounded-2xl bg-white/5 hover:bg-white/10 text-[10px] font-black uppercase tracking-widest transition-all disabled:opacity-50"
          >
            {resetting ? <Loader2 className="w-4 h-4 animate-spin" /> : <RotateCcw className="w-4 h-4" />}
            Reset Logic
          </button>

          <button
            onClick={handleSaveSettings}
            disabled={saving || loading || !hasChanges}
            className={`
              flex items-center gap-3 px-10 py-4 rounded-2xl text-[10px] font-black uppercase tracking-widest transition-all
              ${hasChanges
                ? 'bg-gradient-to-r from-orange-400 to-red-500 text-white shadow-lg shadow-orange-500/20 active:scale-95'
                : 'bg-white/5 text-muted-foreground cursor-not-allowed'}
            `}
          >
            {saving ? <Loader2 className="w-4 h-4 animate-spin" /> : <Save className="w-4 h-4" />}
            {hasChanges ? 'Deploy Update' : 'Synchronized'}
          </button>
        </div>
      </div>

      <div className="relative">
        {renderContent()}
      </div>
    </div>
  );
};
