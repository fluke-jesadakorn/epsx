'use client';

import React, { useState } from 'react';
import { 
  Shield, 
  Database, 
  Bell, 
  Key,
  Save,
  AlertTriangle,
  CheckCircle,
  Info,
  Trash2,
  RefreshCw
} from 'lucide-react';

interface IAMConfig {
  sessionTimeout: number;
  maxLoginAttempts: number;
  passwordPolicy: {
    minLength: number;
    requireUppercase: boolean;
    requireLowercase: boolean;
    requireNumbers: boolean;
    requireSpecialChars: boolean;
  };
  enableAuditLogging: boolean;
  enableNotifications: boolean;
  defaultPackageTier: string;
  autoRevokeExpiredPermissions: boolean;
}

export const IAMSettings: React.FC = () => {
  const [config, setConfig] = useState<IAMConfig>({
    sessionTimeout: 24,
    maxLoginAttempts: 5,
    passwordPolicy: {
      minLength: 8,
      requireUppercase: true,
      requireLowercase: true,
      requireNumbers: true,
      requireSpecialChars: false,
    },
    enableAuditLogging: true,
    enableNotifications: true,
    defaultPackageTier: 'free',
    autoRevokeExpiredPermissions: true,
  });
  
  const [activeSection, setActiveSection] = useState<'security' | 'permissions' | 'notifications' | 'database'>('security');
  const [unsavedChanges, setUnsavedChanges] = useState(false);
  const [saving, setSaving] = useState(false);
  const [lastSaved, setLastSaved] = useState<Date | null>(null);

  const updateConfig = (updates: Partial<IAMConfig>) => {
    setConfig(prev => ({ ...prev, ...updates }));
    setUnsavedChanges(true);
  };

  const updatePasswordPolicy = (updates: Partial<IAMConfig['passwordPolicy']>) => {
    setConfig(prev => ({
      ...prev,
      passwordPolicy: { ...prev.passwordPolicy, ...updates }
    }));
    setUnsavedChanges(true);
  };

  const handleSave = async () => {
    try {
      setSaving(true);
      // Simulate API call
      await new Promise(resolve => setTimeout(resolve, 1000));
      setUnsavedChanges(false);
      setLastSaved(new Date());
    } catch (error) {
      console.error('Failed to save settings:', error);
      alert('Failed to save settings');
    } finally {
      setSaving(false);
    }
  };

  const handleReset = () => {
    if (confirm('Are you sure you want to reset all settings to defaults?')) {
      setConfig({
        sessionTimeout: 24,
        maxLoginAttempts: 5,
        passwordPolicy: {
          minLength: 8,
          requireUppercase: true,
          requireLowercase: true,
          requireNumbers: true,
          requireSpecialChars: false,
        },
        enableAuditLogging: true,
        enableNotifications: true,
        defaultPackageTier: 'free',
        autoRevokeExpiredPermissions: true,
      });
      setUnsavedChanges(true);
    }
  };

  const sections = [
    {
      id: 'security' as const,
      name: 'Security',
      icon: Shield,
      description: 'Authentication and session settings'
    },
    {
      id: 'permissions' as const,
      name: 'Permissions',
      icon: Key,
      description: 'Default permissions and access control'
    },
    {
      id: 'notifications' as const,
      name: 'Notifications',
      icon: Bell,
      description: 'Alert and notification preferences'
    },
    {
      id: 'database' as const,
      name: 'Database',
      icon: Database,
      description: 'Data management and cleanup'
    }
  ];

  return (
    <div className="space-y-6">
      {/* Header */}
      <div className="flex flex-col sm:flex-row gap-4 justify-between">
        <div>
          <h3 className="text-lg font-semibold text-gray-900">IAM Settings</h3>
          <p className="text-sm text-gray-500 mt-1">
            Configure system-wide IAM policies and security settings
          </p>
        </div>
        
        <div className="flex gap-2">
          {unsavedChanges && (
            <div className="flex items-center text-sm text-orange-600 mr-4">
              <AlertTriangle className="h-4 w-4 mr-1" />
              Unsaved changes
            </div>
          )}
          {lastSaved && (
            <div className="flex items-center text-sm text-green-600 mr-4">
              <CheckCircle className="h-4 w-4 mr-1" />
              Saved {lastSaved.toLocaleTimeString()}
            </div>
          )}
          <button
            onClick={handleReset}
            className="inline-flex items-center px-4 py-2 border border-gray-300 rounded-lg text-sm font-medium text-gray-700 bg-white hover:bg-gray-50"
          >
            <RefreshCw className="h-4 w-4 mr-2" />
            Reset
          </button>
          <button
            onClick={handleSave}
            disabled={!unsavedChanges || saving}
            className="inline-flex items-center px-4 py-2 border border-transparent text-sm font-medium rounded-lg shadow-sm text-white bg-blue-600 hover:bg-blue-700 disabled:opacity-50"
          >
            <Save className="h-4 w-4 mr-2" />
            {saving ? 'Saving...' : 'Save Changes'}
          </button>
        </div>
      </div>

      <div className="grid grid-cols-1 lg:grid-cols-4 gap-6">
        {/* Sidebar Navigation */}
        <div className="lg:col-span-1">
          <nav className="space-y-1">
            {sections.map((section) => {
              const Icon = section.icon;
              return (
                <button
                  key={section.id}
                  onClick={() => setActiveSection(section.id)}
                  className={`w-full flex items-center px-3 py-2 text-sm font-medium rounded-lg transition-colors ${
                    activeSection === section.id
                      ? 'bg-blue-50 text-blue-700 border border-blue-200'
                      : 'text-gray-700 hover:bg-gray-50'
                  }`}
                >
                  <Icon className={`h-4 w-4 mr-3 ${
                    activeSection === section.id ? 'text-blue-600' : 'text-gray-400'
                  }`} />
                  <div className="text-left">
                    <div>{section.name}</div>
                    <div className="text-xs text-gray-500 hidden lg:block">
                      {section.description}
                    </div>
                  </div>
                </button>
              );
            })}
          </nav>
        </div>

        {/* Content Area */}
        <div className="lg:col-span-3">
          <div className="bg-white border border-gray-200 rounded-lg p-6">
            {activeSection === 'security' && (
              <div className="space-y-6">
                <div>
                  <h4 className="text-lg font-medium text-gray-900 mb-4">Security Settings</h4>
                  
                  <div className="grid grid-cols-1 md:grid-cols-2 gap-6">
                    <div>
                      <label className="block text-sm font-medium text-gray-700 mb-2">
                        Session Timeout (hours)
                      </label>
                      <input
                        type="number"
                        min="1"
                        max="72"
                        value={config.sessionTimeout}
                        onChange={(e) => updateConfig({ sessionTimeout: parseInt(e.target.value) })}
                        className="w-full border border-gray-300 rounded-lg px-3 py-2 focus:ring-blue-500 focus:border-blue-500"
                      />
                      <p className="text-xs text-gray-500 mt-1">
                        Users will be logged out after this period of inactivity
                      </p>
                    </div>
                    
                    <div>
                      <label className="block text-sm font-medium text-gray-700 mb-2">
                        Max Login Attempts
                      </label>
                      <input
                        type="number"
                        min="3"
                        max="10"
                        value={config.maxLoginAttempts}
                        onChange={(e) => updateConfig({ maxLoginAttempts: parseInt(e.target.value) })}
                        className="w-full border border-gray-300 rounded-lg px-3 py-2 focus:ring-blue-500 focus:border-blue-500"
                      />
                      <p className="text-xs text-gray-500 mt-1">
                        Account will be locked after this many failed attempts
                      </p>
                    </div>
                  </div>
                </div>

                <div>
                  <h5 className="text-md font-medium text-gray-900 mb-4">Password Policy</h5>
                  
                  <div className="space-y-4">
                    <div>
                      <label className="block text-sm font-medium text-gray-700 mb-2">
                        Minimum Length
                      </label>
                      <input
                        type="number"
                        min="6"
                        max="32"
                        value={config.passwordPolicy.minLength}
                        onChange={(e) => updatePasswordPolicy({ minLength: parseInt(e.target.value) })}
                        className="w-32 border border-gray-300 rounded-lg px-3 py-2 focus:ring-blue-500 focus:border-blue-500"
                      />
                    </div>
                    
                    <div className="space-y-3">
                      {[
                        { key: 'requireUppercase', label: 'Require uppercase letters' },
                        { key: 'requireLowercase', label: 'Require lowercase letters' },
                        { key: 'requireNumbers', label: 'Require numbers' },
                        { key: 'requireSpecialChars', label: 'Require special characters' },
                      ].map((item) => (
                        <label key={item.key} className="flex items-center">
                          <input
                            type="checkbox"
                            checked={config.passwordPolicy[item.key as keyof typeof config.passwordPolicy] as boolean}
                            onChange={(e) => updatePasswordPolicy({ [item.key]: e.target.checked })}
                            className="h-4 w-4 text-blue-600 border-gray-300 rounded focus:ring-blue-500"
                          />
                          <span className="ml-2 text-sm text-gray-700">{item.label}</span>
                        </label>
                      ))}
                    </div>
                  </div>
                </div>
              </div>
            )}

            {activeSection === 'permissions' && (
              <div className="space-y-6">
                <div>
                  <h4 className="text-lg font-medium text-gray-900 mb-4">Permission Settings</h4>
                  
                  <div className="space-y-4">
                    <div>
                      <label className="block text-sm font-medium text-gray-700 mb-2">
                        Default Package Tier for New Users
                      </label>
                      <select
                        value={config.defaultPackageTier}
                        onChange={(e) => updateConfig({ defaultPackageTier: e.target.value })}
                        className="w-full border border-gray-300 rounded-lg px-3 py-2 focus:ring-blue-500 focus:border-blue-500"
                      >
                        <option value="free">Free</option>
                        <option value="bronze">Bronze</option>
                        <option value="silver">Silver</option>
                        <option value="gold">Gold</option>
                        <option value="platinum">Platinum</option>
                        <option value="enterprise">Enterprise</option>
                      </select>
                    </div>
                    
                    <label className="flex items-center">
                      <input
                        type="checkbox"
                        checked={config.autoRevokeExpiredPermissions}
                        onChange={(e) => updateConfig({ autoRevokeExpiredPermissions: e.target.checked })}
                        className="h-4 w-4 text-blue-600 border-gray-300 rounded focus:ring-blue-500"
                      />
                      <span className="ml-2 text-sm text-gray-700">
                        Automatically revoke expired custom permissions
                      </span>
                    </label>
                  </div>
                </div>
              </div>
            )}

            {activeSection === 'notifications' && (
              <div className="space-y-6">
                <div>
                  <h4 className="text-lg font-medium text-gray-900 mb-4">Notification Settings</h4>
                  
                  <div className="space-y-4">
                    <label className="flex items-center">
                      <input
                        type="checkbox"
                        checked={config.enableAuditLogging}
                        onChange={(e) => updateConfig({ enableAuditLogging: e.target.checked })}
                        className="h-4 w-4 text-blue-600 border-gray-300 rounded focus:ring-blue-500"
                      />
                      <span className="ml-2 text-sm text-gray-700">
                        Enable audit logging
                      </span>
                    </label>
                    
                    <label className="flex items-center">
                      <input
                        type="checkbox"
                        checked={config.enableNotifications}
                        onChange={(e) => updateConfig({ enableNotifications: e.target.checked })}
                        className="h-4 w-4 text-blue-600 border-gray-300 rounded focus:ring-blue-500"
                      />
                      <span className="ml-2 text-sm text-gray-700">
                        Enable email notifications for admin actions
                      </span>
                    </label>
                  </div>
                </div>
              </div>
            )}

            {activeSection === 'database' && (
              <div className="space-y-6">
                <div>
                  <h4 className="text-lg font-medium text-gray-900 mb-4">Database Management</h4>
                  
                  <div className="space-y-4">
                    <div className="bg-yellow-50 border border-yellow-200 rounded-lg p-4">
                      <div className="flex">
                        <Info className="h-5 w-5 text-yellow-400 mr-2 mt-0.5" />
                        <div>
                          <h5 className="text-sm font-medium text-yellow-800">Database Operations</h5>
                          <p className="text-sm text-yellow-700 mt-1">
                            Database cleanup and maintenance operations are not yet implemented.
                          </p>
                        </div>
                      </div>
                    </div>
                    
                    <div className="space-y-3">
                      <button
                        disabled
                        className="w-full flex items-center justify-center px-4 py-2 border border-gray-300 rounded-lg text-sm font-medium text-gray-400 bg-gray-100 cursor-not-allowed"
                      >
                        <Trash2 className="h-4 w-4 mr-2" />
                        Clean up expired sessions (Coming Soon)
                      </button>
                      
                      <button
                        disabled
                        className="w-full flex items-center justify-center px-4 py-2 border border-gray-300 rounded-lg text-sm font-medium text-gray-400 bg-gray-100 cursor-not-allowed"
                      >
                        <RefreshCw className="h-4 w-4 mr-2" />
                        Archive old audit logs (Coming Soon)
                      </button>
                    </div>
                  </div>
                </div>
              </div>
            )}
          </div>
        </div>
      </div>
    </div>
  );
};
