/**
 * Group-Based Bulk User Operations Component
 * Updated bulk operations interface for the new group-based permission system
 * 
 * Features:
 * - Bulk assign/remove users to/from permission groups
 * - Bulk Web3 wallet processing
 * - Bulk status updates with group considerations
 * - Progress tracking and error reporting
 * - Audit trail integration
 */

'use client';

import { useState, useCallback, useMemo } from 'react';
import { 
  Users, Upload, Download, Zap, CheckCircle, XCircle,
  AlertTriangle, Clock, Globe, Shield, Activity,
  Play, Pause, MoreHorizontal, FileText, BarChart3
} from 'lucide-react';

import { Button } from '@/components/ui/button';
import { Card, CardContent, CardHeader, CardTitle, CardDescription } from '@/components/ui/card';
import { Badge } from '@/components/ui/badge';
import { Input } from '@/components/ui/input';
import { Label } from '@/components/ui/label';
import { Textarea } from '@/components/ui/textarea';
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from '@/components/ui/select';
import { Checkbox } from '@/components/ui/checkbox';
import { Progress } from '@/components/ui/progress';
import { Alert, AlertDescription } from '@/components/ui/alert';
import { Tabs, TabsContent, TabsList, TabsTrigger } from '@/components/ui/tabs';
import { 
  Dialog, DialogContent, DialogHeader, DialogTitle, DialogTrigger, DialogFooter 
} from '@/components/ui/dialog';
import { useToast } from '@/components/ui/use-toast';

import { 
  usePermissionGroups,
  useWeb3AssignmentRules,
  useAdminGroupPermissions
} from '@/hooks/useGroupPermissions';
import { groupManagementClient } from '@/lib/api/group-management-client';
import type { User } from '@/types/core';
import { adminCardVariants, adminButtonVariants } from '@/design-system';
import { cn } from '@/lib/shared';

interface GroupBasedBulkOperationsProps {
  users?: User[];
  selectedUserIds?: string[];
  onOperationComplete?: (result: BulkOperationResult) => void;
  className?: string;
}

interface BulkOperationResult {
  operation: string;
  total: number;
  successful: number;
  failed: number;
  errors: string[];
  details: any[];
}

interface OperationProgress {
  current: number;
  total: number;
  status: 'running' | 'completed' | 'failed' | 'paused';
  errors: string[];
  results: any[];
}

type BulkOperationType = 
  | 'assign_group'
  | 'remove_group'
  | 'process_web3_wallets'
  | 'export_data'
  | 'update_status'
  | 'cleanup_expired';

const BULK_OPERATIONS = [
  {
    type: 'assign_group' as BulkOperationType,
    label: 'Assign to Group',
    icon: <Users className="h-4 w-4" />,
    description: 'Add selected users to a permission group',
    requiresGroup: true
  },
  {
    type: 'remove_group' as BulkOperationType,
    label: 'Remove from Group',
    icon: <XCircle className="h-4 w-4" />,
    description: 'Remove selected users from a permission group',
    requiresGroup: true
  },
  {
    type: 'process_web3_wallets' as BulkOperationType,
    label: 'Process Web3 Wallets',
    icon: <Zap className="h-4 w-4" />,
    description: 'Process blockchain assets for automatic group assignment',
    requiresGroup: false
  },
  {
    type: 'export_data' as BulkOperationType,
    label: 'Export User Data',
    icon: <Download className="h-4 w-4" />,
    description: 'Export user and group data to CSV/JSON',
    requiresGroup: false
  },
  {
    type: 'cleanup_expired' as BulkOperationType,
    label: 'Cleanup Expired Memberships',
    icon: <Clock className="h-4 w-4" />,
    description: 'Remove expired group memberships for selected users',
    requiresGroup: false
  }
];

