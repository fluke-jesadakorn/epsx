'use client';

import { useState, useEffect } from 'react';
import { useAccount } from 'wagmi';
import { useQuery, useMutation, useQueryClient } from '@tanstack/react-query';
import { 
  Shield, 
  Plus, 
  Edit, 
  Trash2, 
  Users, 
  Clock, 
  AlertTriangle,
  CheckCircle,
  Search,
  Filter,
  Download,
  Upload,
  RefreshCw,
  Settings,
  Crown,
  Wallet,
  ExternalLink
} from 'lucide-react';
import { Card, CardContent, CardHeader, CardTitle } from '@/components/ui/card';
import { Button } from '@/components/ui/button';
import { Input } from '@/components/ui/input';
import { Badge } from '@/components/ui/badge';
import { Tabs, TabsContent, TabsList, TabsTrigger } from '@/components/ui/tabs';
import { Dialog, DialogContent, DialogHeader, DialogTitle, DialogTrigger } from '@/components/ui/dialog';
import { Label } from '@/components/ui/label';
import { Textarea } from '@/components/ui/textarea';
import { Switch } from '@/components/ui/switch';
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from '@/components/ui/select';
import { toast } from 'react-hot-toast';
import { format, parseISO, addDays, addMonths, addYears } from 'date-fns';

interface WalletPermission {
  id: string;
  wallet_address: string;
  permission: string;
  source: 'manual' | 'nft' | 'token' | 'dao';
  expires_at?: string;
  granted_at: string;
  granted_by: string;
  metadata?: {
    grant_reason?: string;
    nft_contract?: string;
    nft_token_id?: string;
    token_contract?: string;
    token_amount_required?: string;
    dao_proposal_id?: string;
    dao_vote_count?: number;
  };
  is_active: boolean;
}

interface NFTGateConfig {
  id: string;
  contract_address: string;
  contract_name: string;
  permissions: string[];
  min_token_count: number;
  is_active: boolean;
  created_at: string;
}

interface TokenGateConfig {
  id: string;
  contract_address: string;
  token_symbol: string;
  permissions: string[];
  min_amount: string;
  is_active: boolean;
  created_at: string;
}

interface DAOProposal {
  id: string;
  title: string;
  description: string;
  target_wallet: string;
  permissions: string[];
  votes_for: number;
  votes_against: number;
  votes_required: number;
  status: 'pending' | 'approved' | 'rejected' | 'executed';
  created_at: string;
  expires_at: string;
}

