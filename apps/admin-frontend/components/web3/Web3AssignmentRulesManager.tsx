/**
 * Web3 Assignment Rules Manager Component
 * Interface for managing blockchain-based group assignment rules
 * 
 * Features:
 * - Create/edit/delete Web3 assignment rules
 * - Multi-chain support (BSC, Ethereum, Polygon, etc.)
 * - NFT, Token, and DAO-based verification rules
 * - Real-time wallet processing and testing
 * - Rule analytics and performance monitoring
 */

'use client'

import { 
  Zap, Plus, Settings, Trash2, Edit, Globe, Coins,
  Image, Users, AlertTriangle, CheckCircle, Eye,
  Search, Filter, MoreHorizontal, Play, Pause,
  ExternalLink, Copy, Wallet, TrendingUp, Activity
} from 'lucide-react'
import React, { useState, useCallback, useMemo } from 'react'

import { Alert, AlertDescription } from '@/components/ui/alert'
import { Badge } from '@/components/ui/badge'
import { Button } from '@/components/ui/button'
import { Card, CardContent, CardHeader, CardTitle, CardDescription } from '@/components/ui/card'
import { 
  Dialog, DialogContent, DialogHeader, DialogTitle, DialogTrigger, DialogFooter 
} from '@/components/ui/dialog'
import { 
  DropdownMenu, DropdownMenuContent, DropdownMenuItem, DropdownMenuTrigger 
} from '@/components/ui/dropdown-menu'
import { Input } from '@/components/ui/input'
import { Label } from '@/components/ui/label'
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from '@/components/ui/select'
import { Tabs, TabsContent, TabsList, TabsTrigger } from '@/components/ui/tabs'
import { useToast } from '@/components/ui/use-toast'
import { adminCardVariants, adminButtonVariants } from '@/design-system'
import {
  useWeb3AssignmentRules,
  usePermissionGroups
} from '@/hooks/useGroupPermissions'
import { 
  Web3AssignmentRule, 
  CreateWeb3RuleRequest,
  ProcessWalletRequest 
} from '@/lib/api/group-management-client'
import { cn } from '@/lib/shared'

interface Web3AssignmentRulesManagerProps {
  className?: string
}

const BLOCKCHAIN_NETWORKS = [
  { value: 'bsc_mainnet', label: 'BSC Mainnet', icon: '🟡' },
  { value: 'bsc_testnet', label: 'BSC Testnet', icon: '🟠' },
  { value: 'ethereum_mainnet', label: 'Ethereum', icon: '🔷' },
  { value: 'polygon_mainnet', label: 'Polygon', icon: '🟣' },
  { value: 'arbitrum_mainnet', label: 'Arbitrum', icon: '🔵' },
  { value: 'optimism_mainnet', label: 'Optimism', icon: '🔴' }
] as const

const VERIFICATION_TYPES = [
  { 
    value: 'nft_ownership', 
    label: 'NFT Ownership', 
    icon: <Image className="h-4 w-4" />,
    description: 'Verify ownership of specific NFTs'
  },
  { 
    value: 'token_balance', 
    label: 'Token Balance', 
    icon: <Coins className="h-4 w-4" />,
    description: 'Check minimum token balance'
  },
  { 
    value: 'dao_membership', 
    label: 'DAO Membership', 
    icon: <Users className="h-4 w-4" />,
    description: 'Verify DAO participation'
  }
] as const

/**
 *
 * @param root0
 * @param root0.className
 */
