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
  Shield,
  Zap
} from 'lucide-react';
import * as React from 'react';
import { toast } from 'sonner';

import { useSearchParams } from 'next/navigation';

import { useSettings } from '@/components/providers/settings-provider';
import { settingsApi } from '@/lib/api/settings-client';
import { logger } from '@/lib/logger';
import type { SystemSettings } from '@/types/settings';
import { DEFAULT_SETTINGS } from '@/types/settings';

interface SettingsDashboardProps {
  initialSystemConfig?: unknown;
  initialGeneralSettings?: unknown;
  initialNotificationSettings?: unknown;
  initialSecuritySettings?: unknown;
  initialEnvironmentConfig?: unknown;
}

interface SettingsContentProps {
  loading: boolean;
  activeView: string;
  settings: SystemSettings;
  handleSettingChange: <T extends keyof SystemSettings>(
    category: T,
    key: keyof SystemSettings[T],
    value: SystemSettings[T][typeof key]
  ) => void;
}

function GeneralSettings({ settings, handleSettingChange }: Pick<SettingsContentProps, 'settings' | 'handleSettingChange'>) {
  return (
    <div className="space-y-10 animate-in fade-in slide-in-from-bottom-4 duration-500">
      <div className="rounded-2xl bg-card border border-border/20 overflow-hidden shadow-xl">
        <div className="h-[3px] bg-gradient-to-r from-[#1fc7d4] to-[#7645d9]" />
        <div className="flex items-center gap-4 p-5 border-b border-border/20">
          <div className="p-3 bg-gradient-to-br from-[#1fc7d4]/10 to-[#7645d9]/10 rounded-[18px] text-[#1fc7d4] border border-[#1fc7d4]/20">
            <Globe className="w-5 h-5" />
          </div>
          <div>
            <h2 className="text-base font-bold text-foreground uppercase tracking-wide">System Configuration</h2>
            <p className="text-xs text-muted-foreground">Platform Core Parameters</p>
          </div>
        </div>

        <div className="p-6 space-y-6">
          <div className="space-y-4">
            <label className="text-[10px] font-black uppercase tracking-[0.2em] text-muted-foreground ml-2">
              System Designation
            </label>
            <div className="relative group">
              <input
                type="text"
                value={settings.general.systemName}
                onChange={(e) => handleSettingChange('general', 'systemName', e.target.value)}
                className="w-full h-16 bg-muted/30 border border-border/40 rounded-2xl px-6 font-black text-lg transition-all focus:border-cyan-500/50 focus:bg-muted/50 outline-none"
              />
              <div className="absolute right-6 top-1/2 -translate-y-1/2 opacity-20 pointer-events-none">
                <Zap className="w-6 h-6 text-cyan-400" />
              </div>
            </div>
          </div>

          <div className="space-y-4">
            <label className="text-[10px] font-black uppercase tracking-[0.2em] text-muted-foreground ml-2">
              Authority Email Channel
            </label>
            <div className="relative group">
              <input
                type="email"
                value={settings.general.adminEmail}
                onChange={(e) => handleSettingChange('general', 'adminEmail', e.target.value)}
                className="w-full h-16 bg-muted/30 border border-border/40 rounded-2xl px-6 font-black text-lg transition-all focus:border-cyan-500/50 focus:bg-muted/50 outline-none"
              />
              <div className="absolute right-6 top-1/2 -translate-y-1/2 opacity-20 pointer-events-none">
                <RefreshCw className="w-6 h-6 text-cyan-400" />
              </div>
            </div>
          </div>

          <div className="flex items-center justify-between p-4 rounded-xl bg-muted/30 border border-border/40 hover:bg-red-500/10 transition-all">
            <div className="flex items-center gap-6">
              <div className="w-12 h-12 rounded-xl bg-red-500/20 flex items-center justify-center">
                <Shield className="w-6 h-6 text-red-500" />
              </div>
              <div>
                <div className="text-sm font-black text-foreground uppercase tracking-tight mb-1">Maintenance Lock</div>
                <div className="text-[10px] font-bold text-muted-foreground uppercase opacity-50">Isolate Network from Public Operations</div>
              </div>
            </div>
            <button
              type="button"
              onClick={() => { handleSettingChange('general', 'maintenanceMode', !settings.general.maintenanceMode); }}
              className={`relative w-24 h-12 rounded-full transition-all duration-300 ${settings.general.maintenanceMode ? 'bg-red-500' : 'bg-muted/30'
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
}

function NotificationSettings({ settings, handleSettingChange }: Pick<SettingsContentProps, 'settings' | 'handleSettingChange'>) {
  return (
    <div className="space-y-10 animate-in fade-in slide-in-from-bottom-4 duration-500">
      <div className="rounded-2xl bg-card border border-border/20 overflow-hidden shadow-xl">
        <div className="h-[3px] bg-gradient-to-r from-[#7645d9] to-[#ed4b9e]" />
        <div className="flex items-center gap-4 p-5 border-b border-border/20">
          <div className="p-3 bg-gradient-to-br from-[#7645d9]/10 to-[#ed4b9e]/10 rounded-[18px] text-[#7645d9] border border-[#7645d9]/20">
            <Bell className="w-5 h-5" />
          </div>
          <div>
            <h2 className="text-base font-bold text-foreground uppercase tracking-wide">Signal Processing</h2>
            <p className="text-xs text-muted-foreground">Network Alert Preferences</p>
          </div>
        </div>

        <div className="p-6 grid grid-cols-1 md:grid-cols-2 gap-4">
          {(Object.entries(settings.notifications) as Array<[keyof typeof settings.notifications, boolean]>).map(([key, value]) => (
            <div
              key={key}
              className="flex items-center justify-between p-4 rounded-xl bg-muted/30 border border-border/40 hover:bg-muted/50 transition-all"
            >
              <div className="flex items-center gap-6">
                <div className="w-12 h-12 rounded-xl bg-purple-500/20 flex items-center justify-center">
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
                type="button"
                onClick={() => { handleSettingChange('notifications', key, !value); }}
                className={`relative w-20 h-10 rounded-full transition-all duration-300 ${value ? 'bg-[#7645d9]' : 'bg-muted/30'
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
}

function SecuritySettings({ settings, handleSettingChange }: Pick<SettingsContentProps, 'settings' | 'handleSettingChange'>) {
  return (
    <div className="space-y-10 animate-in fade-in slide-in-from-bottom-4 duration-500">
      <div className="rounded-2xl bg-card border border-border/20 overflow-hidden shadow-xl">
        <div className="h-[3px] bg-gradient-to-r from-[#ffb237] to-[#ed4b9e]" />
        <div className="flex items-center gap-4 p-5 border-b border-border/20">
          <div className="p-3 bg-gradient-to-br from-[#ffb237]/10 to-[#ed4b9e]/10 rounded-[18px] text-[#ffb237] border border-[#ffb237]/20">
            <Shield className="w-5 h-5" />
          </div>
          <div>
            <h2 className="text-base font-bold text-foreground uppercase tracking-wide">Vault Protocols</h2>
            <p className="text-xs text-muted-foreground">Authentication and Access Controls</p>
          </div>
        </div>

        <div className="p-6 space-y-6">
          <div className="space-y-4">
            <label className="text-[10px] font-black uppercase tracking-[0.2em] text-muted-foreground ml-2">
              Auto-Lock Duration (Minutes)
            </label>
            <div className="relative group max-w-md">
              <input
                type="number"
                value={settings.security.sessionTimeout}
                onChange={(e) => {
                  const val = parseInt(e.target.value);
                  handleSettingChange('security', 'sessionTimeout', isNaN(val) ? 30 : val);
                }}
                className="w-full h-16 bg-muted/30 border border-border/40 rounded-2xl px-6 font-black text-lg transition-all focus:border-amber-500/50 focus:bg-muted/50 outline-none"
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
}

const COLOR_PRESETS = [
  { name: 'PancakeSwap', color: '#1fc7d4' },
  { name: 'Eclipse', color: '#7645d9' },
  { name: 'Magma', color: '#ffb237' },
  { name: 'Crimson', color: '#ed4b9e' },
];

function AppearanceSettings({ settings, handleSettingChange }: Pick<SettingsContentProps, 'settings' | 'handleSettingChange'>) {
  return (
    <div className="space-y-10 animate-in fade-in slide-in-from-bottom-4 duration-500">
      <div className="rounded-2xl bg-card border border-border/20 overflow-hidden shadow-xl">
        <div className="h-[3px] bg-gradient-to-r from-[#ed4b9e] to-[#7645d9]" />
        <div className="flex items-center gap-4 p-5 border-b border-border/20">
          <div className="p-3 bg-gradient-to-br from-[#ed4b9e]/10 to-[#7645d9]/10 rounded-[18px] text-[#ed4b9e] border border-[#ed4b9e]/20">
            <Palette className="w-5 h-5" />
          </div>
          <div>
            <h2 className="text-base font-bold text-foreground uppercase tracking-wide">Optical Customization</h2>
            <p className="text-xs text-muted-foreground">Visual Feedback and Interface Styling</p>
          </div>
        </div>

        <div className="p-6 space-y-6">
          <div className="space-y-4">
            <label className="text-[10px] font-black uppercase tracking-[0.2em] text-muted-foreground ml-2">
              Luminosity Mode
            </label>
            <div className="grid grid-cols-1 sm:grid-cols-3 gap-6">
              {(['light', 'dark', 'auto'] as const).map((mode) => (
                <button
                  key={mode}
                  type="button"
                  onClick={() => { handleSettingChange('appearance', 'theme', mode); }}
                  className={`p-6 rounded-2xl border transition-all text-center ${settings.appearance.theme === mode
                    ? 'bg-primary/10 border-primary shadow-lg shadow-pink-500/10'
                    : 'bg-muted/30 border-border/40 hover:bg-muted/50'
                    }`}
                >
                  <div className="font-black text-sm uppercase tracking-widest">
                    {mode === 'light' ? '☀️ Daylight' : mode === 'dark' ? '🌙 Eclipse' : '🔄 Neural'}
                  </div>
                </button>
              ))}
            </div>
          </div>

          <div className="space-y-4">
            <label className="text-[10px] font-black uppercase tracking-[0.2em] text-muted-foreground ml-2">
              Interface Accent Chroma
            </label>
            <div className="flex flex-col sm:flex-row items-center gap-10 p-4 rounded-xl bg-muted/30 border border-border/40">
              <div className="relative w-24 h-24 rounded-2xl overflow-hidden group shadow-2xl">
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
                {COLOR_PRESETS.map((preset) => (
                  <button
                    key={preset.name}
                    type="button"
                    onClick={() => handleSettingChange('appearance', 'primaryColor', preset.color)}
                    className="w-10 h-10 rounded-full border-2 border-border/40 hover:scale-110 transition-transform shadow-lg"
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
}

const SettingsContent: React.FC<SettingsContentProps> = ({
  loading,
  activeView,
  settings,
  handleSettingChange
}) => {
  if (loading) {
    return (
      <div className="flex items-center justify-center py-16">
        <Loader2 className="h-8 w-8 animate-spin text-[#1fc7d4]" />
      </div>
    );
  }

  switch (activeView) {
    case 'general':
      return <GeneralSettings settings={settings} handleSettingChange={handleSettingChange} />;
    case 'notifications':
      return <NotificationSettings settings={settings} handleSettingChange={handleSettingChange} />;
    case 'security':
      return <SecuritySettings settings={settings} handleSettingChange={handleSettingChange} />;
    case 'appearance':
      return <AppearanceSettings settings={settings} handleSettingChange={handleSettingChange} />;
    default:
      return null;
  }
};

function useSettingsState() {
  const [settings, setSettings] = React.useState<SystemSettings>(DEFAULT_SETTINGS);
  const [loading, setLoading] = React.useState(true);
  const [saving, setSaving] = React.useState(false);
  const [resetting, setResetting] = React.useState(false);
  const [hasChanges, setHasChanges] = React.useState(false);
  const [originalSettings, setOriginalSettings] = React.useState<SystemSettings>(DEFAULT_SETTINGS);

  React.useEffect(() => {
    setHasChanges(JSON.stringify(settings) !== JSON.stringify(originalSettings));
  }, [settings, originalSettings]);

  return {
    settings, setSettings,
    loading, setLoading,
    saving, setSaving,
    resetting, setResetting,
    hasChanges,
    originalSettings, setOriginalSettings,
  };
}

/**
 * Modernized Settings Dashboard with PancakeSwap aesthetic
 */
export const SettingsDashboard: React.FC<SettingsDashboardProps> = () => {
  const searchParams = useSearchParams();
  const activeView = searchParams.get('tab') ?? 'general';
  const state = useSettingsState();
  const { applySettings: applyGlobalSettings } = useSettings();

  React.useEffect(() => {
    const loadSettings = async () => {
      try {
        state.setLoading(true);
        const data = await settingsApi.getAll();
        state.setSettings(data);
        state.setOriginalSettings(data);
      } catch (error) {
        logger.error('Failed to load settings:', error);
        toast.error('Failed to load settings. Using defaults.');
        state.setSettings(DEFAULT_SETTINGS);
        state.setOriginalSettings(DEFAULT_SETTINGS);
      } finally {
        state.setLoading(false);
      }
    };

    void loadSettings();
  // eslint-disable-next-line react-hooks/exhaustive-deps
  }, []);

  const handleSettingChange = <T extends keyof SystemSettings>(
    category: T,
    key: keyof SystemSettings[T],
    value: SystemSettings[T][typeof key]
  ) => {
    state.setSettings(prev => ({
      ...prev,
      [category]: {
        ...prev[category],
        [key]: value
      }
    }));
  };

  const handleSaveSettings = async () => {
    try {
      state.setSaving(true);
      const response = await settingsApi.update(state.settings);

      if (response.success === true) {
        state.setOriginalSettings(state.settings);
        applyGlobalSettings(state.settings);
        toast.success(`Settings saved! (${response.updated_count} updated)`);
      } else {
        toast.error(response.message);
      }
    } catch (error) {
      logger.error('Error saving settings:', error);
      toast.error('Failed to save settings. Please try again.');
    } finally {
      state.setSaving(false);
    }
  };

  const doReset = async () => {
    try {
      state.setResetting(true);
      const defaultSettings = await settingsApi.reset();
      state.setSettings(defaultSettings);
      state.setOriginalSettings(defaultSettings);
      applyGlobalSettings(defaultSettings);
      toast.success('Settings reset to defaults!');
    } catch (error) {
      logger.error('Error resetting settings:', error);
      toast.error('Failed to reset settings. Please try again.');
    } finally {
      state.setResetting(false);
    }
  };

  const handleResetSettings = () => {
    toast('Reset all settings to default values? This cannot be undone.', {
      action: {
        label: 'Reset',
        onClick: () => { void doReset(); }
      },
    });
  };

  return (
    <div className="space-y-10">
      {/* Global Control Bar */}
      <div className="flex items-center justify-end gap-4 p-4 rounded-xl bg-card border border-border/20 shadow-xl">
        <div className="flex items-center gap-4">
          <button
            type="button"
            onClick={handleResetSettings}
            disabled={state.resetting || state.loading}
            className="flex items-center gap-3 px-4 py-2 rounded-xl bg-muted/30 hover:bg-muted/50 border border-border/40 text-[10px] font-black uppercase tracking-widest transition-all disabled:opacity-50"
          >
            {state.resetting ? <Loader2 className="w-4 h-4 animate-spin" /> : <RotateCcw className="w-4 h-4" />}
            Reset Logic
          </button>

          <button
            type="button"
            onClick={() => { void handleSaveSettings(); }}
            disabled={state.saving || state.loading || !state.hasChanges}
            className={`
              flex items-center gap-3 px-4 py-2 rounded-xl text-[10px] font-black uppercase tracking-widest transition-all
              ${state.hasChanges
                ? 'bg-gradient-to-r from-[#7645d9] to-[#5a33b8] text-white shadow-lg active:scale-95'
                : 'bg-muted/30 border border-border/40 text-muted-foreground cursor-not-allowed'}
            `}
          >
            {state.saving ? <Loader2 className="w-4 h-4 animate-spin" /> : <Save className="w-4 h-4" />}
            {state.hasChanges ? 'Deploy Update' : 'Synchronized'}
          </button>
        </div>
      </div>

      <div className="relative">
        <SettingsContent
          loading={state.loading}
          activeView={activeView}
          settings={state.settings}
          handleSettingChange={handleSettingChange}
        />
      </div>
    </div>
  );
};