export function Web3PermissionManager() {
  const { address } = useAccount();
  const queryClient = useQueryClient();
  const [searchTerm, setSearchTerm] = useState('');
  const [filterStatus, setFilterStatus] = useState('all');
  const [selectedPermissions, setSelectedPermissions] = useState<string[]>([]);

  // Manual permission form state
  const [manualForm, setManualForm] = useState({
    wallet_address: '',
    permissions: [] as string[],
    expires_in: '90', // days
    grant_reason: '',
  });

  // NFT gate form state
  const [nftForm, setNftForm] = useState({
    contract_address: '',
    contract_name: '',
    permissions: [] as string[],
    min_token_count: 1,
  });

  // Token gate form state
  const [tokenForm, setTokenForm] = useState({
    contract_address: '',
    token_symbol: '',
    permissions: [] as string[],
    min_amount: '',
  });

  // DAO proposal form state
  const [daoForm, setDaoForm] = useState({
    title: '',
    description: '',
    target_wallet: '',
    permissions: [] as string[],
    votes_required: 3,
    expires_in: '7', // days
  });

  // Available permissions
  const availablePermissions = [
    'epsx:analytics:view',
    'epsx:analytics:advanced',
    'epsx:trading:basic',
    'epsx:trading:advanced',
    'epsx:trading:pro',
    'epsx:data:export',
    'epsx:api:read',
    'epsx:api:write',
    'epsx:notifications:manage',
    'admin:users:view',
    'admin:users:manage',
    'admin:permissions:view',
    'admin:permissions:manage',
    'admin:*:*',
  ];

  // Fetch all permissions
  const { data: permissionsData, isLoading: permissionsLoading } = useQuery({
    queryKey: ['admin-permissions'],
    queryFn: async () => {
      const response = await fetch('/api/admin/web3/permissions', {
        credentials: 'include',
      });
      if (!response.ok) throw new Error('Failed to fetch permissions');
      return response.json();
    },
    refetchInterval: 30000,
  });

  // Fetch NFT gate configs
  const { data: nftGatesData, isLoading: nftGatesLoading } = useQuery({
    queryKey: ['nft-gates'],
    queryFn: async () => {
      const response = await fetch('/api/admin/web3/nft-gates', {
        credentials: 'include',
      });
      if (!response.ok) throw new Error('Failed to fetch NFT gates');
      return response.json();
    },
  });

  // Fetch token gate configs
  const { data: tokenGatesData, isLoading: tokenGatesLoading } = useQuery({
    queryKey: ['token-gates'],
    queryFn: async () => {
      const response = await fetch('/api/admin/web3/token-gates', {
        credentials: 'include',
      });
      if (!response.ok) throw new Error('Failed to fetch token gates');
      return response.json();
    },
  });

  // Fetch DAO proposals
  const { data: daoProposalsData, isLoading: daoProposalsLoading } = useQuery({
    queryKey: ['dao-proposals'],
    queryFn: async () => {
      const response = await fetch('/api/admin/web3/dao-proposals', {
        credentials: 'include',
      });
      if (!response.ok) throw new Error('Failed to fetch DAO proposals');
      return response.json();
    },
  });

  // Grant manual permission mutation
  const grantManualPermission = useMutation({
    mutationFn: async (data: typeof manualForm) => {
      const response = await fetch('/api/admin/web3/permissions/grant', {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({
          wallet_address: data.wallet_address,
          permissions: data.permissions,
          expires_at: data.expires_in ? new Date(Date.now() + parseInt(data.expires_in) * 24 * 60 * 60 * 1000).toISOString() : null,
          grant_reason: data.grant_reason,
        }),
        credentials: 'include',
      });
      if (!response.ok) throw new Error('Failed to grant permission');
      return response.json();
    },
    onSuccess: () => {
      toast.success('Permission granted successfully');
      queryClient.invalidateQueries({ queryKey: ['admin-permissions'] });
      setManualForm({ wallet_address: '', permissions: [], expires_in: '90', grant_reason: '' });
    },
    onError: (error: any) => {
      toast.error(error.message || 'Failed to grant permission');
    },
  });

  // Create NFT gate mutation
  const createNFTGate = useMutation({
    mutationFn: async (data: typeof nftForm) => {
      const response = await fetch('/api/admin/web3/nft-gates', {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify(data),
        credentials: 'include',
      });
      if (!response.ok) throw new Error('Failed to create NFT gate');
      return response.json();
    },
    onSuccess: () => {
      toast.success('NFT gate created successfully');
      queryClient.invalidateQueries({ queryKey: ['nft-gates'] });
      setNftForm({ contract_address: '', contract_name: '', permissions: [], min_token_count: 1 });
    },
    onError: (error: any) => {
      toast.error(error.message || 'Failed to create NFT gate');
    },
  });

  // Create token gate mutation
  const createTokenGate = useMutation({
    mutationFn: async (data: typeof tokenForm) => {
      const response = await fetch('/api/admin/web3/token-gates', {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify(data),
        credentials: 'include',
      });
      if (!response.ok) throw new Error('Failed to create token gate');
      return response.json();
    },
    onSuccess: () => {
      toast.success('Token gate created successfully');
      queryClient.invalidateQueries({ queryKey: ['token-gates'] });
      setTokenForm({ contract_address: '', token_symbol: '', permissions: [], min_amount: '' });
    },
    onError: (error: any) => {
      toast.error(error.message || 'Failed to create token gate');
    },
  });

  // Create DAO proposal mutation
  const createDAOProposal = useMutation({
    mutationFn: async (data: typeof daoForm) => {
      const response = await fetch('/api/admin/web3/dao-proposals', {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({
          ...data,
          expires_at: new Date(Date.now() + parseInt(data.expires_in) * 24 * 60 * 60 * 1000).toISOString(),
        }),
        credentials: 'include',
      });
      if (!response.ok) throw new Error('Failed to create DAO proposal');
      return response.json();
    },
    onSuccess: () => {
      toast.success('DAO proposal created successfully');
      queryClient.invalidateQueries({ queryKey: ['dao-proposals'] });
      setDaoForm({ title: '', description: '', target_wallet: '', permissions: [], votes_required: 3, expires_in: '7' });
    },
    onError: (error: any) => {
      toast.error(error.message || 'Failed to create DAO proposal');
    },
  });

  const permissions: WalletPermission[] = permissionsData?.permissions || [];
  const nftGates: NFTGateConfig[] = nftGatesData?.nft_gates || [];
  const tokenGates: TokenGateConfig[] = tokenGatesData?.token_gates || [];
  const daoProposals: DAOProposal[] = daoProposalsData?.proposals || [];

  const filteredPermissions = permissions.filter(permission => {
    const matchesSearch = permission.wallet_address.toLowerCase().includes(searchTerm.toLowerCase()) ||
                         permission.permission.toLowerCase().includes(searchTerm.toLowerCase());
    const matchesFilter = filterStatus === 'all' || 
                         (filterStatus === 'active' && permission.is_active) ||
                         (filterStatus === 'expired' && !permission.is_active);
    return matchesSearch && matchesFilter;
  });

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

  return (
    <div className="space-y-6">
      {/* Header */}
      <Card>
        <CardHeader>
          <CardTitle className="flex items-center gap-2">
            <Crown className="h-6 w-6 text-yellow-500" />
            Web3 Permission Management
          </CardTitle>
          <p className="text-sm text-gray-600 dark:text-gray-400">
            Manage wallet permissions across all authentication methods: Manual, NFT, Token, and DAO governance
          </p>
        </CardHeader>
      </Card>

      {/* Management Tabs */}
      <Tabs defaultValue="permissions" className="w-full">
        <TabsList className="grid w-full grid-cols-5">
          <TabsTrigger value="permissions">Permissions ({permissions.length})</TabsTrigger>
          <TabsTrigger value="manual">👤 Manual</TabsTrigger>
          <TabsTrigger value="nft">🎨 NFT Gates ({nftGates.length})</TabsTrigger>
          <TabsTrigger value="token">🪙 Token Gates ({tokenGates.length})</TabsTrigger>
          <TabsTrigger value="dao">🗳️ DAO Proposals ({daoProposals.length})</TabsTrigger>
        </TabsList>

        {/* All Permissions Tab */}
        <TabsContent value="permissions" className="space-y-4">
          <Card>
            <CardHeader>
              <div className="flex items-center justify-between">
                <CardTitle className="flex items-center gap-2">
                  <Shield className="h-5 w-5 text-blue-500" />
                  All Wallet Permissions
                </CardTitle>
                <div className="flex items-center gap-2">
                  <Button variant="outline" size="sm">
                    <Download className="h-4 w-4 mr-2" />
                    Export
                  </Button>
                  <Button variant="outline" size="sm">
                    <RefreshCw className="h-4 w-4" />
                  </Button>
                </div>
              </div>
            </CardHeader>
            <CardContent>
              {/* Search and Filter */}
              <div className="flex gap-4 mb-6">
                <div className="relative flex-1">
                  <Search className="absolute left-3 top-1/2 transform -translate-y-1/2 text-gray-400 h-4 w-4" />
                  <Input
                    placeholder="Search wallet address or permission..."
                    value={searchTerm}
                    onChange={(e) => setSearchTerm(e.target.value)}
                    className="pl-10"
                  />
                </div>
                <Select value={filterStatus} onValueChange={setFilterStatus}>
                  <SelectTrigger className="w-48">
                    <SelectValue placeholder="Filter status" />
                  </SelectTrigger>
                  <SelectContent>
                    <SelectItem value="all">All Statuses</SelectItem>
                    <SelectItem value="active">Active</SelectItem>
                    <SelectItem value="expired">Expired</SelectItem>
                  </SelectContent>
                </Select>
              </div>

              {/* Permissions List */}
              <div className="space-y-4">
                {filteredPermissions.map((permission) => (
                  <div key={permission.id} className="border rounded-lg p-4">
                    <div className="flex items-start justify-between">
                      <div className="flex-1">
                        <div className="flex items-center gap-2 mb-2">
                          <Badge className={getSourceColor(permission.source)}>
                            {getSourceIcon(permission.source)} {permission.source}
                          </Badge>
                          <Badge variant={permission.is_active ? 'default' : 'secondary'}>
                            {permission.is_active ? 'Active' : 'Expired'}
                          </Badge>
                        </div>
                        
                        <div className="grid grid-cols-2 gap-4">
                          <div>
                            <div className="font-medium text-sm text-gray-700 dark:text-gray-300">Wallet Address</div>
                            <div className="font-mono text-sm">{permission.wallet_address}</div>
                          </div>
                          <div>
                            <div className="font-medium text-sm text-gray-700 dark:text-gray-300">Permission</div>
                            <div className="text-sm">{permission.permission}</div>
                          </div>
                          <div>
                            <div className="font-medium text-sm text-gray-700 dark:text-gray-300">Granted</div>
                            <div className="text-sm">{format(parseISO(permission.granted_at), 'MMM dd, yyyy')}</div>
                          </div>
                          {permission.expires_at && (
                            <div>
                              <div className="font-medium text-sm text-gray-700 dark:text-gray-300">Expires</div>
                              <div className="text-sm">{format(parseISO(permission.expires_at), 'MMM dd, yyyy')}</div>
                            </div>
                          )}
                        </div>
                        
                        {permission.metadata?.grant_reason && (
                          <div className="mt-2">
                            <div className="font-medium text-sm text-gray-700 dark:text-gray-300">Reason</div>
                            <div className="text-sm text-gray-600 dark:text-gray-400">{permission.metadata.grant_reason}</div>
                          </div>
                        )}
                      </div>
                      
                      <div className="flex items-center gap-2">
                        <Button variant="ghost" size="sm">
                          <Edit className="h-4 w-4" />
                        </Button>
                        <Button variant="ghost" size="sm">
                          <Trash2 className="h-4 w-4 text-red-500" />
                        </Button>
                      </div>
                    </div>
                  </div>
                ))}
              </div>
            </CardContent>
          </Card>
        </TabsContent>

        {/* Manual Permission Grant Tab */}
        <TabsContent value="manual" className="space-y-4">
          <Card>
            <CardHeader>
              <CardTitle className="flex items-center gap-2">
                <Users className="h-5 w-5 text-green-500" />
                Grant Manual Permissions
              </CardTitle>
            </CardHeader>
            <CardContent className="space-y-4">
              <div className="space-y-4">
                <div>
                  <Label htmlFor="wallet_address">Wallet Address</Label>
                  <Input
                    id="wallet_address"
                    placeholder="0x..."
                    value={manualForm.wallet_address}
                    onChange={(e) => setManualForm(prev => ({ ...prev, wallet_address: e.target.value }))}
                  />
                </div>

                <div>
                  <Label>Permissions</Label>
                  <Select 
                    value=""
                    onValueChange={(value) => {
                      if (!manualForm.permissions.includes(value)) {
                        setManualForm(prev => ({ 
                          ...prev, 
                          permissions: [...prev.permissions, value] 
                        }));
                      }
                    }}
                  >
                    <SelectTrigger>
                      <SelectValue placeholder="Select permissions to grant" />
                    </SelectTrigger>
                    <SelectContent>
                      {availablePermissions.map((perm) => (
                        <SelectItem key={perm} value={perm}>{perm}</SelectItem>
                      ))}
                    </SelectContent>
                  </Select>
                  <div className="flex flex-wrap gap-2 mt-2">
                    {manualForm.permissions.map((perm) => (
                      <Badge key={perm} variant="secondary">
                        {perm}
                        <button
                          onClick={() => setManualForm(prev => ({ 
                            ...prev, 
                            permissions: prev.permissions.filter(p => p !== perm) 
                          }))}
                          className="ml-2 text-red-500"
                        >
                          ×
                        </button>
                      </Badge>
                    ))}
                  </div>
                </div>

                <div>
                  <Label htmlFor="expires_in">Expires In (Days)</Label>
                  <Select 
                    value={manualForm.expires_in}
                    onValueChange={(value) => setManualForm(prev => ({ ...prev, expires_in: value }))}
                  >
                    <SelectTrigger>
                      <SelectValue />
                    </SelectTrigger>
                    <SelectContent>
                      <SelectItem value="7">7 Days</SelectItem>
                      <SelectItem value="30">30 Days</SelectItem>
                      <SelectItem value="90">90 Days</SelectItem>
                      <SelectItem value="365">1 Year</SelectItem>
                      <SelectItem value="">Never</SelectItem>
                    </SelectContent>
                  </Select>
                </div>

                <div>
                  <Label htmlFor="grant_reason">Grant Reason</Label>
                  <Textarea
                    id="grant_reason"
                    placeholder="Reason for granting these permissions..."
                    value={manualForm.grant_reason}
                    onChange={(e) => setManualForm(prev => ({ ...prev, grant_reason: e.target.value }))}
                  />
                </div>

                <Button 
                  onClick={() => grantManualPermission.mutate(manualForm)}
                  disabled={!manualForm.wallet_address || manualForm.permissions.length === 0 || grantManualPermission.isPending}
                  className="w-full"
                >
                  {grantManualPermission.isPending ? 'Granting...' : 'Grant Permissions'}
                </Button>
              </div>
            </CardContent>
          </Card>
        </TabsContent>

        {/* NFT Gates Tab */}
        <TabsContent value="nft" className="space-y-4">
          <Card>
            <CardHeader>
              <CardTitle className="flex items-center gap-2">
                🎨 NFT Permission Gates
              </CardTitle>
            </CardHeader>
            <CardContent>
              <div className="space-y-4">
                <div>
                  <Label htmlFor="nft_contract">NFT Contract Address</Label>
                  <Input
                    id="nft_contract"
                    placeholder="0x..."
                    value={nftForm.contract_address}
                    onChange={(e) => setNftForm(prev => ({ ...prev, contract_address: e.target.value }))}
                  />
                </div>

                <div>
                  <Label htmlFor="nft_name">Contract Name</Label>
                  <Input
                    id="nft_name"
                    placeholder="e.g., Bored Ape Yacht Club"
                    value={nftForm.contract_name}
                    onChange={(e) => setNftForm(prev => ({ ...prev, contract_name: e.target.value }))}
                  />
                </div>

                <div>
                  <Label htmlFor="min_tokens">Minimum Token Count</Label>
                  <Input
                    id="min_tokens"
                    type="number"
                    min="1"
                    value={nftForm.min_token_count}
                    onChange={(e) => setNftForm(prev => ({ ...prev, min_token_count: parseInt(e.target.value) }))}
                  />
                </div>

                <div>
                  <Label>Granted Permissions</Label>
                  <Select 
                    value=""
                    onValueChange={(value) => {
                      if (!nftForm.permissions.includes(value)) {
                        setNftForm(prev => ({ 
                          ...prev, 
                          permissions: [...prev.permissions, value] 
                        }));
                      }
                    }}
                  >
                    <SelectTrigger>
                      <SelectValue placeholder="Select permissions to grant" />
                    </SelectTrigger>
                    <SelectContent>
                      {availablePermissions.map((perm) => (
                        <SelectItem key={perm} value={perm}>{perm}</SelectItem>
                      ))}
                    </SelectContent>
                  </Select>
                  <div className="flex flex-wrap gap-2 mt-2">
                    {nftForm.permissions.map((perm) => (
                      <Badge key={perm} variant="secondary">
                        {perm}
                        <button
                          onClick={() => setNftForm(prev => ({ 
                            ...prev, 
                            permissions: prev.permissions.filter(p => p !== perm) 
                          }))}
                          className="ml-2 text-red-500"
                        >
                          ×
                        </button>
                      </Badge>
                    ))}
                  </div>
                </div>

                <Button 
                  onClick={() => createNFTGate.mutate(nftForm)}
                  disabled={!nftForm.contract_address || nftForm.permissions.length === 0 || createNFTGate.isPending}
                  className="w-full"
                >
                  {createNFTGate.isPending ? 'Creating...' : 'Create NFT Gate'}
                </Button>
              </div>

              {/* Existing NFT Gates */}
              <div className="mt-8 space-y-4">
                <h4 className="font-medium">Existing NFT Gates</h4>
                {nftGates.map((gate) => (
                  <div key={gate.id} className="border rounded-lg p-4">
                    <div className="flex items-start justify-between">
                      <div>
                        <div className="font-medium">{gate.contract_name}</div>
                        <div className="text-sm text-gray-600 dark:text-gray-400 font-mono">{gate.contract_address}</div>
                        <div className="text-sm">Min tokens: {gate.min_token_count}</div>
                        <div className="flex flex-wrap gap-1 mt-2">
                          {gate.permissions.map((perm) => (
                            <Badge key={perm} variant="outline" className="text-xs">{perm}</Badge>
                          ))}
                        </div>
                      </div>
                      <Badge variant={gate.is_active ? 'default' : 'secondary'}>
                        {gate.is_active ? 'Active' : 'Inactive'}
                      </Badge>
                    </div>
                  </div>
                ))}
              </div>
            </CardContent>
          </Card>
        </TabsContent>

        {/* Token Gates Tab */}
        <TabsContent value="token" className="space-y-4">
          <Card>
            <CardHeader>
              <CardTitle className="flex items-center gap-2">
                🪙 Token Permission Gates
              </CardTitle>
            </CardHeader>
            <CardContent>
              <div className="space-y-4">
                <div>
                  <Label htmlFor="token_contract">Token Contract Address</Label>
                  <Input
                    id="token_contract"
                    placeholder="0x..."
                    value={tokenForm.contract_address}
                    onChange={(e) => setTokenForm(prev => ({ ...prev, contract_address: e.target.value }))}
                  />
                </div>

                <div>
                  <Label htmlFor="token_symbol">Token Symbol</Label>
                  <Input
                    id="token_symbol"
                    placeholder="e.g., EPSX"
                    value={tokenForm.token_symbol}
                    onChange={(e) => setTokenForm(prev => ({ ...prev, token_symbol: e.target.value }))}
                  />
                </div>

                <div>
                  <Label htmlFor="min_amount">Minimum Amount (with decimals)</Label>
                  <Input
                    id="min_amount"
                    placeholder="e.g., 1000000000000000000 (1 token with 18 decimals)"
                    value={tokenForm.min_amount}
                    onChange={(e) => setTokenForm(prev => ({ ...prev, min_amount: e.target.value }))}
                  />
                </div>

                <div>
                  <Label>Granted Permissions</Label>
                  <Select 
                    value=""
                    onValueChange={(value) => {
                      if (!tokenForm.permissions.includes(value)) {
                        setTokenForm(prev => ({ 
                          ...prev, 
                          permissions: [...prev.permissions, value] 
                        }));
                      }
                    }}
                  >
                    <SelectTrigger>
                      <SelectValue placeholder="Select permissions to grant" />
                    </SelectTrigger>
                    <SelectContent>
                      {availablePermissions.map((perm) => (
                        <SelectItem key={perm} value={perm}>{perm}</SelectItem>
                      ))}
                    </SelectContent>
                  </Select>
                  <div className="flex flex-wrap gap-2 mt-2">
                    {tokenForm.permissions.map((perm) => (
                      <Badge key={perm} variant="secondary">
                        {perm}
                        <button
                          onClick={() => setTokenForm(prev => ({ 
                            ...prev, 
                            permissions: prev.permissions.filter(p => p !== perm) 
                          }))}
                          className="ml-2 text-red-500"
                        >
                          ×
                        </button>
                      </Badge>
                    ))}
                  </div>
                </div>

                <Button 
                  onClick={() => createTokenGate.mutate(tokenForm)}
                  disabled={!tokenForm.contract_address || !tokenForm.min_amount || tokenForm.permissions.length === 0 || createTokenGate.isPending}
                  className="w-full"
                >
                  {createTokenGate.isPending ? 'Creating...' : 'Create Token Gate'}
                </Button>
              </div>

              {/* Existing Token Gates */}
              <div className="mt-8 space-y-4">
                <h4 className="font-medium">Existing Token Gates</h4>
                {tokenGates.map((gate) => (
                  <div key={gate.id} className="border rounded-lg p-4">
                    <div className="flex items-start justify-between">
                      <div>
                        <div className="font-medium">{gate.token_symbol}</div>
                        <div className="text-sm text-gray-600 dark:text-gray-400 font-mono">{gate.contract_address}</div>
                        <div className="text-sm">Min amount: {gate.min_amount}</div>
                        <div className="flex flex-wrap gap-1 mt-2">
                          {gate.permissions.map((perm) => (
                            <Badge key={perm} variant="outline" className="text-xs">{perm}</Badge>
                          ))}
                        </div>
                      </div>
                      <Badge variant={gate.is_active ? 'default' : 'secondary'}>
                        {gate.is_active ? 'Active' : 'Inactive'}
                      </Badge>
                    </div>
                  </div>
                ))}
              </div>
            </CardContent>
          </Card>
        </TabsContent>

        {/* DAO Proposals Tab */}
        <TabsContent value="dao" className="space-y-4">
          <Card>
            <CardHeader>
              <CardTitle className="flex items-center gap-2">
                🗳️ DAO Governance Proposals
              </CardTitle>
            </CardHeader>
            <CardContent>
              <div className="space-y-4">
                <div>
                  <Label htmlFor="proposal_title">Proposal Title</Label>
                  <Input
                    id="proposal_title"
                    placeholder="Grant permissions to wallet..."
                    value={daoForm.title}
                    onChange={(e) => setDaoForm(prev => ({ ...prev, title: e.target.value }))}
                  />
                </div>

                <div>
                  <Label htmlFor="proposal_description">Description</Label>
                  <Textarea
                    id="proposal_description"
                    placeholder="Detailed description of the proposal..."
                    value={daoForm.description}
                    onChange={(e) => setDaoForm(prev => ({ ...prev, description: e.target.value }))}
                  />
                </div>

                <div>
                  <Label htmlFor="target_wallet">Target Wallet</Label>
                  <Input
                    id="target_wallet"
                    placeholder="0x..."
                    value={daoForm.target_wallet}
                    onChange={(e) => setDaoForm(prev => ({ ...prev, target_wallet: e.target.value }))}
                  />
                </div>

                <div>
                  <Label>Permissions to Grant</Label>
                  <Select 
                    value=""
                    onValueChange={(value) => {
                      if (!daoForm.permissions.includes(value)) {
                        setDaoForm(prev => ({ 
                          ...prev, 
                          permissions: [...prev.permissions, value] 
                        }));
                      }
                    }}
                  >
                    <SelectTrigger>
                      <SelectValue placeholder="Select permissions to grant" />
                    </SelectTrigger>
                    <SelectContent>
                      {availablePermissions.map((perm) => (
                        <SelectItem key={perm} value={perm}>{perm}</SelectItem>
                      ))}
                    </SelectContent>
                  </Select>
                  <div className="flex flex-wrap gap-2 mt-2">
                    {daoForm.permissions.map((perm) => (
                      <Badge key={perm} variant="secondary">
                        {perm}
                        <button
                          onClick={() => setDaoForm(prev => ({ 
                            ...prev, 
                            permissions: prev.permissions.filter(p => p !== perm) 
                          }))}
                          className="ml-2 text-red-500"
                        >
                          ×
                        </button>
                      </Badge>
                    ))}
                  </div>
                </div>

                <div>
                  <Label htmlFor="votes_required">Votes Required</Label>
                  <Input
                    id="votes_required"
                    type="number"
                    min="1"
                    value={daoForm.votes_required}
                    onChange={(e) => setDaoForm(prev => ({ ...prev, votes_required: parseInt(e.target.value) }))}
                  />
                </div>

                <div>
                  <Label htmlFor="expires_in_dao">Voting Period (Days)</Label>
                  <Select 
                    value={daoForm.expires_in}
                    onValueChange={(value) => setDaoForm(prev => ({ ...prev, expires_in: value }))}
                  >
                    <SelectTrigger>
                      <SelectValue />
                    </SelectTrigger>
                    <SelectContent>
                      <SelectItem value="3">3 Days</SelectItem>
                      <SelectItem value="7">7 Days</SelectItem>
                      <SelectItem value="14">14 Days</SelectItem>
                      <SelectItem value="30">30 Days</SelectItem>
                    </SelectContent>
                  </Select>
                </div>

                <Button 
                  onClick={() => createDAOProposal.mutate(daoForm)}
                  disabled={!daoForm.title || !daoForm.target_wallet || daoForm.permissions.length === 0 || createDAOProposal.isPending}
                  className="w-full"
                >
                  {createDAOProposal.isPending ? 'Creating...' : 'Create DAO Proposal'}
                </Button>
              </div>

              {/* Existing DAO Proposals */}
              <div className="mt-8 space-y-4">
                <h4 className="font-medium">Active Proposals</h4>
                {daoProposals.map((proposal) => (
                  <div key={proposal.id} className="border rounded-lg p-4">
                    <div className="flex items-start justify-between">
                      <div className="flex-1">
                        <div className="font-medium">{proposal.title}</div>
                        <div className="text-sm text-gray-600 dark:text-gray-400 mt-1">{proposal.description}</div>
                        <div className="text-sm font-mono mt-2">Target: {proposal.target_wallet}</div>
                        <div className="flex items-center gap-4 mt-2 text-sm">
                          <span>For: {proposal.votes_for}</span>
                          <span>Against: {proposal.votes_against}</span>
                          <span>Required: {proposal.votes_required}</span>
                        </div>
                        <div className="flex flex-wrap gap-1 mt-2">
                          {proposal.permissions.map((perm) => (
                            <Badge key={perm} variant="outline" className="text-xs">{perm}</Badge>
                          ))}
                        </div>
                      </div>
                      <div className="text-right">
                        <Badge 
                          variant={
                            proposal.status === 'approved' ? 'default' :
                            proposal.status === 'rejected' ? 'destructive' :
                            proposal.status === 'executed' ? 'default' : 'secondary'
                          }
                        >
                          {proposal.status}
                        </Badge>
                        <div className="text-xs text-gray-500 mt-1">
                          Expires: {format(parseISO(proposal.expires_at), 'MMM dd')}
                        </div>
                      </div>
                    </div>
                  </div>
                ))}
              </div>
            </CardContent>
          </Card>
        </TabsContent>
      </Tabs>
    </div>
  );
}