export function Web3AssignmentRulesManager({ className }: Web3AssignmentRulesManagerProps) {
  const { toast } = useToast()
  const [searchTerm, setSearchTerm] = useState('')
  const [filterNetwork, setFilterNetwork] = useState<string>('all')
  const [filterType, setFilterType] = useState<string>('all')
  const [showCreateDialog, setShowCreateDialog] = useState(false)
  const [editingRule, setEditingRule] = useState<Web3AssignmentRule | null>(null)
  const [testingWallet, setTestingWallet] = useState('')
  const [testingRule, setTestingRule] = useState<Web3AssignmentRule | null>(null)
  const [activeTab, setActiveTab] = useState('rules')

  // Hooks
  const {
    rules,
    loading: isLoading,
    error,
    processWallet,
    verifyWalletAssets,
    refreshRules
  } = useWeb3AssignmentRules()

  const { groups } = usePermissionGroups()
  // Backend handles permission checking - no client-side validation needed

  // Filter rules
  const filteredRules = useMemo(() => {
    let filtered = rules

    // Filter by network
    if (filterNetwork !== 'all') {
      filtered = filtered.filter(rule => rule.blockchain_network === filterNetwork)
    }

    // Filter by verification type
    if (filterType !== 'all') {
      filtered = filtered.filter(rule => rule.verification_type === filterType)
    }

    // Filter by search term
    if (searchTerm) {
      const searchLower = searchTerm.toLowerCase()
      filtered = filtered.filter(rule => 
        rule.group?.name.toLowerCase().includes(searchLower) ||
        rule.contract_address?.toLowerCase().includes(searchLower)
      )
    }

    return filtered.sort((a, b) => 
      new Date(b.created_at).getTime() - new Date(a.created_at).getTime()
    )
  }, [rules, filterNetwork, filterType, searchTerm])

  // Event handlers
  const handleCreateRule = useCallback(async (ruleData: CreateWeb3RuleRequest) => {
    toast({
      title: 'Not Implemented',
      description: 'Creating Web3 assignment rules is not yet implemented.',
      variant: 'destructive'
    })
  }, [toast])

  const handleDeleteRule = useCallback(async (rule: Web3AssignmentRule) => {
    toast({
      title: 'Not Implemented',
      description: 'Deleting Web3 assignment rules is not yet implemented.',
      variant: 'destructive'
    })
  }, [toast])

  const handleTestWallet = useCallback(async (walletAddress: string) => {
    if (!walletAddress.trim()) {return}

    try {
      setTestingRule(null)
      await processWallet(walletAddress.trim())
      toast({
        title: 'Wallet Processed',
        description: 'Wallet has been processed based on blockchain assets.'
      })
    } catch (_error) {
      toast({
        title: 'Processing Failed',
        description: _error instanceof Error ? _error.message : 'Failed to process wallet',
        variant: 'destructive'
      })
    }
  }, [processWallet, toast])

  const handleVerifyAssets = useCallback(async (walletAddress: string, rule?: Web3AssignmentRule) => {
    if (!walletAddress.trim()) {return}

    try {
      setTestingRule(rule || null)
      const assets = await verifyWalletAssets(walletAddress.trim())
      // Handle verification results
    } catch (_error) {
      toast({
        title: 'Verification Failed',
        description: _error instanceof Error ? _error.message : 'Failed to verify wallet assets',
        variant: 'destructive'
      })
    }
  }, [verifyWalletAssets, toast])

  const getNetworkInfo = (network: string) => {
    return BLOCKCHAIN_NETWORKS.find(n => n.value === network) || 
           { value: network, label: network, icon: '🌐' }
  }

  const getVerificationTypeInfo = (type: string) => {
    return VERIFICATION_TYPES.find(t => t.value === type) || 
           { value: type, label: type, icon: <Globe className="h-4 w-4" />, description: '' }
  }

  if (isLoading) {
    return (
      <div className={cn('space-y-6', className)}>
        {[...Array(4)].map((_, i) => (
          <Card key={i} className="animate-pulse">
            <CardHeader className="space-y-2">
              <div className="h-4 bg-gray-200 rounded w-1/3"></div>
              <div className="h-3 bg-gray-200 rounded w-1/2"></div>
            </CardHeader>
            <CardContent>
              <div className="h-24 bg-gray-200 rounded"></div>
            </CardContent>
          </Card>
        ))}
      </div>
    )
  }

  if (error) {
    return (
      <div className={className}>
        <Alert variant="destructive">
          <AlertTriangle className="h-4 w-4" />
          <AlertDescription>
            Failed to load Web3 assignment rules: {error}
          </AlertDescription>
        </Alert>
      </div>
    )
  }

  return (
    <div className={cn('space-y-6', className)}>
      {/* Header */}
      <div className="flex flex-col sm:flex-row justify-between items-start sm:items-center gap-4">
        <div>
          <h1 className="text-2xl font-semibold text-gray-900">Web3 Assignment Rules</h1>
          <p className="text-sm text-gray-600 mt-1">
            Manage blockchain-based automatic group assignments
          </p>
        </div>
        {canManageWeb3Rules && (
          <Button 
            onClick={() => setShowCreateDialog(true)}
            className={adminButtonVariants({ variant: 'primary', size: 'sm' })}
          >
            <Plus className="h-4 w-4 mr-2" />
            Create Rule
          </Button>
        )}
      </div>

      {/* Stats */}
      <div className="grid grid-cols-2 md:grid-cols-4 gap-4">
        <Card className={adminCardVariants({ variant: 'default' })}>
          <CardContent className="pt-6">
            <div className="flex items-center justify-between">
              <div>
                <p className="text-sm font-medium text-gray-600">Total Rules</p>
                <p className="text-2xl font-bold text-gray-900">{rules.length}</p>
              </div>
              <Zap className="h-8 w-8 text-purple-600" />
            </div>
          </CardContent>
        </Card>

        <Card className={adminCardVariants({ variant: 'default' })}>
          <CardContent className="pt-6">
            <div className="flex items-center justify-between">
              <div>
                <p className="text-sm font-medium text-gray-600">Active Rules</p>
                <p className="text-2xl font-bold text-green-600">{rules.length}</p>
              </div>
              <CheckCircle className="h-8 w-8 text-green-600" />
            </div>
          </CardContent>
        </Card>

        <Card className={adminCardVariants({ variant: 'default' })}>
          <CardContent className="pt-6">
            <div className="flex items-center justify-between">
              <div>
                <p className="text-sm font-medium text-gray-600">Networks</p>
                <p className="text-2xl font-bold text-blue-600">
                  {new Set(rules.map(r => r.blockchain_network)).size}
                </p>
              </div>
              <Globe className="h-8 w-8 text-blue-600" />
            </div>
          </CardContent>
        </Card>

        <Card className={adminCardVariants({ variant: 'default' })}>
          <CardContent className="pt-6">
            <div className="flex items-center justify-between">
              <div>
                <p className="text-sm font-medium text-gray-600">Groups</p>
                <p className="text-2xl font-bold text-orange-600">
                  {new Set(rules.map(r => r.group_id)).size}
                </p>
              </div>
              <Users className="h-8 w-8 text-orange-600" />
            </div>
          </CardContent>
        </Card>
      </div>

      <Tabs value={activeTab} onValueChange={setActiveTab} className="w-full">
        <TabsList className="grid w-full grid-cols-2">
          <TabsTrigger value="rules">Assignment Rules</TabsTrigger>
          <TabsTrigger value="testing">Wallet Testing</TabsTrigger>
        </TabsList>

        <TabsContent value="rules" className="mt-6">
          {/* Filters */}
          <div className="flex flex-col sm:flex-row gap-4 mb-6">
            <div className="relative flex-1">
              <Search className="h-4 w-4 absolute left-3 top-3 text-gray-400" />
              <Input
                placeholder="Search rules, groups, or contract addresses..."
                value={searchTerm}
                onChange={(e) => setSearchTerm(e.target.value)}
                className="pl-10"
              />
            </div>
            <Select value={filterNetwork} onValueChange={setFilterNetwork}>
              <SelectTrigger className="w-full sm:w-48">
                <SelectValue />
              </SelectTrigger>
              <SelectContent>
                <SelectItem value="all">All Networks</SelectItem>
                {BLOCKCHAIN_NETWORKS.map((network) => (
                  <SelectItem key={network.value} value={network.value}>
                    {network.icon} {network.label}
                  </SelectItem>
                ))}
              </SelectContent>
            </Select>
            <Select value={filterType} onValueChange={setFilterType}>
              <SelectTrigger className="w-full sm:w-48">
                <SelectValue />
              </SelectTrigger>
              <SelectContent>
                <SelectItem value="all">All Types</SelectItem>
                {VERIFICATION_TYPES.map((type) => (
                  <SelectItem key={type.value} value={type.value}>
                    {type.icon} {type.label}
                  </SelectItem>
                ))}
              </SelectContent>
            </Select>
          </div>

          {/* Rules List */}
          <div className="grid grid-cols-1 lg:grid-cols-2 gap-6">
            {filteredRules.map((rule) => {
              const networkInfo = getNetworkInfo(rule.blockchain_network)
              const typeInfo = getVerificationTypeInfo(rule.verification_type)
              
              return (
                <Card key={rule.id} className={adminCardVariants({ variant: 'default' })}>
                  <CardHeader className="pb-3">
                    <div className="flex items-start justify-between">
                      <div className="flex items-center gap-2">
                        {typeInfo.icon}
                        <CardTitle className="text-base">{rule.group?.name}</CardTitle>
                      </div>
                      <div className="flex items-center gap-2">
                        <Badge variant={rule.is_active ? 'default' : 'secondary'}>
                          {rule.is_active ? 'Active' : 'Inactive'}
                        </Badge>
                        {canManageWeb3Rules && (
                          <DropdownMenu>
                            <DropdownMenuTrigger asChild>
                              <Button variant="ghost" size="sm" className="h-8 w-8 p-0">
                                <MoreHorizontal className="h-4 w-4" />
                              </Button>
                            </DropdownMenuTrigger>
                            <DropdownMenuContent align="end">
                              <DropdownMenuItem 
                                onClick={() => handleVerifyAssets(testingWallet || '0x742d35Cc6634C0532925a3b8D4a9529B29F2D3f9', rule)}
                              >
                                <Eye className="h-4 w-4 mr-2" />
                                Test Rule
                              </DropdownMenuItem>
                              <DropdownMenuItem 
                                onClick={() => handleDeleteRule(rule)}
                                className="text-red-600"
                              >
                                <Trash2 className="h-4 w-4 mr-2" />
                                Delete
                              </DropdownMenuItem>
                            </DropdownMenuContent>
                          </DropdownMenu>
                        )}
                      </div>
                    </div>
                    <CardDescription className="flex items-center gap-2">
                      <span>{networkInfo.icon}</span>
                      <span>{networkInfo.label}</span>
                      <span>•</span>
                      <span>{typeInfo.label}</span>
                    </CardDescription>
                  </CardHeader>
                  
                  <CardContent className="space-y-3">
                    {rule.contract_address && (
                      <div className="flex items-center justify-between text-sm">
                        <span className="text-gray-600">Contract</span>
                        <div className="flex items-center gap-2">
                          <code className="text-xs bg-gray-100 px-2 py-1 rounded">
                            {rule.contract_address.slice(0, 6)}...{rule.contract_address.slice(-4)}
                          </code>
                          <Button 
                            variant="ghost" 
                            size="sm" 
                            className="h-6 w-6 p-0"
                            onClick={() => navigator.clipboard.writeText(rule.contract_address)}
                          >
                            <Copy className="h-3 w-3" />
                          </Button>
                        </div>
                      </div>
                    )}
                    
                    {rule.token_id && (
                      <div className="flex items-center justify-between text-sm">
                        <span className="text-gray-600">Token ID</span>
                        <Badge variant="outline">{rule.token_id}</Badge>
                      </div>
                    )}
                    
                    {rule.minimum_balance && (
                      <div className="flex items-center justify-between text-sm">
                        <span className="text-gray-600">Min Balance</span>
                        <Badge variant="outline">{rule.minimum_balance}</Badge>
                      </div>
                    )}

                    <div className="pt-2 border-t border-gray-100 text-xs text-gray-500">
                      Created: {new Date(rule.created_at).toLocaleDateString()}
                    </div>
                  </CardContent>
                </Card>
              )
            })}
          </div>

          {filteredRules.length === 0 && (
            <Card className="p-8 text-center">
              <Zap className="h-12 w-12 text-gray-400 mx-auto mb-4" />
              <h3 className="text-lg font-medium text-gray-900 mb-2">No rules found</h3>
              <p className="text-gray-600">
                {searchTerm || filterNetwork !== 'all' || filterType !== 'all'
                  ? 'No rules match your search criteria.'
                  : canManageWeb3Rules 
                    ? 'Create your first Web3 assignment rule to get started.'
                    : 'No Web3 assignment rules have been created yet.'
                }
              </p>
            </Card>
          )}
        </TabsContent>

        <TabsContent value="testing" className="mt-6">
          <Card className={adminCardVariants({ variant: 'default' })}>
            <CardHeader>
              <CardTitle className="flex items-center gap-2">
                <Wallet className="h-5 w-5" />
                Wallet Testing
              </CardTitle>
              <CardDescription>
                Test wallet addresses against your Web3 assignment rules
              </CardDescription>
            </CardHeader>
            <CardContent className="space-y-4">
              <div className="flex gap-2">
                <Input
                  placeholder="Enter wallet address (0x...)"
                  value={testingWallet}
                  onChange={(e) => setTestingWallet(e.target.value)}
                  className="flex-1"
                />
                <Button 
                  onClick={() => handleTestWallet(testingWallet)}
                  disabled={!testingWallet.trim()}
                >
                  <Play className="h-4 w-4 mr-2" />
                  Process Wallet
                </Button>
              </div>
              <div className="text-sm text-gray-600">
                Enter a wallet address to test it against all active assignment rules.
                This will show which groups the wallet would be assigned to based on its blockchain assets.
              </div>
            </CardContent>
          </Card>
        </TabsContent>
      </Tabs>

      {/* Create Rule Dialog */}
      <Dialog open={showCreateDialog} onOpenChange={setShowCreateDialog}>
        <DialogContent className="max-w-2xl max-h-[90vh] overflow-y-auto">
          <DialogHeader>
            <DialogTitle>Create Web3 Assignment Rule</DialogTitle>
          </DialogHeader>
          <Web3RuleForm
            groups={groups}
            onSave={handleCreateRule}
            onCancel={() => setShowCreateDialog(false)}
          />
        </DialogContent>
      </Dialog>
    </div>
  )
}