export function GroupBasedBulkOperations({
  users = [],
  selectedUserIds = [],
  onOperationComplete,
  className
}: GroupBasedBulkOperationsProps) {
  const { toast } = useToast();
  
  // State
  const [selectedOperation, setSelectedOperation] = useState<BulkOperationType | ''>('');
  const [selectedGroupId, setSelectedGroupId] = useState<string>('');
  const [reason, setReason] = useState<string>('');
  const [expiryDays, setExpiryDays] = useState<string>('');
  const [progress, setProgress] = useState<OperationProgress | null>(null);
  const [showConfirmDialog, setShowConfirmDialog] = useState(false);

  // Hooks
  const { groups, systemGroups } = usePermissionGroups();
  const { processWallet, bulkProcessWallets } = useWeb3AssignmentRules();
  const { canManageUsers, canManageGroups } = useAdminGroupPermissions();

  // Get selected users data
  const selectedUsers = useMemo(() => {
    return users.filter(user => selectedUserIds.includes(user.id));
  }, [users, selectedUserIds]);

  const selectedOperationData = useMemo(() => {
    return BULK_OPERATIONS.find(op => op.type === selectedOperation);
  }, [selectedOperation]);

  // Validation
  const isValidOperation = useMemo(() => {
    if (!selectedOperation || selectedUserIds.length === 0) return false;
    if (selectedOperationData?.requiresGroup && !selectedGroupId) return false;
    return true;
  }, [selectedOperation, selectedOperationData, selectedGroupId, selectedUserIds.length]);

  // Operation handlers
  const handleAssignGroup = useCallback(async (userIds: string[], groupId: string): Promise<BulkOperationResult> => {
    const results = [];
    const errors = [];
    let successful = 0;

    const expiresAt = expiryDays 
      ? new Date(Date.now() + parseInt(expiryDays) * 24 * 60 * 60 * 1000).toISOString()
      : null;

    for (const userId of userIds) {
      try {
        await groupManagementClient.assignUserToGroup({
          user_id: userId,
          group_id: groupId,
          expires_at: expiresAt,
          reason: reason || 'Bulk assignment operation'
        });
        successful++;
        results.push({ userId, status: 'success' });
        
        // Update progress
        setProgress(prev => prev ? {
          ...prev,
          current: prev.current + 1,
          results: [...prev.results, { userId, status: 'success' }]
        } : null);
      } catch (error) {
        const errorMsg = error instanceof Error ? error.message : 'Unknown error';
        errors.push(`User ${userId}: ${errorMsg}`);
        results.push({ userId, status: 'error', error: errorMsg });
        
        setProgress(prev => prev ? {
          ...prev,
          current: prev.current + 1,
          errors: [...prev.errors, errorMsg],
          results: [...prev.results, { userId, status: 'error' }]
        } : null);
      }
    }

    return {
      operation: 'assign_group',
      total: userIds.length,
      successful,
      failed: userIds.length - successful,
      errors,
      details: results
    };
  }, [reason, expiryDays]);

  const handleRemoveGroup = useCallback(async (userIds: string[], groupId: string): Promise<BulkOperationResult> => {
    const results = [];
    const errors = [];
    let successful = 0;

    for (const userId of userIds) {
      try {
        await groupManagementClient.removeUserFromGroup(userId, groupId);
        successful++;
        results.push({ userId, status: 'success' });
        
        setProgress(prev => prev ? {
          ...prev,
          current: prev.current + 1,
          results: [...prev.results, { userId, status: 'success' }]
        } : null);
      } catch (error) {
        const errorMsg = error instanceof Error ? error.message : 'Unknown error';
        errors.push(`User ${userId}: ${errorMsg}`);
        results.push({ userId, status: 'error', error: errorMsg });
        
        setProgress(prev => prev ? {
          ...prev,
          current: prev.current + 1,
          errors: [...prev.errors, errorMsg],
          results: [...prev.results, { userId, status: 'error' }]
        } : null);
      }
    }

    return {
      operation: 'remove_group',
      total: userIds.length,
      successful,
      failed: userIds.length - successful,
      errors,
      details: results
    };
  }, []);

  const handleProcessWeb3Wallets = useCallback(async (userIds: string[]): Promise<BulkOperationResult> => {
    const usersWithWallets = selectedUsers.filter(user => user.wallet_address);
    const walletAddresses = usersWithWallets.map(user => user.wallet_address!);
    
    if (walletAddresses.length === 0) {
      return {
        operation: 'process_web3_wallets',
        total: userIds.length,
        successful: 0,
        failed: userIds.length,
        errors: ['No wallet addresses found for selected users'],
        details: []
      };
    }

    const results = [];
    const errors = [];
    let successful = 0;

    try {
      const bulkResult = await bulkProcessWallets(walletAddresses);
      successful = walletAddresses.length;
      results.push({ wallets: walletAddresses, result: bulkResult });
      
      setProgress(prev => prev ? {
        ...prev,
        current: walletAddresses.length,
        status: 'completed',
        results: [{ status: 'success', details: bulkResult }]
      } : null);
    } catch (error) {
      const errorMsg = error instanceof Error ? error.message : 'Bulk processing failed';
      errors.push(errorMsg);
      results.push({ status: 'error', error: errorMsg });
      
      setProgress(prev => prev ? {
        ...prev,
        current: walletAddresses.length,
        status: 'failed',
        errors: [errorMsg]
      } : null);
    }

    return {
      operation: 'process_web3_wallets',
      total: userIds.length,
      successful,
      failed: userIds.length - successful,
      errors,
      details: results
    };
  }, [selectedUsers, bulkProcessWallets]);

  const handleExportData = useCallback(async (userIds: string[]): Promise<BulkOperationResult> => {
    try {
      // Collect user data with group information
      const exportData = await Promise.all(
        selectedUsers.map(async (user) => {
          const userGroups = await groupManagementClient.getUserGroups(user.id);
          const userPermissions = await groupManagementClient.getUserPermissions(user.id);
          
          return {
            ...user,
            groups: userGroups.map(membership => ({
              group_name: membership.group?.name,
              is_active: membership.is_active,
              expires_at: membership.expires_at,
              granted_at: membership.granted_at
            })),
            effective_permissions: userPermissions
          };
        })
      );

      // Create and download CSV
      const csv = convertToCSV(exportData);
      downloadCSV(csv, 'bulk-user-export.csv');

      setProgress(prev => prev ? {
        ...prev,
        current: userIds.length,
        status: 'completed',
        results: [{ status: 'success', exported: exportData.length }]
      } : null);

      return {
        operation: 'export_data',
        total: userIds.length,
        successful: exportData.length,
        failed: 0,
        errors: [],
        details: [{ exported: exportData.length }]
      };
    } catch (error) {
      const errorMsg = error instanceof Error ? error.message : 'Export failed';
      return {
        operation: 'export_data',
        total: userIds.length,
        successful: 0,
        failed: userIds.length,
        errors: [errorMsg],
        details: []
      };
    }
  }, [selectedUsers]);

  // Main operation executor
  const executeOperation = useCallback(async () => {
    if (!isValidOperation) return;

    setProgress({
      current: 0,
      total: selectedUserIds.length,
      status: 'running',
      errors: [],
      results: []
    });

    let result: BulkOperationResult;

    try {
      switch (selectedOperation) {
        case 'assign_group':
          result = await handleAssignGroup(selectedUserIds, selectedGroupId);
          break;
        case 'remove_group':
          result = await handleRemoveGroup(selectedUserIds, selectedGroupId);
          break;
        case 'process_web3_wallets':
          result = await handleProcessWeb3Wallets(selectedUserIds);
          break;
        case 'export_data':
          result = await handleExportData(selectedUserIds);
          break;
        default:
          throw new Error('Unsupported operation');
      }

      setProgress(prev => prev ? { ...prev, status: 'completed' } : null);
      
      toast({
        title: 'Bulk Operation Complete',
        description: `${result.successful} of ${result.total} operations completed successfully`
      });

      onOperationComplete?.(result);
    } catch (error) {
      setProgress(prev => prev ? { ...prev, status: 'failed' } : null);
      toast({
        title: 'Bulk Operation Failed',
        description: error instanceof Error ? error.message : 'Operation failed',
        variant: 'destructive'
      });
    }
  }, [
    isValidOperation,
    selectedOperation,
    selectedUserIds,
    selectedGroupId,
    handleAssignGroup,
    handleRemoveGroup,
    handleProcessWeb3Wallets,
    handleExportData,
    onOperationComplete,
    toast
  ]);

  const handleConfirmOperation = useCallback(() => {
    setShowConfirmDialog(false);
    executeOperation();
  }, [executeOperation]);

  // Helper functions
  const convertToCSV = (data: any[]): string => {
    if (data.length === 0) return '';
    
    const headers = Object.keys(data[0]);
    const rows = data.map(row => 
      headers.map(header => {
        const value = row[header];
        if (Array.isArray(value)) {
          return JSON.stringify(value);
        }
        return String(value || '');
      }).join(',')
    );
    
    return [headers.join(','), ...rows].join('\n');
  };

  const downloadCSV = (csv: string, filename: string) => {
    const blob = new Blob([csv], { type: 'text/csv;charset=utf-8;' });
    const link = document.createElement('a');
    const url = URL.createObjectURL(blob);
    link.setAttribute('href', url);
    link.setAttribute('download', filename);
    link.style.visibility = 'hidden';
    document.body.appendChild(link);
    link.click();
    document.body.removeChild(link);
  };

  return (
    <div className={cn('space-y-6', className)}>
      {/* Header */}
      <div className="flex items-center justify-between">
        <div>
          <h2 className="text-xl font-semibold text-gray-900">Bulk Operations</h2>
          <p className="text-sm text-gray-600">
            Perform bulk operations on {selectedUserIds.length} selected user(s)
          </p>
        </div>
        <Badge variant="outline" className="text-sm">
          {selectedUserIds.length} users selected
        </Badge>
      </div>

      {selectedUserIds.length === 0 ? (
        <Alert>
          <Users className="h-4 w-4" />
          <AlertDescription>
            Select users from the user management interface to perform bulk operations.
          </AlertDescription>
        </Alert>
      ) : (
        <div className="space-y-6">
          {/* Operation Selection */}
          <Card className={adminCardVariants({ variant: 'default' })}>
            <CardHeader>
              <CardTitle className="flex items-center gap-2">
                <Activity className="h-5 w-5" />
                Select Operation
              </CardTitle>
            </CardHeader>
            <CardContent className="space-y-4">
              <div>
                <Label>Operation Type</Label>
                <Select 
                  value={selectedOperation} 
                  onValueChange={(value: BulkOperationType) => setSelectedOperation(value)}
                >
                  <SelectTrigger>
                    <SelectValue placeholder="Choose an operation" />
                  </SelectTrigger>
                  <SelectContent>
                    {BULK_OPERATIONS.map((operation) => (
                      <SelectItem key={operation.type} value={operation.type}>
                        <div className="flex items-center gap-2">
                          {operation.icon}
                          <div>
                            <div className="font-medium">{operation.label}</div>
                            <div className="text-xs text-gray-500">{operation.description}</div>
                          </div>
                        </div>
                      </SelectItem>
                    ))}
                  </SelectContent>
                </Select>
              </div>

              {selectedOperationData?.requiresGroup && (
                <div>
                  <Label>Target Group</Label>
                  <Select value={selectedGroupId} onValueChange={setSelectedGroupId}>
                    <SelectTrigger>
                      <SelectValue placeholder="Select a group" />
                    </SelectTrigger>
                    <SelectContent>
                      {groups.map((group) => (
                        <SelectItem key={group.id} value={group.id}>
                          <div className="flex items-center gap-2">
                            {group.is_system_group ? (
                              <Shield className="h-3 w-3 text-yellow-600" />
                            ) : (
                              <Users className="h-3 w-3 text-blue-600" />
                            )}
                            {group.name}
                          </div>
                        </SelectItem>
                      ))}
                    </SelectContent>
                  </Select>
                </div>
              )}

              {selectedOperation === 'assign_group' && (
                <div>
                  <Label>Expiry (days, optional)</Label>
                  <Input
                    type="number"
                    value={expiryDays}
                    onChange={(e) => setExpiryDays(e.target.value)}
                    placeholder="Leave empty for no expiry"
                    min="1"
                  />
                </div>
              )}

              <div>
                <Label>Reason (optional)</Label>
                <Textarea
                  value={reason}
                  onChange={(e) => setReason(e.target.value)}
                  placeholder="Optional reason for this operation"
                  rows={3}
                />
              </div>
            </CardContent>
          </Card>

          {/* Execute Button */}
          <div className="flex justify-end">
            <Button
              onClick={() => setShowConfirmDialog(true)}
              disabled={!isValidOperation || progress?.status === 'running'}
              className={adminButtonVariants({ variant: 'primary' })}
            >
              {selectedOperationData?.icon}
              Execute Operation
            </Button>
          </div>

          {/* Progress Display */}
          {progress && (
            <Card className={adminCardVariants({ variant: 'default' })}>
              <CardHeader>
                <CardTitle className="flex items-center gap-2">
                  {progress.status === 'running' ? (
                    <Play className="h-5 w-5 text-blue-600" />
                  ) : progress.status === 'completed' ? (
                    <CheckCircle className="h-5 w-5 text-green-600" />
                  ) : (
                    <XCircle className="h-5 w-5 text-red-600" />
                  )}
                  Operation Progress
                </CardTitle>
              </CardHeader>
              <CardContent className="space-y-4">
                <div>
                  <div className="flex justify-between text-sm mb-1">
                    <span>Progress: {progress.current} of {progress.total}</span>
                    <span>{Math.round((progress.current / progress.total) * 100)}%</span>
                  </div>
                  <Progress value={(progress.current / progress.total) * 100} />
                </div>

                {progress.errors.length > 0 && (
                  <Alert variant="destructive">
                    <AlertTriangle className="h-4 w-4" />
                    <AlertDescription>
                      {progress.errors.length} error(s) occurred during processing
                    </AlertDescription>
                  </Alert>
                )}

                <div className="grid grid-cols-3 gap-4 text-sm">
                  <div className="text-center">
                    <div className="text-2xl font-bold text-blue-600">{progress.current}</div>
                    <div className="text-gray-500">Processed</div>
                  </div>
                  <div className="text-center">
                    <div className="text-2xl font-bold text-green-600">
                      {progress.results.filter(r => r.status === 'success').length}
                    </div>
                    <div className="text-gray-500">Successful</div>
                  </div>
                  <div className="text-center">
                    <div className="text-2xl font-bold text-red-600">
                      {progress.results.filter(r => r.status === 'error').length}
                    </div>
                    <div className="text-gray-500">Failed</div>
                  </div>
                </div>
              </CardContent>
            </Card>
          )}
        </div>
      )}

      {/* Confirmation Dialog */}
      <Dialog open={showConfirmDialog} onOpenChange={setShowConfirmDialog}>
        <DialogContent>
          <DialogHeader>
            <DialogTitle>Confirm Bulk Operation</DialogTitle>
          </DialogHeader>
          <div className="space-y-4">
            <Alert>
              <AlertTriangle className="h-4 w-4" />
              <AlertDescription>
                You are about to perform "{selectedOperationData?.label}" on {selectedUserIds.length} user(s).
                This action cannot be undone.
              </AlertDescription>
            </Alert>
            
            {selectedOperationData?.requiresGroup && selectedGroupId && (
              <div className="text-sm">
                <strong>Target Group:</strong> {groups.find(g => g.id === selectedGroupId)?.name}
              </div>
            )}
            
            {reason && (
              <div className="text-sm">
                <strong>Reason:</strong> {reason}
              </div>
            )}
          </div>
          <DialogFooter>
            <Button variant="outline" onClick={() => setShowConfirmDialog(false)}>
              Cancel
            </Button>
            <Button onClick={handleConfirmOperation} variant="destructive">
              Confirm Operation
            </Button>
          </DialogFooter>
        </DialogContent>
      </Dialog>
    </div>
  );
}

export default GroupBasedBulkOperations;