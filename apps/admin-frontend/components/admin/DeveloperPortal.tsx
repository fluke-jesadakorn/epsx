'use client';

import React, { useState, useEffect } from 'react';
import { Button } from '@/components/ui/button';
import { FormField, Input, Select, Textarea } from '@/components/ui/form-components';
import { AdminService } from '@/services/adminService';
import { useModuleAuth } from '@/auth/module-ctx';
import { toast } from 'react-hot-toast';
import { 
  Key, 
  Plus, 
  Eye, 
  EyeOff, 
  Copy, 
  Trash2, 
  AlertTriangle, 
  ExternalLink as _ExternalLink,
  Code,
  BookOpen,
  Activity,
  Settings,
  Shield,
  Clock,
  Globe,
  BarChart3,
  Download
} from 'lucide-react';

// Types for API key management
interface ApiKey {
  id: string;
  key_prefix: string;
  client_name: string;
  client_description?: string;
  status: 'active' | 'revoked' | 'expired';
  total_requests: number;
  created_at: string;
  created_by: string;
  expires_at?: string;
  allowed_modules: ApiKeyModuleConfig[];
  ip_restrictions: string[];
  rate_limits: Record<string, number>;
  last_used_at?: string;
}

interface ApiKeyModuleConfig {
  module_id: string;
  module_name: string;
  access_level: string;
  custom_quotas?: Record<string, any>;
}

interface CreateApiKeyRequest {
  client_name: string;
  client_description?: string;
  client_contact_email?: string;
  allowed_modules: Array<{
    module_id: string;
    access_level: string;
    custom_quotas?: Record<string, any>;
  }>;
  ip_restrictions: string[];
  expires_at?: string;
}

interface Module {
  id: string;
  name: string;
  display_name: string;
  description?: string;
  category: string;
  status: string;
  access_levels: Record<string, any>;
  default_quotas: Record<string, any>;
}

const ACCESS_LEVELS = [
  { value: 'bronze', label: 'Bronze - Basic access', color: 'text-amber-600', description: 'Limited API calls, basic features' },
  { value: 'silver', label: 'Silver - Enhanced features', color: 'text-gray-500', description: 'Moderate API calls, real-time data' },
  { value: 'gold', label: 'Gold - Advanced tools', color: 'text-yellow-500', description: 'High API calls, advanced analytics' },
  { value: 'platinum', label: 'Platinum - Premium access', color: 'text-purple-600', description: 'Very high API calls, premium features' },
  { value: 'enterprise', label: 'Enterprise - Full access', color: 'text-blue-600', description: 'Unlimited access, all features' },
];