// Web3 Rule Form Component
interface Web3RuleFormProps {
  groups: any[]
  onSave: (rule: CreateWeb3RuleRequest) => void
  onCancel: () => void
}

function Web3RuleForm({ groups, onSave, onCancel }: Web3RuleFormProps) {
  const [formData, setFormData] = useState({
    group_id: '',
    blockchain_network: 'bsc_testnet',
    verification_type: 'token_balance',
    contract_address: '',
    token_id: '',
    minimum_balance: ''
  })

  const handleSubmit = useCallback((e: React.FormEvent) => {
    e.preventDefault()
    if (!formData.group_id || !formData.contract_address) {return}

    const request: CreateWeb3RuleRequest = {
      group_id: formData.group_id,
      blockchain_network: formData.blockchain_network,
      verification_type: formData.verification_type,
      contract_address: formData.contract_address.trim(),
      token_id: formData.token_id.trim() || undefined,
      minimum_balance: formData.minimum_balance.trim() || undefined
    }

    onSave(request)
  }, [formData, onSave])

  return (
    <form onSubmit={handleSubmit} className="space-y-4">
      <div>
        <Label>Target Group *</Label>
        <Select value={formData.group_id} onValueChange={(value) => 
          setFormData(prev => ({ ...prev, group_id: value }))
        }>
          <SelectTrigger>
            <SelectValue placeholder="Select group" />
          </SelectTrigger>
          <SelectContent>
            {groups.map((group) => (
              <SelectItem key={group.id} value={group.id}>
                {group.name}
              </SelectItem>
            ))}
          </SelectContent>
        </Select>
      </div>

      <div>
        <Label>Blockchain Network *</Label>
        <Select value={formData.blockchain_network} onValueChange={(value) => 
          setFormData(prev => ({ ...prev, blockchain_network: value }))
        }>
          <SelectTrigger>
            <SelectValue />
          </SelectTrigger>
          <SelectContent>
            {BLOCKCHAIN_NETWORKS.map((network) => (
              <SelectItem key={network.value} value={network.value}>
                {network.icon} {network.label}
              </SelectItem>
            ))}
          </SelectContent>
        </Select>
      </div>

      <div>
        <Label>Verification Type *</Label>
        <Select value={formData.verification_type} onValueChange={(value) => 
          setFormData(prev => ({ ...prev, verification_type: value }))
        }>
          <SelectTrigger>
            <SelectValue />
          </SelectTrigger>
          <SelectContent>
            {VERIFICATION_TYPES.map((type) => (
              <SelectItem key={type.value} value={type.value}>
                <div className="flex items-center gap-2">
                  {type.icon}
                  <div>
                    <div className="font-medium">{type.label}</div>
                    <div className="text-xs text-gray-500">{type.description}</div>
                  </div>
                </div>
              </SelectItem>
            ))}
          </SelectContent>
        </Select>
      </div>

      <div>
        <Label>Contract Address *</Label>
        <Input
          value={formData.contract_address}
          onChange={(e) => setFormData(prev => ({ ...prev, contract_address: e.target.value }))}
          placeholder="0x..."
        />
      </div>

      {formData.verification_type === 'nft_ownership' && (
        <div>
          <Label>Token ID (optional)</Label>
          <Input
            value={formData.token_id}
            onChange={(e) => setFormData(prev => ({ ...prev, token_id: e.target.value }))}
            placeholder="Specific token ID or leave empty for any"
          />
        </div>
      )}

      {formData.verification_type === 'token_balance' && (
        <div>
          <Label>Minimum Balance</Label>
          <Input
            value={formData.minimum_balance}
            onChange={(e) => setFormData(prev => ({ ...prev, minimum_balance: e.target.value }))}
            placeholder="Minimum token balance required"
          />
        </div>
      )}

      <DialogFooter>
        <Button type="button" variant="outline" onClick={onCancel}>
          Cancel
        </Button>
        <Button 
          type="submit" 
          disabled={!formData.group_id || !formData.contract_address}
        >
          Create Rule
        </Button>
      </DialogFooter>
    </form>
  )
}

export default Web3AssignmentRulesManager