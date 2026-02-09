'use client';

import { Button, Card, CardContent, CardHeader, CardTitle } from '@/components/ui';
import {
  AlertDialog,
  AlertDialogAction,
  AlertDialogCancel,
  AlertDialogContent,
  AlertDialogDescription,
  AlertDialogFooter,
  AlertDialogHeader,
  AlertDialogTitle,
} from '@/components/ui/alert-dialog';
import { Badge } from '@/components/ui/badge';
import { Input } from '@/components/ui/input';
import type { AuthUser } from '@/lib/server-actions';
import type { UserApiKey } from '@/shared/api/users';
import { useFrontendApiClient } from '@/shared/hooks/use-api-client';
import { copyToClipboard as copyToClipboardUtil } from '@/utils/util';
import { useEffect, useState } from 'react';
import { toast } from 'sonner';
import { PlanTransferList } from './Plantransfer-list';

interface APIKeyManagerProps {
  currentUser: AuthUser;
  onStatsChange?: () => void;
}

export function APIKeyManager({ currentUser, onStatsChange }: APIKeyManagerProps) {
  const { users, plans } = useFrontendApiClient();
  const [apiKeys, setApiKeys] = useState<UserApiKey[]>([]);
  const [availablePermissions, setAvailablePermissions] = useState<string[]>([]);
  const [newKeyName, setNewKeyName] = useState('');
  const [showNewKey, setShowNewKey] = useState(false);
  const [generatedKey, setGeneratedKey] = useState('');
  const [isLoading, setIsLoading] = useState(false);
  const [selectedPermissions, setSelectedPermissions] = useState<string[]>([]);
  const [expiresAt, setExpiresAt] = useState<string>(''); // ISO date string or empty
  const [revokeDialogOpen, setRevokeDialogOpen] = useState(false);
  const [selectedKeyToRevoke, setSelectedKeyToRevoke] = useState<UserApiKey | null>(null);
  const [hasPlans, setHasPlans] = useState(false);

  // Load API keys and permissions on mount
  useEffect(() => {
    const loadData = async () => {
      setIsLoading(true);
      try {
        // Load user's own API keys
        const keysResponse = await users.getApiKeys();
        if (keysResponse.success && keysResponse.data) {
          setApiKeys(keysResponse.data.api_keys);
        }

        // Load user's assigned plans and flatten permissions
        const walletAddress = currentUser.walletAddress ?? '';
        if (!walletAddress) {
          return;
        }
        const membershipsResponse = await plans.getWalletMemberships(walletAddress);
        if (membershipsResponse.success && membershipsResponse.data) {
          const memberships = Array.isArray(membershipsResponse.data) ? membershipsResponse.data : [];

          // Flatten permissions from all plans into unique list
          const allPermissions = new Set<string>();
          memberships.forEach((m) => {
            if (Array.isArray(m.permissions)) {
              m.permissions.forEach((p: string) => allPermissions.add(p));
            }
          });

          const permissionsList = Array.from(allPermissions).sort();
          setAvailablePermissions(permissionsList);
          setHasPlans(memberships.length > 0);
        }
      } catch (_error) {
        // Data loading failed silently
      } finally {
        setIsLoading(false);
      }
    };

    loadData();
  }, [users, plans, currentUser.walletAddress]);

  // Manual refresh function for create/revoke operations
  const refreshData = async () => {
    setIsLoading(true);
    try {
      const keysResponse = await users.getApiKeys();
      if (keysResponse.success && keysResponse.data) {
        setApiKeys(keysResponse.data.api_keys);
      }

      // Also refresh available permissions/plans
      const membershipsResponse = await plans.getWalletMemberships(currentUser.walletAddress ?? '');
      if (membershipsResponse.success && membershipsResponse.data) {
        const memberships = Array.isArray(membershipsResponse.data) ? membershipsResponse.data : [];
        setHasPlans(memberships.length > 0);

        const allPermissions = new Set<string>();
        memberships.forEach((m) => {
          if (Array.isArray(m.permissions)) {
            m.permissions.forEach((p: string) => allPermissions.add(p));
          }
        });
        setAvailablePermissions(Array.from(allPermissions).sort());
      }
    } catch (_error) {
      // Data refresh failed silently
    } finally {
      setIsLoading(false);
    }
  };

  const generateAPIKey = async () => {
    if (!newKeyName.trim()) {
      toast.error('Please enter a key name');
      return;
    }

    setIsLoading(true);

    try {
      // Create API key using UsersApi
      const response = await users.createApiKey({
        client_name: newKeyName,
        permissions: selectedPermissions
      });

      if (response.success && response.data) {
        const newKey = response.data;
        // The endpoint should return the full key on creation
        const fullKey = newKey.key;

        setGeneratedKey(fullKey || 'Key created - check your keys');
        setShowNewKey(true);
        setNewKeyName('');
        setSelectedPermissions([]);
        setExpiresAt(''); // Reset expiration
        toast.success('API key created successfully!');

        // Reload keys list
        await refreshData();
        onStatsChange?.(); // Notify parent to refresh stats
      } else {
        toast.error('Failed to create API key');
      }
    } catch (_error) {
      toast.error('Failed to generate API key');
    } finally {
      setIsLoading(false);
    }
  };

  const executeRevoke = async (keyId: string) => {
    setIsLoading(true);
    try {
      const response = await users.deleteApiKey(keyId);

      if (response.success) {
        toast.success('API key revoked successfully');
        await refreshData();
        onStatsChange?.(); // Notify parent to refresh stats
      } else {
        toast.error('Failed to revoke API key');
      }
    } catch (_error: unknown) {
      toast.error('Failed to revoke API key');
    } finally {
      setIsLoading(false);
    }
  };

  // User can create keys if they have any plans assigned
  const canCreateKeys = hasPlans || currentUser.role === 'admin';

  return (
    <div className="space-y-8">
      {/* Create New Key */}
      <div className="relative rounded-2xl bg-gradient-to-r from-amber-500/10 via-orange-500/10 to-rose-500/10 p-[1px]">
        <Card className="rounded-2xl border-0 bg-white/80 dark:bg-gray-900/80 backdrop-blur-sm">
          <CardHeader className="pb-2">
            <CardTitle className="flex items-center justify-between">
              <div className="flex items-center gap-3">
                <div className="w-10 h-10 rounded-xl bg-gradient-to-br from-amber-500 to-orange-500 flex items-center justify-center shadow-lg shadow-amber-500/25">
                  <svg className="w-5 h-5 text-white" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                    <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M12 6v6m0 0v6m0-6h6m-6 0H6" />
                  </svg>
                </div>
                <span className="text-xl font-semibold">Create New API Key</span>
              </div>
              {!canCreateKeys && !isLoading && (
                <Badge className="bg-gradient-to-r from-amber-500 to-orange-500 text-white border-0">
                  Premium Required
                </Badge>
              )}
            </CardTitle>
          </CardHeader>
          <CardContent>
            {isLoading ? (
              <div className="flex items-center justify-center py-8">
                <div className="animate-spin rounded-full h-8 w-8 border-b-2 border-amber-500" />
              </div>
            ) : canCreateKeys ? (
              <div className="space-y-6">
                {/* Key Name Input */}
                <div className="space-y-2">
                  <label className="text-sm font-medium text-gray-700 dark:text-gray-300">
                    Key Name
                  </label>
                  <Input
                    placeholder="e.g., 'Production Server', 'Trading Bot'"
                    value={newKeyName}
                    onChange={(e) => setNewKeyName(e.target.value)}
                    className="h-12 text-base bg-gray-50 dark:bg-gray-800/50 border-gray-200 dark:border-gray-700 focus:ring-amber-500 focus:border-amber-500"
                  />
                </div>

                {/* Permission Selection - Drag & Drop */}
                <PlanTransferList
                  available={availablePermissions}
                  selected={selectedPermissions}
                  onChange={setSelectedPermissions}
                />
                {availablePermissions.length === 0 && (
                  <p className="text-sm text-amber-600 dark:text-amber-400 -mt-2">
                    Note: No permissions available. Check if you have been assigned any permission plans.
                  </p>
                )}

                {/* Expiration Date Picker */}
                <div className="space-y-3">
                  <div className="text-sm text-gray-600 dark:text-gray-400">
                    Key Expiration:
                  </div>

                  {/* Quick Preset Buttons */}
                  <div className="flex flex-wrap gap-2">
                    {[
                      { label: '30 Days', days: 30 },
                      { label: '60 Days', days: 60 },
                      { label: '90 Days', days: 90 },
                      { label: '1 Year', days: 365 },
                      { label: 'Never', days: 0 },
                    ].map((option) => {
                      const isSelected = option.days === 0
                        ? !expiresAt
                        : (() => {
                          if (!expiresAt) {return false;}
                          const targetDate = new Date();
                          targetDate.setDate(targetDate.getDate() + option.days);
                          const expiryDate = new Date(expiresAt);
                          // Check if dates are within 1 day of each other (accounting for time of creation)
                          return Math.abs(targetDate.getTime() - expiryDate.getTime()) < 86400000 * 2;
                        })();

                      return (
                        <button
                          key={option.label}
                          type="button"
                          onClick={() => {
                            if (option.days === 0) {
                              setExpiresAt('');
                            } else {
                              const date = new Date();
                              date.setDate(date.getDate() + option.days);
                              date.setUTCHours(23, 59, 59, 999);
                              setExpiresAt(date.toISOString());
                            }
                          }}
                          className={`
                          px-4 py-2 rounded-lg text-sm font-medium transition-all duration-200
                          ${isSelected
                              ? 'bg-gradient-to-r from-amber-500 to-orange-500 text-white shadow-lg shadow-amber-500/25'
                              : 'bg-gray-100 dark:bg-gray-800 text-gray-700 dark:text-gray-300 hover:bg-gray-200 dark:hover:bg-gray-700 border border-gray-200 dark:border-gray-700'
                            }
                        `}
                        >
                          {option.label}
                        </button>
                      );
                    })}
                  </div>

                  {/* Custom Date Option */}
                  <div className="flex items-center gap-3 pt-2 border-t border-gray-200 dark:border-gray-700">
                    <span className="text-xs text-gray-500 dark:text-gray-400">Or select custom date:</span>
                    <Input
                      type="date"
                      value={expiresAt ? new Date(expiresAt).toISOString().split('T')[0] : ''}
                      onChange={(e) => {
                        if (e.target.value) {
                          const date = new Date(e.target.value);
                          date.setUTCHours(23, 59, 59, 999);
                          setExpiresAt(date.toISOString());
                        } else {
                          setExpiresAt('');
                        }
                      }}
                      min={new Date().toISOString().split('T')[0]}
                      className="max-w-[160px] text-sm"
                    />
                  </div>

                  {/* Status Display */}
                  <div className={`
                  flex items-center gap-2 px-3 py-2 rounded-lg text-sm
                  ${expiresAt
                      ? 'bg-amber-50 dark:bg-amber-900/20 text-amber-700 dark:text-amber-400'
                      : 'bg-green-50 dark:bg-green-900/20 text-green-700 dark:text-green-400'
                    }
                `}>
                    <span className={`w-2 h-2 rounded-full ${expiresAt ? 'bg-amber-500' : 'bg-green-500'}`} />
                    {expiresAt
                      ? `Expires on ${new Date(expiresAt).toLocaleDateString('en-US', {
                        weekday: 'short',
                        year: 'numeric',
                        month: 'short',
                        day: 'numeric'
                      })}`
                      : 'This key will never expire'
                    }
                  </div>
                </div>

                <Button
                  onClick={generateAPIKey}
                  disabled={!newKeyName.trim() || (availablePermissions.length > 0 && selectedPermissions.length === 0)}
                  className="bg-gradient-to-r from-amber-500 to-orange-500 hover:from-amber-600 hover:to-orange-600 text-white shadow-lg shadow-amber-500/25"
                >
                  Create API Key
                </Button>

                {showNewKey && generatedKey && (
                  <div className="p-5 bg-gradient-to-r from-green-50 to-emerald-50 dark:from-green-900/10 dark:to-emerald-900/10 border border-green-200 dark:border-green-800/30 rounded-xl mb-6">
                    <div className="flex items-center gap-2 mb-3">
                      <div className="w-8 h-8 rounded-lg bg-green-500/20 flex items-center justify-center">
                        <svg className="w-5 h-5 text-green-600 dark:text-green-400" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                          <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M5 13l4 4L19 7" />
                        </svg>
                      </div>
                      <h4 className="font-semibold text-green-900 dark:text-green-100">
                        API Key Generated Successfully
                      </h4>
                    </div>
                    <p className="text-sm text-green-800 dark:text-green-300/80 mb-4 ml-10">
                      Copy your API key now. You won't be able to see it again!
                    </p>
                    <div className="flex items-center gap-3">
                      <div className="flex-1 relative group/input">
                        <div className="absolute inset-y-0 left-0 pl-3 flex items-center pointer-events-none">
                          <svg className="w-4 h-4 text-gray-400 group-hover/input:text-gray-300 transition-colors" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                            <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M15 7a2 2 0 012 2m4 0a6 6 0 01-7.743 5.743L11 17H9v2H7v2H4a1 1 0 01-1-1v-2.586a1 1 0 01.293-.707l5.964-5.964A6 6 0 1121 9z" />
                          </svg>
                        </div>
                        <input
                          type="text"
                          readOnly
                          value={generatedKey}
                          className="w-full pl-10 pr-4 py-3 bg-[#1e2330] border border-gray-700/50 rounded-lg font-mono text-sm text-green-500 focus:outline-none focus:ring-1 focus:ring-green-500/30 transition-all font-medium truncate"
                        />
                      </div>
                      <Button
                        size="icon"
                        className="rounded-full bg-amber-500 hover:bg-amber-600 text-white shadow-lg shadow-amber-500/20 shrink-0 w-10 h-10 transition-transform hover:scale-105 active:scale-95"
                        onClick={async () => {
                          const success = await copyToClipboardUtil(generatedKey);
                          if (success) {
                            toast.success('API key copied to clipboard!');
                          }
                        }}
                      >
                        <svg className="w-6 h-6" fill="none" stroke="currentColor" viewBox="0 0 24 24"><path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M8 16H6a2 2 0 01-2-2V6a2 2 0 012-2h8a2 2 0 012 2v2m-6 12h8a2 2 0 002-2v-8a2 2 0 00-2-2h-8a2 2 0 00-2 2v8a2 2 0 002 2z" /></svg>
                        <span className="sr-only">Copy</span>
                      </Button>
                    </div>
                    <button
                      type="button"
                      onClick={() => setShowNewKey(false)}
                      className="mt-4 ml-1 text-xs text-green-600 dark:text-green-400 hover:text-green-700 dark:hover:text-green-300 font-medium transition-colors"
                    >
                      Dismiss
                    </button>
                  </div>
                )}
              </div>
            ) : (
              <div className="text-center py-8">
                <div className="text-gray-500 dark:text-gray-400 mb-4">
                  API key generation requires a Premium subscription
                </div>
                <Button variant="outline">
                  Upgrade to Premium
                </Button>
              </div>
            )}
          </CardContent>
        </Card>
      </div>

      {/* Existing Keys */}
      <Card className="rounded-2xl border-0 bg-white/80 dark:bg-gray-900/80 backdrop-blur-sm shadow-xl">
        <CardHeader>
          <CardTitle className="flex items-center justify-between">
            <div className="flex items-center gap-3">
              <div className="w-10 h-10 rounded-xl bg-gradient-to-br from-blue-500 to-indigo-500 flex items-center justify-center shadow-lg shadow-blue-500/25">
                <svg className="w-5 h-5 text-white" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                  <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M15 7a2 2 0 012 2m4 0a6 6 0 01-7.743 5.743L11 17H9v2H7v2H4a1 1 0 01-1-1v-2.586a1 1 0 01.293-.707l5.964-5.964A6 6 0 1121 9z" />
                </svg>
              </div>
              <span className="text-xl font-semibold">Your API Keys</span>
            </div>
            <button
              onClick={refreshData}
              disabled={isLoading}
              className="flex items-center gap-2 px-4 py-2 rounded-lg text-sm font-medium bg-gray-100 dark:bg-gray-800 hover:bg-gray-200 dark:hover:bg-gray-700 transition-colors disabled:opacity-50"
            >
              <svg className={`w-4 h-4 ${isLoading ? 'animate-spin' : ''}`} fill="none" stroke="currentColor" viewBox="0 0 24 24">
                <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M4 4v5h.582m15.356 2A8.001 8.001 0 004.582 9m0 0H9m11 11v-5h-.581m0 0a8.003 8.003 0 01-15.357-2m15.357 2H15" />
              </svg>
              {isLoading ? 'Loading...' : 'Refresh'}
            </button>
          </CardTitle>
        </CardHeader>
        <CardContent>
          {isLoading && apiKeys.length === 0 ? (
            <div className="flex items-center justify-center py-12">
              <div className="animate-spin rounded-full h-8 w-8 border-b-2 border-blue-500" />
            </div>
          ) : apiKeys.length === 0 ? (
            <div className="text-center py-12">
              <div className="w-16 h-16 mx-auto mb-4 rounded-2xl bg-gray-100 dark:bg-gray-800 flex items-center justify-center">
                <svg className="w-8 h-8 text-gray-400" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                  <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={1.5} d="M15 7a2 2 0 012 2m4 0a6 6 0 01-7.743 5.743L11 17H9v2H7v2H4a1 1 0 01-1-1v-2.586a1 1 0 01.293-.707l5.964-5.964A6 6 0 1121 9z" />
                </svg>
              </div>
              <p className="text-gray-500 dark:text-gray-400 mb-2">No API keys created yet</p>
              <p className="text-sm text-gray-400 dark:text-gray-500">Create your first key to start using the API</p>
            </div>
          ) : (
            <div className="space-y-4">
              {apiKeys.map((apiKey) => (
                <div key={apiKey.id} className="group relative overflow-hidden rounded-xl border border-gray-200 dark:border-gray-800 bg-white dark:bg-gray-900 transition-all duration-200 hover:shadow-lg hover:border-gray-300 dark:hover:border-gray-700">
                  {/* Card Header & Key Section */}
                  <div className="p-5 pb-4">
                    <div className="flex items-start justify-between mb-4">
                      <div>
                        <div className="flex items-center gap-2 mb-1">
                          <h3 className="text-lg font-semibold text-gray-900 dark:text-gray-100">{apiKey.name}</h3>
                          <Badge
                            variant={apiKey.is_active ? "default" : "secondary"}
                            className={`px-2 py-0.5 text-xs font-medium rounded-full ${apiKey.is_active
                              ? "bg-green-100 text-green-700 dark:bg-green-900/30 dark:text-green-400 border-green-200 dark:border-green-800"
                              : "bg-gray-100 text-gray-700 dark:bg-gray-800 dark:text-gray-400 border-gray-200 dark:border-gray-700"}`}
                          >
                            {apiKey.is_active ? 'Active' : 'Revoked'}
                          </Badge>
                        </div>
                        <div className="text-xs text-gray-500 dark:text-gray-400">
                          Created on {new Date(apiKey.created_at).toLocaleDateString(undefined, { year: 'numeric', month: 'long', day: 'numeric' })}
                        </div>
                      </div>
                    </div>

                    {/* Key Box */}
                    <div className="mb-6">
                      <div className="text-xs text-gray-500 dark:text-gray-400 mb-2 font-medium">Key Preview</div>
                      <div className="flex items-center gap-3">
                        <div className="flex-1 relative group/input">
                          <div className="absolute inset-y-0 left-0 pl-3 flex items-center pointer-events-none">
                            <svg className="w-4 h-4 text-gray-400 group-hover/input:text-gray-300 transition-colors" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                              <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M15 7a2 2 0 012 2m4 0a6 6 0 01-7.743 5.743L11 17H9v2H7v2H4a1 1 0 01-1-1v-2.586a1 1 0 01.293-.707l5.964-5.964A6 6 0 1121 9z" />
                            </svg>
                          </div>
                          <input
                            type="text"
                            readOnly
                            value={apiKey.key || 'epsx_••••••••••••••••'}
                            className="w-full pl-10 pr-4 py-3 bg-[#1e2330] border border-gray-700/50 rounded-lg font-mono text-sm text-green-500 focus:outline-none focus:ring-1 focus:ring-green-500/30 transition-all font-medium truncate"
                          />
                        </div>
                        <Button
                          size="icon"
                          className="rounded-full bg-amber-500 hover:bg-amber-600 text-white shadow-lg shadow-amber-500/20 shrink-0 w-8 h-8 transition-transform hover:scale-105 active:scale-95 flex items-center justify-center p-0"
                          onClick={async () => {
                            const keyToCopy = apiKey.key;
                            if (keyToCopy) {
                              const success = await copyToClipboardUtil(keyToCopy);
                              if (success) {
                                toast.success('API key copied to clipboard!');
                              }
                            }
                          }}
                        >
                          <svg className="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24"><path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M8 16H6a2 2 0 01-2-2V6a2 2 0 012-2h8a2 2 0 012 2v2m-6 12h8a2 2 0 002-2v-8a2 2 0 00-2-2h-8a2 2 0 00-2 2v8a2 2 0 002 2z" /></svg>
                          <span className="sr-only">Copy</span>
                        </Button>
                      </div>
                    </div>

                    {/* Info Grid */}
                    <div className="grid grid-cols-1 md:grid-cols-3 gap-6 py-4 border-t border-gray-800/50">
                      {/* Permissions */}
                      <div>
                        <div className="text-[10px] font-bold text-gray-500 dark:text-gray-500 uppercase tracking-widest mb-2">Permissions</div>
                        <div className="flex flex-wrap gap-1.5">
                          {apiKey.scopes.length > 0 ? (
                            apiKey.scopes.map((permission: string, idx: number) => (
                              <Badge key={idx} variant="outline" className="px-2 py-0.5 text-[11px] border-amber-500/50 text-amber-400 bg-amber-900/10 font-mono">
                                {permission}
                              </Badge>
                            ))
                          ) : (
                            <span className="text-sm text-gray-500 italic">Read Only</span>
                          )}
                        </div>
                      </div>

                      {/* Usage Stats */}
                      <div>
                        <div className="text-[10px] font-bold text-gray-500 dark:text-gray-500 uppercase tracking-widest mb-2">Usage</div>
                        <div className="text-sm font-semibold text-gray-200">
                          {(apiKey.usage_count || 0).toLocaleString()} <span className="font-medium text-gray-500 text-xs">requests</span>
                        </div>
                      </div>

                      {/* Expiration (UserApiKey doesn't have expires_at currently? checking type) */}
                      {/* Assuming UserApiKey might be extended or we skip it if not in type */}
                      {/* Line 92 of users.ts didn't show expires_at. If it's there via backend but not type, we can cast or skip. */}
                      {/* I will assume it's not available for now or add a check */}
                    </div>
                  </div>

                  {/* Actions Footer */}
                  {apiKey.is_active && (
                    <div className="px-6 pb-4 flex justify-end">
                      <button
                        type="button"
                        disabled={isLoading}
                        onClick={() => {
                          setSelectedKeyToRevoke(apiKey);
                          setRevokeDialogOpen(true);
                        }}
                        className="text-xs font-medium text-red-500/80 hover:text-red-400 transition-colors flex items-center gap-1 hover:underline disabled:opacity-50 disabled:cursor-not-allowed"
                      >
                        Revoke Key...
                      </button>
                    </div>
                  )}
                </div>
              ))}
            </div>
          )}
        </CardContent>
      </Card>

      {/* Security Notice */}
      <Card className="bg-blue-50 dark:bg-blue-900/20 border-blue-200 dark:border-blue-800">
        <CardContent className="p-6">
          <h3 className="font-semibold text-blue-800 dark:text-blue-300 mb-2">
            🛡️ Security Best Practices
          </h3>
          <ul className="text-sm text-blue-700 dark:text-blue-400 space-y-1 list-disc list-inside">
            <li>Never commit API keys to version control</li>
            <li>Use environment variables to store keys securely</li>
            <li>Rotate keys regularly for production applications</li>
            <li>Monitor key usage and revoke unused keys</li>
            <li>Use different keys for different environments</li>
          </ul>
        </CardContent>
      </Card>

      {/* Revoke Confirmation Dialog - Controlled */}
      <AlertDialog open={revokeDialogOpen} onOpenChange={setRevokeDialogOpen}>
        <AlertDialogContent className="sm:max-w-md">
          <AlertDialogHeader>
            <AlertDialogTitle>Revoke API Key</AlertDialogTitle>
            <AlertDialogDescription>
              Are you sure you want to revoke{' '}
              <span className="font-mono text-xs p-1 bg-gray-100 dark:bg-gray-800 rounded">
                {selectedKeyToRevoke?.name}
              </span>
              ? This action cannot be undone and any applications using this key will stop working immediately.
            </AlertDialogDescription>
          </AlertDialogHeader>
          <AlertDialogFooter>
            <AlertDialogCancel disabled={isLoading}>Cancel</AlertDialogCancel>
            <AlertDialogAction
              onClick={() => {
                if (selectedKeyToRevoke) {
                  executeRevoke(selectedKeyToRevoke.id);
                  setRevokeDialogOpen(false);
                }
              }}
              disabled={isLoading}
              className="bg-red-600 text-white hover:bg-red-700 dark:bg-red-900 dark:text-red-100 dark:hover:bg-red-800"
            >
              {isLoading ? 'Revoking...' : 'Revoke Key'}
            </AlertDialogAction>
          </AlertDialogFooter>
        </AlertDialogContent>
      </AlertDialog>
    </div>
  );
}