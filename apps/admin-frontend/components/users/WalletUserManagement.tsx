/**
 * Wallet User Management Component
 * Complete Web3 wallet-based user management for admin interface
 * Handles wallet lookup, permission assignment, and group management
 */
'use client';

import { useCallback, useState } from 'react';

import { Alert, AlertDescription, AlertTitle } from '@/components/ui/alert';
import { Badge } from '@/components/ui/badge';
import { Button } from '@/components/ui/button';
import {
  Card,
  CardContent,
  CardDescription,
  CardHeader,
  CardTitle,
} from '@/components/ui/card';
import { Input } from '@/components/ui/input';
import { Label } from '@/components/ui/label';
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from '@/components/ui/select';
import { Separator } from '@/components/ui/separator';
import { Tabs, TabsContent, TabsList, TabsTrigger } from '@/components/ui/Tabs';
import { Textarea } from '@/components/ui/textarea';
import { apiFetch } from '@/lib/api-fetch';
import type { GroupMembership, Web3Permission } from '@/shared/types/web3-auth';
import { createWeb3AdminClient } from '@/shared/utils/web3-api-client';

interface WalletUserData {
  wallet_address: string;
  user_id?: string;
  permissions: Web3Permission[];
  groups: GroupMembership[];
  created_at?: string;
  last_active?: string;
  is_active: boolean;
}

interface PermissionAssignment {
  permission: string;
  expires_at?: number;
  granted_by?: string;
  notes?: string;
}

const COMMON_PERMISSIONS = [
  'epsx:analytics:view',
  'epsx:analytics:advanced',
  'epsx:realtime:access',
  'epsx:analytics:export',
  'epsx:payment:manage',
  'admin:users:view',
  'admin:users:manage',
  'admin:analytics:view',
  'admin:system:manage',
  'admin:*:*',
];

const PERMISSION_GROUPS = [
  { id: 'basic', name: 'Basic User', description: 'Basic analytics access' },
  {
    id: 'premium',
    name: 'Premium User',
    description: 'Advanced analytics and exports',
  },
  { id: 'admin', name: 'Administrator', description: 'Full system access' },
  { id: 'analyst', name: 'Analyst', description: 'Advanced analytics tools' },
  {
    id: 'developer',
    name: 'Developer',
    description: 'API and development access',
  },
];

/**
 *
 */