export const DeveloperPortal: React.FC = () => {
  const { hasModuleAccess, canPerformAction } = useModuleAuth();
  const [activeTab, setActiveTab] = useState<'overview' | 'keys' | 'docs' | 'usage'>('overview');
  const [apiKeys, setApiKeys] = useState<ApiKey[]>([]);
  const [modules, setModules] = useState<Module[]>([]);
  const [loading, setLoading] = useState(true);
  const [showCreateDialog, setShowCreateDialog] = useState(false);
  const [showKeyValue, setShowKeyValue] = useState<string | null>(null);
  const [newApiKey, setNewApiKey] = useState<string | null>(null);
  const [createForm, setCreateForm] = useState<CreateApiKeyRequest>({
    client_name: '',
    client_description: '',
    client_contact_email: '',
    allowed_modules: [],
    ip_restrictions: [],
  });

  // Load initial data
  useEffect(() => {
    loadData();
  }, []);

  const loadData = async () => {
    try {
      setLoading(true);
      const [keysRes, modulesRes] = await Promise.all([
        AdminService.listApiKeys(),
        AdminService.getModules({ status: 'active' })
      ]);
      
      if (keysRes.success) {
        setApiKeys(keysRes.data.api_keys || []);
      }
      
      if (modulesRes.success) {
        setModules(modulesRes.data.modules || []);
      }
    } catch (error) {
      console.error('Failed to load data:', error);
      toast.error('Failed to load developer portal data');
    } finally {
      setLoading(false);
    }
  };

  // Handle API key creation
  const handleCreateApiKey = async () => {
    if (!createForm.client_name || createForm.allowed_modules.length === 0) {
      toast.error('Please provide client name and select at least one module');
      return;
    }

    try {
      const response = await AdminService.createApiKey(createForm);
      if (response.success) {
        setNewApiKey(response.data.api_key);
        toast.success('API key created successfully!');
        loadData();
        setShowCreateDialog(false);
        // Reset form
        setCreateForm({
          client_name: '',
          client_description: '',
          client_contact_email: '',
          allowed_modules: [],
          ip_restrictions: [],
        });
      } else {
        toast.error('Failed to create API key');
      }
    } catch (error) {
      console.error('Failed to create API key:', error);
      toast.error('Failed to create API key');
    }
  };

  // Handle API key revocation
  const handleRevokeApiKey = async (keyId: string, keyName: string) => {
    const reason = prompt(`Are you sure you want to revoke the API key for "${keyName}"? Please provide a reason:`);
    if (!reason) return;

    try {
      const response = await AdminService.revokeApiKey(keyId, reason);
      if (response.success) {
        toast.success('API key revoked successfully');
        loadData();
      } else {
        toast.error('Failed to revoke API key');
      }
    } catch (error) {
      console.error('Failed to revoke API key:', error);
      toast.error('Failed to revoke API key');
    }
  };

  // Add module to create form
  const addModuleToForm = (moduleId: string) => {
    if (createForm.allowed_modules.some(m => m.module_id === moduleId)) {
      toast.error('Module already added');
      return;
    }

    setCreateForm(prev => ({
      ...prev,
      allowed_modules: [...prev.allowed_modules, {
        module_id: moduleId,
        access_level: 'bronze',
        custom_quotas: {}
      }]
    }));
  };

  // Remove module from create form
  const removeModuleFromForm = (moduleId: string) => {
    setCreateForm(prev => ({
      ...prev,
      allowed_modules: prev.allowed_modules.filter(m => m.module_id !== moduleId)
    }));
  };

  // Update module in create form
  const updateModuleInForm = (moduleId: string, updates: Partial<{ access_level: string; custom_quotas: Record<string, any> }>) => {
    setCreateForm(prev => ({
      ...prev,
      allowed_modules: prev.allowed_modules.map(m => 
        m.module_id === moduleId ? { ...m, ...updates } : m
      )
    }));
  };

  // Copy to clipboard
  const copyToClipboard = (text: string, label: string) => {
    navigator.clipboard.writeText(text).then(() => {
      toast.success(`${label} copied to clipboard`);
    }).catch(() => {
      toast.error('Failed to copy to clipboard');
    });
  };

  // Get status color
  const getStatusColor = (status: string) => {
    switch (status) {
      case 'active': return 'bg-green-100 text-green-800';
      case 'revoked': return 'bg-red-100 text-red-800';
      case 'expired': return 'bg-yellow-100 text-yellow-800';
      default: return 'bg-gray-100 text-gray-800';
    }
  };

  // Get access level color
  const getAccessLevelColor = (level: string) => {
    const accessLevel = ACCESS_LEVELS.find(al => al.value === level);
    return accessLevel?.color || 'text-gray-600';
  };

  // Check permissions
  const canManageApiKeys = hasModuleAccess('admin') && canPerformAction('admin', 'manage_api_keys');

  if (!canManageApiKeys) {
    return (
      <div className="flex items-center justify-center min-h-[60vh]">
        <div className="text-center max-w-md">
          <Shield className="w-16 h-16 text-gray-400 mx-auto mb-4" />
          <h2 className="text-xl font-semibold text-gray-900 mb-2">Access Denied</h2>
          <p className="text-gray-600 mb-4">
            You don&apos;t have permission to access the developer portal. This feature requires admin-level access.
          </p>
        </div>
      </div>
    );
  }

  if (loading) {
    return (
      <div className="flex items-center justify-center p-8">
        <div className="animate-spin rounded-full h-8 w-8 border-b-2 border-blue-600"></div>
        <span className="ml-2 text-gray-600">Loading developer portal...</span>
      </div>
    );
  }

  return (
    <div className="p-6 max-w-7xl mx-auto">
      <div className="mb-6">
        <h1 className="text-2xl font-bold text-gray-900 mb-2">Developer Portal</h1>
        <p className="text-gray-600">Manage API keys and third-party integrations</p>
      </div>

      {/* Tab Navigation */}
      <div className="flex space-x-1 mb-6 bg-gray-100 p-1 rounded-lg w-fit">
        <button
          onClick={() => setActiveTab('overview')}
          className={`px-4 py-2 rounded-md font-medium transition-colors ${
            activeTab === 'overview'
              ? 'bg-white text-blue-600 shadow-sm'
              : 'text-gray-600 hover:text-gray-900'
          }`}
        >
          <BarChart3 className="w-4 h-4 inline mr-2" />
          Overview
        </button>
        <button
          onClick={() => setActiveTab('keys')}
          className={`px-4 py-2 rounded-md font-medium transition-colors ${
            activeTab === 'keys'
              ? 'bg-white text-blue-600 shadow-sm'
              : 'text-gray-600 hover:text-gray-900'
          }`}
        >
          <Key className="w-4 h-4 inline mr-2" />
          API Keys
        </button>
        <button
          onClick={() => setActiveTab('docs')}
          className={`px-4 py-2 rounded-md font-medium transition-colors ${
            activeTab === 'docs'
              ? 'bg-white text-blue-600 shadow-sm'
              : 'text-gray-600 hover:text-gray-900'
          }`}
        >
          <BookOpen className="w-4 h-4 inline mr-2" />
          Documentation
        </button>
        <button
          onClick={() => setActiveTab('usage')}
          className={`px-4 py-2 rounded-md font-medium transition-colors ${
            activeTab === 'usage'
              ? 'bg-white text-blue-600 shadow-sm'
              : 'text-gray-600 hover:text-gray-900'
          }`}
        >
          <Activity className="w-4 h-4 inline mr-2" />
          Usage Analytics
        </button>
      </div>

      {/* Overview Tab */}
      {activeTab === 'overview' && (
        <div className="space-y-6">
          {/* Stats Overview */}
          <div className="grid grid-cols-1 md:grid-cols-4 gap-6">
            <div className="bg-white rounded-lg shadow p-6">
              <div className="flex items-center">
                <div className="p-2 bg-blue-100 rounded-lg">
                  <Key className="w-6 h-6 text-blue-600" />
                </div>
                <div className="ml-4">
                  <p className="text-sm font-medium text-gray-600">Total API Keys</p>
                  <p className="text-2xl font-bold text-gray-900">{apiKeys.length}</p>
                </div>
              </div>
            </div>
            
            <div className="bg-white rounded-lg shadow p-6">
              <div className="flex items-center">
                <div className="p-2 bg-green-100 rounded-lg">
                  <Activity className="w-6 h-6 text-green-600" />
                </div>
                <div className="ml-4">
                  <p className="text-sm font-medium text-gray-600">Active Keys</p>
                  <p className="text-2xl font-bold text-gray-900">
                    {apiKeys.filter(key => key.status === 'active').length}
                  </p>
                </div>
              </div>
            </div>
            
            <div className="bg-white rounded-lg shadow p-6">
              <div className="flex items-center">
                <div className="p-2 bg-purple-100 rounded-lg">
                  <BarChart3 className="w-6 h-6 text-purple-600" />
                </div>
                <div className="ml-4">
                  <p className="text-sm font-medium text-gray-600">Total Requests</p>
                  <p className="text-2xl font-bold text-gray-900">
                    {apiKeys.reduce((sum, key) => sum + key.total_requests, 0).toLocaleString()}
                  </p>
                </div>
              </div>
            </div>
            
            <div className="bg-white rounded-lg shadow p-6">
              <div className="flex items-center">
                <div className="p-2 bg-orange-100 rounded-lg">
                  <Settings className="w-6 h-6 text-orange-600" />
                </div>
                <div className="ml-4">
                  <p className="text-sm font-medium text-gray-600">Available Modules</p>
                  <p className="text-2xl font-bold text-gray-900">{modules.length}</p>
                </div>
              </div>
            </div>
          </div>

          {/* Recent API Keys */}
          <div className="bg-white rounded-lg shadow">
            <div className="p-6 border-b">
              <div className="flex items-center justify-between">
                <h3 className="text-lg font-medium text-gray-900">Recent API Keys</h3>
                <Button onClick={() => setShowCreateDialog(true)} size="sm">
                  <Plus className="w-4 h-4 mr-2" />
                  Create New Key
                </Button>
              </div>
            </div>
            <div className="divide-y">
              {apiKeys.slice(0, 5).map((apiKey) => (
                <div key={apiKey.id} className="p-6">
                  <div className="flex items-center justify-between">
                    <div className="flex items-center space-x-4">
                      <div className="p-2 bg-gray-100 rounded-lg">
                        <Key className="w-5 h-5 text-gray-600" />
                      </div>
                      <div>
                        <h4 className="font-medium text-gray-900">{apiKey.client_name}</h4>
                        <p className="text-sm text-gray-500">Key: {apiKey.key_prefix}...</p>
                      </div>
                    </div>
                    <div className="flex items-center space-x-3">
                      <span className={`px-2 py-1 rounded-full text-xs font-medium ${getStatusColor(apiKey.status)}`}>
                        {apiKey.status}
                      </span>
                      <div className="text-sm text-gray-500">
                        {apiKey.total_requests.toLocaleString()} requests
                      </div>
                    </div>
                  </div>
                  <div className="mt-3 flex flex-wrap gap-2">
                    {apiKey.allowed_modules.map((module) => (
                      <span key={module.module_id} className="inline-flex items-center px-2 py-1 rounded-full text-xs bg-blue-100 text-blue-800">
                        {module.module_name}
                        <span className={`ml-1 ${getAccessLevelColor(module.access_level)}`}>
                          ({module.access_level})
                        </span>
                      </span>
                    ))}
                  </div>
                </div>
              ))}
            </div>
          </div>

          {/* Available Modules */}
          <div className="bg-white rounded-lg shadow">
            <div className="p-6 border-b">
              <h3 className="text-lg font-medium text-gray-900">Available Modules</h3>
              <p className="text-sm text-gray-600">Choose from these modules when creating API keys</p>
            </div>
            <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-6 p-6">
              {modules.map((module) => (
                <div key={module.id} className="border rounded-lg p-4">
                  <div className="flex items-start justify-between mb-3">
                    <div>
                      <h4 className="font-medium text-gray-900">{module.display_name}</h4>
                      <p className="text-sm text-gray-500">{module.name}</p>
                    </div>
                    <span className="px-2 py-1 bg-green-100 text-green-800 rounded-full text-xs font-medium">
                      {module.status}
                    </span>
                  </div>
                  <p className="text-sm text-gray-600 mb-3">
                    {module.description || 'No description available'}
                  </p>
                  <div className="text-xs text-gray-500">
                    <span className="font-medium">Category:</span> {module.category}
                  </div>
                </div>
              ))}
            </div>
          </div>
        </div>
      )}

      {/* API Keys Tab */}
      {activeTab === 'keys' && (
        <div className="space-y-6">
          <div className="flex items-center justify-between">
            <div>
              <h2 className="text-lg font-semibold text-gray-900">API Key Management</h2>
              <p className="text-sm text-gray-600">Create and manage API keys for third-party integrations</p>
            </div>
            <Button onClick={() => setShowCreateDialog(true)}>
              <Plus className="w-4 h-4 mr-2" />
              Create API Key
            </Button>
          </div>

          <div className="grid grid-cols-1 gap-6">
            {apiKeys.map((apiKey) => (
              <div key={apiKey.id} className="bg-white rounded-lg shadow p-6">
                <div className="flex items-start justify-between mb-4">
                  <div className="flex items-center space-x-4">
                    <div className="p-3 bg-blue-100 rounded-lg">
                      <Key className="w-6 h-6 text-blue-600" />
                    </div>
                    <div>
                      <h3 className="font-semibold text-gray-900">{apiKey.client_name}</h3>
                      <p className="text-sm text-gray-500">{apiKey.client_description || 'No description'}</p>
                      <div className="flex items-center space-x-4 mt-1">
                        <span className="text-xs text-gray-500">
                          Created: {new Date(apiKey.created_at).toLocaleDateString()}
                        </span>
                        {apiKey.last_used_at && (
                          <span className="text-xs text-gray-500">
                            Last used: {new Date(apiKey.last_used_at).toLocaleDateString()}
                          </span>
                        )}
                      </div>
                    </div>
                  </div>
                  <div className="flex items-center space-x-2">
                    <span className={`px-3 py-1 rounded-full text-xs font-medium ${getStatusColor(apiKey.status)}`}>
                      {apiKey.status}
                    </span>
                    {apiKey.status === 'active' && (
                      <Button
                        size="sm"
                        variant="outline"
                        onClick={() => handleRevokeApiKey(apiKey.id, apiKey.client_name)}
                      >
                        <Trash2 className="w-4 h-4" />
                      </Button>
                    )}
                  </div>
                </div>

                <div className="grid grid-cols-1 md:grid-cols-3 gap-4 mb-4">
                  <div className="p-3 bg-gray-50 rounded-lg">
                    <div className="flex items-center justify-between">
                      <span className="text-sm font-medium text-gray-700">API Key</span>
                      <div className="flex items-center space-x-2">
                        <button
                          onClick={() => setShowKeyValue(showKeyValue === apiKey.id ? null : apiKey.id)}
                          className="p-1 text-gray-400 hover:text-gray-600"
                        >
                          {showKeyValue === apiKey.id ? <EyeOff className="w-4 h-4" /> : <Eye className="w-4 h-4" />}
                        </button>
                        <button
                          onClick={() => copyToClipboard(apiKey.key_prefix + '...', 'API Key')}
                          className="p-1 text-gray-400 hover:text-gray-600"
                        >
                          <Copy className="w-4 h-4" />
                        </button>
                      </div>
                    </div>
                    <code className="text-sm text-gray-900 font-mono">
                      {showKeyValue === apiKey.id ? `${apiKey.key_prefix}${'*'.repeat(32)}` : `${apiKey.key_prefix}...`}
                    </code>
                  </div>

                  <div className="p-3 bg-gray-50 rounded-lg">
                    <span className="text-sm font-medium text-gray-700">Total Requests</span>
                    <div className="text-lg font-semibold text-gray-900">
                      {apiKey.total_requests.toLocaleString()}
                    </div>
                  </div>

                  <div className="p-3 bg-gray-50 rounded-lg">
                    <span className="text-sm font-medium text-gray-700">Modules</span>
                    <div className="text-lg font-semibold text-gray-900">
                      {apiKey.allowed_modules.length}
                    </div>
                  </div>
                </div>

                <div className="space-y-3">
                  <div>
                    <h4 className="text-sm font-medium text-gray-700 mb-2">Allowed Modules</h4>
                    <div className="flex flex-wrap gap-2">
                      {apiKey.allowed_modules.map((module) => (
                        <span key={module.module_id} className="inline-flex items-center px-3 py-1 rounded-full text-xs bg-blue-100 text-blue-800">
                          {module.module_name}
                          <span className={`ml-2 px-1 rounded text-xs ${getAccessLevelColor(module.access_level)}`}>
                            {module.access_level.toUpperCase()}
                          </span>
                        </span>
                      ))}
                    </div>
                  </div>

                  {apiKey.ip_restrictions.length > 0 && (
                    <div>
                      <h4 className="text-sm font-medium text-gray-700 mb-2">IP Restrictions</h4>
                      <div className="flex flex-wrap gap-2">
                        {apiKey.ip_restrictions.map((ip, index) => (
                          <span key={index} className="inline-flex items-center px-2 py-1 rounded text-xs bg-gray-100 text-gray-800">
                            <Globe className="w-3 h-3 mr-1" />
                            {ip}
                          </span>
                        ))}
                      </div>
                    </div>
                  )}

                  {apiKey.expires_at && (
                    <div className="flex items-center text-sm text-gray-600">
                      <Clock className="w-4 h-4 mr-2" />
                      Expires: {new Date(apiKey.expires_at).toLocaleDateString()}
                    </div>
                  )}
                </div>
              </div>
            ))}
          </div>
        </div>
      )}

      {/* Documentation Tab */}
      {activeTab === 'docs' && (
        <div className="space-y-6">
          <div className="bg-white rounded-lg shadow">
            <div className="p-6 border-b">
              <h2 className="text-lg font-semibold text-gray-900">API Documentation</h2>
              <p className="text-sm text-gray-600">Complete guide to using our module-based API</p>
            </div>
            <div className="p-6 space-y-6">
              {/* Authentication */}
              <div>
                <h3 className="text-md font-semibold text-gray-900 mb-3 flex items-center">
                  <Shield className="w-5 h-5 mr-2 text-blue-600" />
                  Authentication
                </h3>
                <div className="bg-gray-50 rounded-lg p-4">
                  <p className="text-sm text-gray-700 mb-3">
                    Include your API key in the Authorization header:
                  </p>
                  <code className="block bg-gray-900 text-green-400 p-3 rounded text-sm font-mono">
                    curl -H &quot;Authorization: Bearer YOUR_API_KEY&quot; \
                    <br />     https://api.epsx.com/v1/modules/stock-ranking/rankings
                  </code>
                </div>
              </div>

              {/* Base URL */}
              <div>
                <h3 className="text-md font-semibold text-gray-900 mb-3 flex items-center">
                  <Globe className="w-5 h-5 mr-2 text-blue-600" />
                  Base URL
                </h3>
                <div className="bg-gray-50 rounded-lg p-4">
                  <code className="text-sm font-mono text-gray-900">
                    https://api.epsx.com/v1/
                  </code>
                </div>
              </div>

              {/* Available Endpoints */}
              <div>
                <h3 className="text-md font-semibold text-gray-900 mb-3 flex items-center">
                  <Code className="w-5 h-5 mr-2 text-blue-600" />
                  Available Endpoints
                </h3>
                <div className="space-y-4">
                  {modules.map((module) => (
                    <div key={module.id} className="border rounded-lg p-4">
                      <h4 className="font-medium text-gray-900 mb-2">{module.display_name}</h4>
                      <div className="space-y-2 text-sm">
                        <div className="flex items-center space-x-3">
                          <span className="px-2 py-1 bg-green-100 text-green-800 rounded text-xs font-mono">GET</span>
                          <code className="text-gray-700">/modules/{module.name}/status</code>
                          <span className="text-gray-500">- Get module status</span>
                        </div>
                        <div className="flex items-center space-x-3">
                          <span className="px-2 py-1 bg-blue-100 text-blue-800 rounded text-xs font-mono">GET</span>
                          <code className="text-gray-700">/modules/{module.name}/data</code>
                          <span className="text-gray-500">- Get module data</span>
                        </div>
                        <div className="flex items-center space-x-3">
                          <span className="px-2 py-1 bg-purple-100 text-purple-800 rounded text-xs font-mono">POST</span>
                          <code className="text-gray-700">/modules/{module.name}/analyze</code>
                          <span className="text-gray-500">- Perform analysis</span>
                        </div>
                      </div>
                    </div>
                  ))}
                </div>
              </div>

              {/* Rate Limits */}
              <div>
                <h3 className="text-md font-semibold text-gray-900 mb-3 flex items-center">
                  <AlertTriangle className="w-5 h-5 mr-2 text-blue-600" />
                  Rate Limits
                </h3>
                <div className="bg-yellow-50 border border-yellow-200 rounded-lg p-4">
                  <div className="space-y-2 text-sm text-yellow-800">
                    <div><strong>Bronze:</strong> 100 requests/hour, 1,000 requests/day</div>
                    <div><strong>Silver:</strong> 500 requests/hour, 5,000 requests/day</div>
                    <div><strong>Gold:</strong> 2,000 requests/hour, 20,000 requests/day</div>
                    <div><strong>Platinum:</strong> 10,000 requests/hour, 100,000 requests/day</div>
                    <div><strong>Enterprise:</strong> Unlimited (fair usage policy)</div>
                  </div>
                </div>
              </div>

              {/* Error Codes */}
              <div>
                <h3 className="text-md font-semibold text-gray-900 mb-3">Common Error Codes</h3>
                <div className="space-y-2 text-sm">
                  <div className="flex items-center space-x-3">
                    <code className="px-2 py-1 bg-red-100 text-red-800 rounded">401</code>
                    <span className="text-gray-700">Unauthorized - Invalid API key</span>
                  </div>
                  <div className="flex items-center space-x-3">
                    <code className="px-2 py-1 bg-red-100 text-red-800 rounded">403</code>
                    <span className="text-gray-700">Forbidden - Insufficient permissions</span>
                  </div>
                  <div className="flex items-center space-x-3">
                    <code className="px-2 py-1 bg-yellow-100 text-yellow-800 rounded">429</code>
                    <span className="text-gray-700">Too Many Requests - Rate limit exceeded</span>
                  </div>
                  <div className="flex items-center space-x-3">
                    <code className="px-2 py-1 bg-gray-100 text-gray-800 rounded">500</code>
                    <span className="text-gray-700">Internal Server Error - Contact support</span>
                  </div>
                </div>
              </div>
            </div>
          </div>
        </div>
      )}

      {/* Usage Analytics Tab */}
      {activeTab === 'usage' && (
        <div className="space-y-6">
          <div className="bg-white rounded-lg shadow p-6">
            <div className="flex items-center justify-between mb-6">
              <h2 className="text-lg font-semibold text-gray-900">Usage Analytics</h2>
              <Button variant="outline" size="sm">
                <Download className="w-4 h-4 mr-2" />
                Export Data
              </Button>
            </div>

            {/* Usage Charts Placeholder */}
            <div className="grid grid-cols-1 md:grid-cols-2 gap-6">
              <div className="p-6 border border-gray-200 rounded-lg">
                <h3 className="font-medium text-gray-900 mb-4">Requests Over Time</h3>
                <div className="h-48 bg-gray-50 rounded flex items-center justify-center">
                  <div className="text-center text-gray-500">
                    <BarChart3 className="w-8 h-8 mx-auto mb-2" />
                    <p>Chart placeholder</p>
                    <p className="text-xs">Requests timeline would be displayed here</p>
                  </div>
                </div>
              </div>

              <div className="p-6 border border-gray-200 rounded-lg">
                <h3 className="font-medium text-gray-900 mb-4">Module Usage Distribution</h3>
                <div className="h-48 bg-gray-50 rounded flex items-center justify-center">
                  <div className="text-center text-gray-500">
                    <Activity className="w-8 h-8 mx-auto mb-2" />
                    <p>Chart placeholder</p>
                    <p className="text-xs">Module usage breakdown would be displayed here</p>
                  </div>
                </div>
              </div>
            </div>

            {/* Usage Table */}
            <div className="mt-6">
              <h3 className="font-medium text-gray-900 mb-4">API Key Usage Details</h3>
              <div className="overflow-x-auto">
                <table className="min-w-full border-collapse border border-gray-200">
                  <thead className="bg-gray-50">
                    <tr>
                      <th className="border border-gray-200 px-4 py-3 text-left text-sm font-medium text-gray-900">API Key</th>
                      <th className="border border-gray-200 px-4 py-3 text-left text-sm font-medium text-gray-900">Client</th>
                      <th className="border border-gray-200 px-4 py-3 text-left text-sm font-medium text-gray-900">Requests (24h)</th>
                      <th className="border border-gray-200 px-4 py-3 text-left text-sm font-medium text-gray-900">Most Used Module</th>
                      <th className="border border-gray-200 px-4 py-3 text-left text-sm font-medium text-gray-900">Last Activity</th>
                    </tr>
                  </thead>
                  <tbody>
                    {apiKeys.map((apiKey) => (
                      <tr key={apiKey.id} className="hover:bg-gray-50">
                        <td className="border border-gray-200 px-4 py-3 text-sm font-mono text-gray-900">
                          {apiKey.key_prefix}...
                        </td>
                        <td className="border border-gray-200 px-4 py-3 text-sm text-gray-900">
                          {apiKey.client_name}
                        </td>
                        <td className="border border-gray-200 px-4 py-3 text-sm text-gray-900">
                          {Math.floor(apiKey.total_requests * 0.1).toLocaleString()}
                        </td>
                        <td className="border border-gray-200 px-4 py-3 text-sm text-gray-900">
                          {apiKey.allowed_modules[0]?.module_name || 'N/A'}
                        </td>
                        <td className="border border-gray-200 px-4 py-3 text-sm text-gray-900">
                          {apiKey.last_used_at ? new Date(apiKey.last_used_at).toLocaleDateString() : 'Never'}
                        </td>
                      </tr>
                    ))}
                  </tbody>
                </table>
              </div>
            </div>
          </div>
        </div>
      )}

      {/* Create API Key Dialog */}
      {showCreateDialog && (
        <div className="fixed inset-0 bg-black bg-opacity-50 flex items-center justify-center z-50">
          <div className="bg-white rounded-lg shadow-xl w-full max-w-4xl max-h-[90vh] overflow-hidden">
            <div className="p-6 border-b">
              <h2 className="text-lg font-semibold text-gray-900">Create API Key</h2>
              <p className="text-sm text-gray-500">Generate a new API key for third-party integration</p>
            </div>
            
            <div className="p-6 max-h-[70vh] overflow-y-auto">
              <div className="space-y-6">
                <div className="grid grid-cols-1 md:grid-cols-2 gap-4">
                  <FormField label="Client Name" required>
                    <Input
                      value={createForm.client_name}
                      onChange={(e) => setCreateForm(prev => ({ ...prev, client_name: e.target.value }))}
                      placeholder="My Application"
                    />
                  </FormField>
                  
                  <FormField label="Contact Email">
                    <Input
                      type="email"
                      value={createForm.client_contact_email || ''}
                      onChange={(e) => setCreateForm(prev => ({ ...prev, client_contact_email: e.target.value }))}
                      placeholder="contact@example.com"
                    />
                  </FormField>
                </div>
                
                <FormField label="Description">
                  <Textarea
                    value={createForm.client_description || ''}
                    onChange={(e) => setCreateForm(prev => ({ ...prev, client_description: e.target.value }))}
                    placeholder="Brief description of your application and use case"
                    rows={3}
                  />
                </FormField>
                
                <FormField label="Expiration Date (Optional)">
                  <Input
                    type="datetime-local"
                    value={createForm.expires_at || ''}
                    onChange={(e) => setCreateForm(prev => ({ ...prev, expires_at: e.target.value || undefined }))}
                  />
                </FormField>
                
                <FormField label="IP Restrictions (Optional)">
                  <Textarea
                    value={createForm.ip_restrictions.join('\n')}
                    onChange={(e) => setCreateForm(prev => ({ 
                      ...prev, 
                      ip_restrictions: e.target.value.split('\n').filter(ip => ip.trim()) 
                    }))}
                    placeholder="192.168.1.0/24\n203.0.113.0/24\nOne IP address or CIDR block per line"
                    rows={3}
                  />
                </FormField>
                
                {/* Selected Modules */}
                {createForm.allowed_modules.length > 0 && (
                  <div>
                    <h3 className="font-medium text-gray-900 mb-3">Selected Modules</h3>
                    <div className="space-y-3">
                      {createForm.allowed_modules.map((moduleConfig) => {
                        const module = modules.find(m => m.id === moduleConfig.module_id);
                        return (
                          <div key={moduleConfig.module_id} className="p-4 bg-gray-50 rounded-lg">
                            <div className="flex items-center justify-between mb-3">
                              <div>
                                <h4 className="font-medium text-gray-900">{module?.display_name}</h4>
                                <p className="text-sm text-gray-500">{module?.name}</p>
                              </div>
                              <Button
                                size="sm"
                                variant="outline"
                                onClick={() => removeModuleFromForm(moduleConfig.module_id)}
                              >
                                Remove
                              </Button>
                            </div>
                            
                            <FormField label="Access Level">
                              <Select
                                value={moduleConfig.access_level}
                                onChange={(e) => updateModuleInForm(moduleConfig.module_id, { access_level: e.target.value })}
                              >
                                {ACCESS_LEVELS.map(level => (
                                  <option key={level.value} value={level.value}>
                                    {level.label}
                                  </option>
                                ))}
                              </Select>
                            </FormField>
                          </div>
                        );
                      })}
                    </div>
                  </div>
                )}
                
                {/* Available Modules */}
                <div>
                  <h3 className="font-medium text-gray-900 mb-3">Available Modules</h3>
                  <div className="grid grid-cols-1 md:grid-cols-2 gap-3 max-h-60 overflow-y-auto">
                    {modules
                      .filter(module => !createForm.allowed_modules.some(m => m.module_id === module.id))
                      .map((module) => (
                        <div key={module.id} className="p-3 border rounded-lg hover:bg-gray-50 cursor-pointer"
                             onClick={() => addModuleToForm(module.id)}>
                          <div className="flex items-center justify-between">
                            <div>
                              <h4 className="font-medium text-gray-900 text-sm">{module.display_name}</h4>
                              <p className="text-xs text-gray-500">{module.category}</p>
                            </div>
                            <Plus className="w-4 h-4 text-blue-600" />
                          </div>
                        </div>
                      ))}
                  </div>
                </div>
              </div>
            </div>
            
            <div className="p-6 border-t bg-gray-50 flex justify-end space-x-3">
              <Button variant="outline" onClick={() => setShowCreateDialog(false)}>
                Cancel
              </Button>
              <Button onClick={handleCreateApiKey}>
                Create API Key
              </Button>
            </div>
          </div>
        </div>
      )}

      {/* New API Key Display */}
      {newApiKey && (
        <div className="fixed inset-0 bg-black bg-opacity-50 flex items-center justify-center z-50">
          <div className="bg-white rounded-lg shadow-xl w-full max-w-md">
            <div className="p-6 border-b">
              <h2 className="text-lg font-semibold text-gray-900">API Key Created</h2>
            </div>
            
            <div className="p-6">
              <div className="bg-yellow-50 border border-yellow-200 rounded-lg p-4 mb-4">
                <div className="flex items-start">
                  <AlertTriangle className="w-5 h-5 text-yellow-600 mt-0.5 mr-2" />
                  <div>
                    <h3 className="font-medium text-yellow-800">Important</h3>
                    <p className="text-sm text-yellow-700 mt-1">
                      This is the only time you&apos;ll see your API key. Please copy it and store it securely.
                    </p>
                  </div>
                </div>
              </div>
              
              <div className="bg-gray-50 rounded-lg p-4">
                <div className="flex items-center justify-between mb-2">
                  <label className="text-sm font-medium text-gray-700">Your API Key</label>
                  <button
                    onClick={() => copyToClipboard(newApiKey, 'API Key')}
                    className="p-1 text-gray-400 hover:text-gray-600"
                  >
                    <Copy className="w-4 h-4" />
                  </button>
                </div>
                <code className="block text-sm font-mono text-gray-900 bg-white p-3 rounded border break-all">
                  {newApiKey}
                </code>
              </div>
            </div>
            
            <div className="p-6 border-t bg-gray-50 flex justify-end">
              <Button onClick={() => setNewApiKey(null)}>
                I&apos;ve Saved the Key
              </Button>
            </div>
          </div>
        </div>
      )}
    </div>
  );
};
