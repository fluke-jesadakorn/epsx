'use client';

import { useState, useEffect, useCallback } from 'react';
import { useAccount } from 'wagmi';
import { useQuery } from '@tanstack/react-query';
import { 
  Shield, 
  Clock, 
  User, 
  AlertCircle, 
  RefreshCw, 
  CheckCircle,
  ExternalLink,
  Filter,
  Search,
  Wallet
} from 'lucide-react';
import { Card, CardContent, CardHeader, CardTitle } from '@/components/ui/card';
import { Button } from '@/components/ui/button';
import { Input } from '@/components/ui/input';
import { Badge } from '@/components/ui/badge';
import { Tabs, TabsContent, TabsList, TabsTrigger } from '@/components/ui/tabs';
import { format, isAfter, parseISO } from 'date-fns';

interface WalletPermission {
  permission: string;
  source: 'manual' | 'nft' | 'token' | 'dao';
  expires_at?: string;
  metadata?: {
    grant_reason?: string;
    granted_by?: string;
    nft_contract?: string;
    nft_token_id?: string;
    token_contract?: string;
    token_amount_required?: string;
    dao_proposal_id?: string;
    dao_vote_count?: number;
  };
  granted_at: string;
  health_score?: number;
}

interface PermissionHealth {
  total_permissions: number;
  active_permissions: number;
  expiring_soon: number;
  health_score: number;
  last_updated: string;
}

interface WalletPermissionsProps {
  className?: string;
  showHealthScore?: boolean;
  autoRefresh?: boolean;
  refreshInterval?: number;
}