export function WalletUserManagement() {
  const [walletAddress, setWalletAddress] = useState('');
  const [userData, setUserData] = useState<WalletUserData | null>(null);
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [success, setSuccess] = useState<string | null>(null);
  const [newPermission, setNewPermission] = useState<PermissionAssignment>({
    permission: '',
    notes: '',
  });
  const [selectedGroup, setSelectedGroup] = useState('');
  const [activeTab, setActiveTab] = useState('overview');

  const apiClient = createWeb3AdminClient({ serverSide: false });

  const validateWalletAddress = (address: string): boolean => {
    return /^0x[a-fA-F0-9]{40}$/.test(address);
  };

  const clearMessages = useCallback(() => {
    setError(null);
    setSuccess(null);
  }, []);

  const lookupWallet = useCallback(async () => {
    if (!walletAddress || !validateWalletAddress(walletAddress)) {
      setError('Please enter a valid wallet address (0x...)');
      return;
    }

    setLoading(true);
    clearMessages();

    try {
      // Get user permissions and data
      const permissionsData = await apiFetch(
        `/api/auth/web3/permissions?wallet_address=${walletAddress}`
      );

      // Construct user data from response
      const walletUserData: WalletUserData = {
        wallet_address: walletAddress,
        user_id: permissionsData.user_id,
        permissions: permissionsData.permissions || [],
        groups: permissionsData.groups || [],
        is_active: permissionsData.has_access || false,
        last_active: permissionsData.last_active,
        created_at: permissionsData.created_at,
      };

      setUserData(walletUserData);
      setSuccess('Wallet data loaded successfully');
    } catch (err) {
      setError(err instanceof Error ? err.message : 'Failed to lookup wallet');
      setUserData(null);
    } finally {
      setLoading(false);
    }
  }, [walletAddress, clearMessages]);

  const grantPermission = useCallback(async () => {
    if (!userData || !newPermission.permission) {
      setError('Please select a permission to grant');
      return;
    }

    setLoading(true);
    clearMessages();

    try {
      const payload = {
        wallet_address: userData.wallet_address,
        permission: newPermission.permission,
        expires_at: newPermission.expires_at,
        notes: newPermission.notes,
      };

      await apiFetch('/api/admin/permissions/grant', {
        method: 'POST',
        body: JSON.stringify(payload),
      });

      setSuccess(
        `Permission "${newPermission.permission}" granted successfully`
      );
      setNewPermission({ permission: '', notes: '' });

      // Refresh user data
      await lookupWallet();
    } catch (err) {
      setError(
        err instanceof Error ? err.message : 'Failed to grant permission'
      );
    } finally {
      setLoading(false);
    }
  }, [userData, newPermission, clearMessages, lookupWallet]);

  const revokePermission = useCallback(
    async (permission: string) => {
      if (!userData) {
        return;
      }

      setLoading(true);
      clearMessages();

      try {
        await apiFetch('/api/admin/permissions/revoke', {
          method: 'POST',
          body: JSON.stringify({
            wallet_address: userData.wallet_address,
            permission,
          }),
        });

        setSuccess(`Permission "${permission}" revoked successfully`);

        // Refresh user data
        await lookupWallet();
      } catch (err) {
        setError(
          err instanceof Error ? err.message : 'Failed to revoke permission'
        );
      } finally {
        setLoading(false);
      }
    },
    [userData, clearMessages, lookupWallet]
  );

  const assignToGroup = useCallback(async () => {
    if (!userData || !selectedGroup) {
      setError('Please select a group to assign');
      return;
    }

    setLoading(true);
    clearMessages();

    try {
      await apiFetch('/api/admin/groups/assign', {
        method: 'POST',
        body: JSON.stringify({
          wallet_address: userData.wallet_address,
          group_id: selectedGroup,
        }),
      });

      setSuccess(`User assigned to group "${selectedGroup}" successfully`);
      setSelectedGroup('');

      // Refresh user data
      await lookupWallet();
    } catch (err) {
      setError(
        err instanceof Error ? err.message : 'Failed to assign to group'
      );
    } finally {
      setLoading(false);
    }
  }, [userData, selectedGroup, clearMessages, lookupWallet]);

  const formatTimestamp = (timestamp?: string | number) => {
    if (!timestamp) {
      return 'Never';
    }
    const date = new Date(
      typeof timestamp === 'string' ? timestamp : timestamp * 1000
    );
    return date.toLocaleString();
  };

  const isPermissionExpired = (permission: Web3Permission) => {
    if (!permission.expires_at) {
      return false;
    }
    return Date.now() > new Date(permission.expires_at).getTime();
  };

  return (
    <div className="mx-auto w-full max-w-6xl space-y-6">
      {/* Wallet Lookup */}
      <Card className="border-2 border-purple-300/50 bg-white/80 shadow-xl backdrop-blur-sm dark:border-purple-700/50 dark:bg-gray-800/80">
        <CardHeader>
          <CardTitle className="flex items-center gap-3 text-xl">
            <div className="text-2xl">🔎</div>
            <span className="bg-gradient-to-r from-purple-600 to-pink-600 bg-clip-text font-bold text-transparent">
              Wallet User Lookup
            </span>
          </CardTitle>
          <CardDescription className="text-base text-gray-600 dark:text-gray-400">
            Enter a wallet address to view and manage user permissions
          </CardDescription>
        </CardHeader>
        <CardContent>
          <div className="flex gap-4">
            <div className="flex-1">
              <Label htmlFor="wallet-address" className="text-sm font-medium">
                Wallet Address
              </Label>
              <Input
                id="wallet-address"
                placeholder="0x7877e415a13532d9E43Df7Fd2CC256f93a39ced7"
                value={walletAddress}
                onChange={e => setWalletAddress(e.target.value)}
                className="mt-1.5 h-10 font-mono"
              />
            </div>
            <div className="flex items-end">
              <Button
                onClick={lookupWallet}
                disabled={loading || !walletAddress}
                className="h-10 border-0 bg-gradient-to-r from-purple-500 to-pink-500 px-6 text-white hover:from-purple-600 hover:to-pink-600"
              >
                {loading ? 'Looking up...' : 'Lookup Wallet'}
              </Button>
            </div>
          </div>

          {/* Messages */}
          {error && (
            <Alert className="mt-4 border-red-200 bg-red-50">
              <AlertTitle>Error</AlertTitle>
              <AlertDescription>{error}</AlertDescription>
            </Alert>
          )}

          {success && (
            <Alert className="mt-4 border-green-200 bg-green-50">
              <AlertTitle>Success</AlertTitle>
              <AlertDescription>{success}</AlertDescription>
            </Alert>
          )}
        </CardContent>
      </Card>

      {/* User Data Display */}
      {userData && (
        <Tabs value={activeTab} onValueChange={setActiveTab} className="w-full">
          <TabsList className="grid w-full grid-cols-4 border-2 border-indigo-300/50 bg-white/80 p-1 backdrop-blur-sm dark:border-indigo-700/50 dark:bg-gray-800/80">
            <TabsTrigger
              value="overview"
              className="data-[state=active]:bg-gradient-to-r data-[state=active]:from-indigo-400 data-[state=active]:to-purple-500 data-[state=active]:text-white"
            >
              Overview
            </TabsTrigger>
            <TabsTrigger
              value="permissions"
              className="data-[state=active]:bg-gradient-to-r data-[state=active]:from-green-400 data-[state=active]:to-emerald-500 data-[state=active]:text-white"
            >
              Permissions
            </TabsTrigger>
            <TabsTrigger
              value="groups"
              className="data-[state=active]:bg-gradient-to-r data-[state=active]:from-blue-400 data-[state=active]:to-cyan-500 data-[state=active]:text-white"
            >
              Groups
            </TabsTrigger>
            <TabsTrigger
              value="actions"
              className="data-[state=active]:bg-gradient-to-r data-[state=active]:from-orange-400 data-[state=active]:to-red-500 data-[state=active]:text-white"
            >
              Actions
            </TabsTrigger>
          </TabsList>

          {/* Overview Tab */}
          <TabsContent value="overview">
            <Card className="border-2 border-indigo-300/50 bg-white/80 shadow-xl backdrop-blur-sm dark:border-indigo-700/50 dark:bg-gray-800/80">
              <CardHeader>
                <CardTitle className="flex items-center gap-3 text-xl">
                  <div className="text-2xl">👤</div>
                  <span className="bg-gradient-to-r from-indigo-600 to-purple-600 bg-clip-text font-bold text-transparent">
                    User Overview
                  </span>
                </CardTitle>
                <CardDescription className="text-base text-gray-600 dark:text-gray-400">
                  Basic information about the wallet user
                </CardDescription>
              </CardHeader>
              <CardContent className="space-y-4">
                <div className="grid grid-cols-2 gap-4">
                  <div>
                    <Label>Wallet Address</Label>
                    <p className="rounded bg-gray-100 p-2 font-mono text-sm">
                      {userData.wallet_address}
                    </p>
                  </div>
                  <div>
                    <Label>User ID</Label>
                    <p className="text-sm">
                      {userData.user_id || 'Not assigned'}
                    </p>
                  </div>
                  <div>
                    <Label>Status</Label>
                    <Badge
                      className={
                        userData.is_active
                          ? 'border-0 bg-gradient-to-r from-green-400 to-emerald-500 text-white'
                          : 'border-0 bg-gray-200 text-gray-600 dark:bg-gray-700 dark:text-gray-400'
                      }
                    >
                      {userData.is_active ? '🟢 Active' : '⚪ Inactive'}
                    </Badge>
                  </div>
                  <div>
                    <Label>Created</Label>
                    <p className="text-sm">
                      {formatTimestamp(userData.created_at)}
                    </p>
                  </div>
                  <div>
                    <Label>Last Active</Label>
                    <p className="text-sm">
                      {formatTimestamp(userData.last_active)}
                    </p>
                  </div>
                </div>

                <Separator />

                <div>
                  <Label>Permission Count</Label>
                  <div className="mt-2 flex gap-2">
                    <Badge variant="outline">
                      Total: {userData.permissions.length}
                    </Badge>
                    <Badge variant="outline">
                      Expired:{' '}
                      {userData.permissions.filter(isPermissionExpired).length}
                    </Badge>
                    <Badge variant="outline">
                      Groups: {userData.groups.length}
                    </Badge>
                  </div>
                </div>
              </CardContent>
            </Card>
          </TabsContent>

          {/* Permissions Tab */}
          <TabsContent value="permissions">
            <div className="space-y-4">
              {/* Current Permissions */}
              <Card className="border-2 border-green-300/50 bg-white/80 shadow-xl backdrop-blur-sm dark:border-green-700/50 dark:bg-gray-800/80">
                <CardHeader>
                  <CardTitle className="flex items-center gap-3 text-xl">
                    <div className="text-2xl">✅</div>
                    <span className="bg-gradient-to-r from-green-600 to-emerald-600 bg-clip-text font-bold text-transparent">
                      Current Permissions
                    </span>
                  </CardTitle>
                  <CardDescription className="text-base text-gray-600 dark:text-gray-400">
                    Active permissions for this wallet
                  </CardDescription>
                </CardHeader>
                <CardContent>
                  {userData.permissions.length === 0 ? (
                    <p className="text-gray-500">No permissions assigned</p>
                  ) : (
                    <div className="space-y-2">
                      {userData.permissions.map((permission, index) => (
                        <div
                          key={index}
                          className="relative overflow-hidden rounded-xl bg-gradient-to-r from-green-400/10 via-emerald-400/10 to-teal-400/10 p-0.5"
                        >
                          <div className="flex items-center justify-between rounded-xl bg-white/95 p-3 backdrop-blur-xl dark:bg-gray-900/95">
                            <div className="flex-1">
                              <p className="text-sm font-semibold">
                                {permission.permission}
                              </p>
                              {permission.expires_at && (
                                <p className="mt-1 text-xs text-gray-500 dark:text-gray-400">
                                  Expires:{' '}
                                  {formatTimestamp(permission.expires_at)}
                                  {isPermissionExpired(permission) && (
                                    <span className="ml-2 text-red-500">
                                      (Expired)
                                    </span>
                                  )}
                                </p>
                              )}
                            </div>
                            <Button
                              variant="outline"
                              size="sm"
                              onClick={() =>
                                revokePermission(permission.permission)
                              }
                              disabled={loading}
                              className="border-2 border-red-300 hover:bg-red-50 dark:border-red-600 dark:hover:bg-red-950"
                            >
                              Revoke
                            </Button>
                          </div>
                        </div>
                      ))}
                    </div>
                  )}
                </CardContent>
              </Card>

              {/* Grant New Permission */}
              <Card className="border-2 border-blue-300/50 bg-white/80 shadow-xl backdrop-blur-sm dark:border-blue-700/50 dark:bg-gray-800/80">
                <CardHeader>
                  <CardTitle className="flex items-center gap-3 text-xl">
                    <div className="text-2xl">➕</div>
                    <span className="bg-gradient-to-r from-blue-600 to-cyan-600 bg-clip-text font-bold text-transparent">
                      Grant Permission
                    </span>
                  </CardTitle>
                  <CardDescription className="text-base text-gray-600 dark:text-gray-400">
                    Assign a new permission to this wallet
                  </CardDescription>
                </CardHeader>
                <CardContent className="space-y-4">
                  <div>
                    <Label>Permission</Label>
                    <Select
                      value={newPermission.permission}
                      onValueChange={value =>
                        setNewPermission(prev => ({
                          ...prev,
                          permission: value,
                        }))
                      }
                    >
                      <SelectTrigger>
                        <SelectValue placeholder="Select permission" />
                      </SelectTrigger>
                      <SelectContent>
                        {COMMON_PERMISSIONS.map(permission => (
                          <SelectItem key={permission} value={permission}>
                            {permission}
                          </SelectItem>
                        ))}
                      </SelectContent>
                    </Select>
                  </div>

                  <div>
                    <Label>Expiry Date (Optional)</Label>
                    <Input
                      type="datetime-local"
                      onChange={e => {
                        const timestamp = e.target.value
                          ? new Date(e.target.value).getTime() / 1000
                          : undefined;
                        setNewPermission(prev => ({
                          ...prev,
                          expires_at: timestamp,
                        }));
                      }}
                    />
                  </div>

                  <div>
                    <Label>Notes (Optional)</Label>
                    <Textarea
                      placeholder="Add notes about this permission grant..."
                      value={newPermission.notes}
                      onChange={e =>
                        setNewPermission(prev => ({
                          ...prev,
                          notes: e.target.value,
                        }))
                      }
                    />
                  </div>

                  <Button
                    onClick={grantPermission}
                    disabled={loading || !newPermission.permission}
                  >
                    Grant Permission
                  </Button>
                </CardContent>
              </Card>
            </div>
          </TabsContent>

          {/* Groups Tab */}
          <TabsContent value="groups">
            <div className="space-y-4">
              {/* Current Groups */}
              <Card>
                <CardHeader>
                  <CardTitle>Group Memberships</CardTitle>
                  <CardDescription>
                    Permission groups this wallet belongs to
                  </CardDescription>
                </CardHeader>
                <CardContent>
                  {userData.groups.length === 0 ? (
                    <p className="text-gray-500">Not assigned to any groups</p>
                  ) : (
                    <div className="space-y-2">
                      {userData.groups.map((group, index) => (
                        <div
                          key={index}
                          className="flex items-center justify-between rounded border p-3"
                        >
                          <div>
                            <p className="font-medium">{group.group_name}</p>
                            <p className="text-sm text-gray-500">
                              Type: {group.group_type}
                            </p>
                          </div>
                          <Badge variant="secondary">
                            {group.assignment_source || 'Manual'}
                          </Badge>
                        </div>
                      ))}
                    </div>
                  )}
                </CardContent>
              </Card>

              {/* Assign to Group */}
              <Card>
                <CardHeader>
                  <CardTitle>Assign to Group</CardTitle>
                  <CardDescription>
                    Add this wallet to a permission group
                  </CardDescription>
                </CardHeader>
                <CardContent className="space-y-4">
                  <div>
                    <Label>Permission Group</Label>
                    <Select
                      value={selectedGroup}
                      onValueChange={setSelectedGroup}
                    >
                      <SelectTrigger>
                        <SelectValue placeholder="Select group" />
                      </SelectTrigger>
                      <SelectContent>
                        {PERMISSION_GROUPS.map(group => (
                          <SelectItem key={group.id} value={group.id}>
                            <div>
                              <p className="font-medium">{group.name}</p>
                              <p className="text-sm text-gray-500">
                                {group.description}
                              </p>
                            </div>
                          </SelectItem>
                        ))}
                      </SelectContent>
                    </Select>
                  </div>

                  <Button
                    onClick={assignToGroup}
                    disabled={loading || !selectedGroup}
                  >
                    Assign to Group
                  </Button>
                </CardContent>
              </Card>
            </div>
          </TabsContent>

          {/* Actions Tab */}
          <TabsContent value="actions">
            <Card>
              <CardHeader>
                <CardTitle>Administrative Actions</CardTitle>
                <CardDescription>
                  Advanced management options for this wallet
                </CardDescription>
              </CardHeader>
              <CardContent className="space-y-4">
                <div className="grid grid-cols-2 gap-4">
                  <Button variant="outline">View Activity History</Button>
                  <Button variant="outline">Export Permissions</Button>
                  <Button variant="outline">Reset Permissions</Button>
                  <Button variant="destructive">Deactivate Wallet</Button>
                </div>

                <Separator />

                <div className="text-sm text-gray-600">
                  <p>
                    <strong>Note:</strong> Administrative actions are logged and
                    may require additional confirmation.
                  </p>
                </div>
              </CardContent>
            </Card>
          </TabsContent>
        </Tabs>
      )}
    </div>
  );
}