export function WalletPermissions({ 
  className = '',
  showHealthScore = true,
  autoRefresh = true,
  refreshInterval = 30000 // 30 seconds
}: WalletPermissionsProps) {
  const { address, isConnected } = useAccount();
  const [searchTerm, setSearchTerm] = useState('');
  const [filterSource, setFilterSource] = useState<string>('all');

  // Fetch wallet permissions
  const { 
    data: permissionsData, 
    isLoading, 
    error, 
    refetch 
  } = useQuery({
    queryKey: ['wallet-permissions', address],
    queryFn: async () => {
      if (!address) throw new Error('No wallet connected');
      
      const response = await fetch('/api/auth/web3/permissions', {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({ wallet_address: address }),
        credentials: 'include',
      });

      if (!response.ok) {
        throw new Error('Failed to fetch permissions');
      }

      return response.json();
    },
    enabled: !!address && isConnected,
    refetchInterval: autoRefresh ? refreshInterval : false,
    staleTime: 30000,
  });

  const permissions: WalletPermission[] = permissionsData?.permissions || [];
  const health: PermissionHealth = permissionsData?.health || {
    total_permissions: 0,
    active_permissions: 0,
    expiring_soon: 0,
    health_score: 0,
    last_updated: new Date().toISOString(),
  };

  // Filter and search permissions
  const filteredPermissions = permissions.filter(permission => {
    const matchesSearch = permission.permission.toLowerCase().includes(searchTerm.toLowerCase()) ||
                         permission.source.toLowerCase().includes(searchTerm.toLowerCase());
    const matchesFilter = filterSource === 'all' || permission.source === filterSource;
    return matchesSearch && matchesFilter;
  });

  // Group permissions by source
  const groupedPermissions = {
    manual: filteredPermissions.filter(p => p.source === 'manual'),
    nft: filteredPermissions.filter(p => p.source === 'nft'),
    token: filteredPermissions.filter(p => p.source === 'token'),
    dao: filteredPermissions.filter(p => p.source === 'dao'),
  };

  const getSourceIcon = (source: string) => {
    switch (source) {
      case 'nft': return '🎨';
      case 'token': return '🪙';
      case 'dao': return '🗳️';
      default: return '👤';
    }
  };

  const getSourceColor = (source: string) => {
    switch (source) {
      case 'nft': return 'bg-purple-50 text-purple-700 border-purple-200 dark:bg-purple-900/20 dark:text-purple-300 dark:border-purple-700';
      case 'token': return 'bg-blue-50 text-blue-700 border-blue-200 dark:bg-blue-900/20 dark:text-blue-300 dark:border-blue-700';
      case 'dao': return 'bg-green-50 text-green-700 border-green-200 dark:bg-green-900/20 dark:text-green-300 dark:border-green-700';
      default: return 'bg-gray-50 text-gray-700 border-gray-200 dark:bg-gray-900/20 dark:text-gray-300 dark:border-gray-700';
    }
  };

  const isExpiringSoon = (expiresAt?: string) => {
    if (!expiresAt) return false;
    const expiryDate = parseISO(expiresAt);
    const thirtyDaysFromNow = new Date();
    thirtyDaysFromNow.setDate(thirtyDaysFromNow.getDate() + 30);
    return isAfter(expiryDate, new Date()) && !isAfter(expiryDate, thirtyDaysFromNow);
  };

  const isExpired = (expiresAt?: string) => {
    if (!expiresAt) return false;
    return !isAfter(parseISO(expiresAt), new Date());
  };

  const getHealthColor = (score: number) => {
    if (score >= 80) return 'text-green-600 dark:text-green-400';
    if (score >= 60) return 'text-yellow-600 dark:text-yellow-400';
    return 'text-red-600 dark:text-red-400';
  };

  if (!isConnected) {
    return (
      <Card className={className}>
        <CardContent className="flex items-center justify-center p-6">
          <div className="text-center">
            <Wallet className="h-12 w-12 text-gray-400 mx-auto mb-4" />
            <p className="text-gray-600 dark:text-gray-400">
              Connect your wallet to view permissions
            </p>
          </div>
        </CardContent>
      </Card>
    );
  }

  if (error) {
    return (
      <Card className={className}>
        <CardContent className="flex items-center justify-center p-6">
          <div className="text-center">
            <AlertCircle className="h-12 w-12 text-red-400 mx-auto mb-4" />
            <p className="text-red-600 dark:text-red-400 mb-4">
              Failed to load permissions
            </p>
            <Button onClick={() => refetch()} variant="outline" size="sm">
              <RefreshCw className="h-4 w-4 mr-2" />
              Retry
            </Button>
          </div>
        </CardContent>
      </Card>
    );
  }

  return (
    <Card className={className}>
      <CardHeader>
        <div className="flex items-center justify-between">
          <CardTitle className="flex items-center gap-2">
            <Shield className="h-5 w-5 text-blue-500" />
            Wallet Permissions
          </CardTitle>
          <div className="flex items-center gap-2">
            {showHealthScore && (
              <Badge variant="outline" className={getHealthColor(health.health_score)}>
                Health: {health.health_score}%
              </Badge>
            )}
            <Button 
              onClick={() => refetch()} 
              variant="ghost" 
              size="sm"
              disabled={isLoading}
            >
              <RefreshCw className={`h-4 w-4 ${isLoading ? 'animate-spin' : ''}`} />
            </Button>
          </div>
        </div>

        {/* Health Summary */}
        {showHealthScore && (
          <div className="grid grid-cols-4 gap-4 mt-4">
            <div className="text-center">
              <div className="text-2xl font-bold text-blue-600 dark:text-blue-400">
                {health.total_permissions}
              </div>
              <div className="text-sm text-gray-600 dark:text-gray-400">Total</div>
            </div>
            <div className="text-center">
              <div className="text-2xl font-bold text-green-600 dark:text-green-400">
                {health.active_permissions}
              </div>
              <div className="text-sm text-gray-600 dark:text-gray-400">Active</div>
            </div>
            <div className="text-center">
              <div className="text-2xl font-bold text-yellow-600 dark:text-yellow-400">
                {health.expiring_soon}
              </div>
              <div className="text-sm text-gray-600 dark:text-gray-400">Expiring</div>
            </div>
            <div className="text-center">
              <div className={`text-2xl font-bold ${getHealthColor(health.health_score)}`}>
                {health.health_score}%
              </div>
              <div className="text-sm text-gray-600 dark:text-gray-400">Health</div>
            </div>
          </div>
        )}
      </CardHeader>

      <CardContent>
        {/* Search and Filter */}
        <div className="flex gap-4 mb-6">
          <div className="relative flex-1">
            <Search className="absolute left-3 top-1/2 transform -translate-y-1/2 text-gray-400 h-4 w-4" />
            <Input
              placeholder="Search permissions..."
              value={searchTerm}
              onChange={(e) => setSearchTerm(e.target.value)}
              className="pl-10"
            />
          </div>
          <select
            value={filterSource}
            onChange={(e) => setFilterSource(e.target.value)}
            className="px-3 py-2 border border-gray-200 rounded-md text-sm dark:border-gray-700 dark:bg-gray-800"
          >
            <option value="all">All Sources</option>
            <option value="manual">👤 Manual</option>
            <option value="nft">🎨 NFT</option>
            <option value="token">🪙 Token</option>
            <option value="dao">🗳️ DAO</option>
          </select>
        </div>

        {/* Permissions Tabs */}
        <Tabs defaultValue="all" className="w-full">
          <TabsList className="grid w-full grid-cols-5">
            <TabsTrigger value="all">All ({filteredPermissions.length})</TabsTrigger>
            <TabsTrigger value="manual">👤 {groupedPermissions.manual.length}</TabsTrigger>
            <TabsTrigger value="nft">🎨 {groupedPermissions.nft.length}</TabsTrigger>
            <TabsTrigger value="token">🪙 {groupedPermissions.token.length}</TabsTrigger>
            <TabsTrigger value="dao">🗳️ {groupedPermissions.dao.length}</TabsTrigger>
          </TabsList>

          <TabsContent value="all" className="mt-4">
            <PermissionList permissions={filteredPermissions} />
          </TabsContent>
          
          <TabsContent value="manual" className="mt-4">
            <PermissionList permissions={groupedPermissions.manual} />
          </TabsContent>
          
          <TabsContent value="nft" className="mt-4">
            <PermissionList permissions={groupedPermissions.nft} />
          </TabsContent>
          
          <TabsContent value="token" className="mt-4">
            <PermissionList permissions={groupedPermissions.token} />
          </TabsContent>
          
          <TabsContent value="dao" className="mt-4">
            <PermissionList permissions={groupedPermissions.dao} />
          </TabsContent>
        </Tabs>
      </CardContent>
    </Card>
  );
}

function PermissionList({ permissions }: { permissions: WalletPermission[] }) {
  if (permissions.length === 0) {
    return (
      <div className="text-center py-8">
        <Shield className="h-12 w-12 text-gray-400 mx-auto mb-4" />
        <p className="text-gray-600 dark:text-gray-400">No permissions found</p>
      </div>
    );
  }

  return (
    <div className="space-y-4">
      {permissions.map((permission, index) => (
        <PermissionCard key={index} permission={permission} />
      ))}
    </div>
  );
}

function PermissionCard({ permission }: { permission: WalletPermission }) {
  const isExpiringSoon = (expiresAt?: string) => {
    if (!expiresAt) return false;
    const expiryDate = parseISO(expiresAt);
    const thirtyDaysFromNow = new Date();
    thirtyDaysFromNow.setDate(thirtyDaysFromNow.getDate() + 30);
    return isAfter(expiryDate, new Date()) && !isAfter(expiryDate, thirtyDaysFromNow);
  };

  const isExpired = (expiresAt?: string) => {
    if (!expiresAt) return false;
    return !isAfter(parseISO(expiresAt), new Date());
  };

  const getSourceColor = (source: string) => {
    switch (source) {
      case 'nft': return 'bg-purple-50 text-purple-700 border-purple-200 dark:bg-purple-900/20 dark:text-purple-300 dark:border-purple-700';
      case 'token': return 'bg-blue-50 text-blue-700 border-blue-200 dark:bg-blue-900/20 dark:text-blue-300 dark:border-blue-700';
      case 'dao': return 'bg-green-50 text-green-700 border-green-200 dark:bg-green-900/20 dark:text-green-300 dark:border-green-700';
      default: return 'bg-gray-50 text-gray-700 border-gray-200 dark:bg-gray-900/20 dark:text-gray-300 dark:border-gray-700';
    }
  };

  const getSourceIcon = (source: string) => {
    switch (source) {
      case 'nft': return '🎨';
      case 'token': return '🪙';
      case 'dao': return '🗳️';
      default: return '👤';
    }
  };

  return (
    <div className={`border rounded-lg p-4 ${isExpired(permission.expires_at) ? 'opacity-50 bg-gray-50 dark:bg-gray-900/50' : 'bg-white dark:bg-gray-800'}`}>
      <div className="flex items-start justify-between">
        <div className="flex-1">
          <div className="flex items-center gap-2 mb-2">
            <Badge className={getSourceColor(permission.source)}>
              {getSourceIcon(permission.source)} {permission.source}
            </Badge>
            {permission.expires_at && (
              <Badge variant={isExpired(permission.expires_at) ? 'destructive' : isExpiringSoon(permission.expires_at) ? 'secondary' : 'outline'}>
                <Clock className="h-3 w-3 mr-1" />
                {isExpired(permission.expires_at) ? 'Expired' : isExpiringSoon(permission.expires_at) ? 'Expiring Soon' : 'Active'}
              </Badge>
            )}
          </div>
          
          <div className="font-medium text-gray-900 dark:text-gray-100 mb-1">
            {permission.permission}
          </div>
          
          {permission.metadata?.grant_reason && (
            <div className="text-sm text-gray-600 dark:text-gray-400 mb-2">
              {permission.metadata.grant_reason}
            </div>
          )}
          
          <div className="text-xs text-gray-500 dark:text-gray-500">
            Granted: {format(parseISO(permission.granted_at), 'MMM dd, yyyy')}
            {permission.expires_at && (
              <span className="ml-4">
                Expires: {format(parseISO(permission.expires_at), 'MMM dd, yyyy')}
              </span>
            )}
          </div>
        </div>
        
        <div className="ml-4">
          {permission.metadata?.nft_contract && (
            <Button variant="ghost" size="sm" asChild>
              <a 
                href={`https://etherscan.io/address/${permission.metadata.nft_contract}`}
                target="_blank"
                rel="noopener noreferrer"
                title="View NFT Contract"
              >
                <ExternalLink className="h-4 w-4" />
              </a>
            </Button>
          )}
        </div>
      </div>
    </div>
  );
}