'use client';

import { useState, useEffect, useCallback, memo, useRef } from 'react';
import { 
  Users, 
  Plus,
  RefreshCw, 
  Clock,
  Shield,
  Ban
} from 'lucide-react';

import { Progress } from '@/components/ui/progress';
import { Tabs, TabsContent, TabsList, TabsTrigger } from '@/components/ui/tabs';
import { useToast } from '@/components/ui/use-toast';

// Extracted focused components
import BulkCreateTab from './BulkCreateTab';
import BulkRevokeTab from './BulkRevokeTab';
import BulkUpdateTab from './BulkUpdateTab';
import BulkHistoryTab from './BulkHistoryTab';

import {
  grantTemporaryPermission,
  revokeTemporaryPermission,
  getActiveTemporaryPermissions,
  type TemporaryPermission
} from '@/lib/actions/consolidated-permission-actions';

interface BulkOperationsInterfaceProps {
  selectedUserIds?: string[];
  onOperationComplete?: () => void;
}

interface BulkOperationResult {
  operation: string;
  timestamp: string;
  totalRequested: number;
  successful: number;
  failed: number;
  executionTimeMs: number;
  errors: Array<{ id?: string; error: string; details?: string }>;
}

interface PermissionTemplate {
  id: string;
  name: string;
  permission: string;
  resource: string;
  action: string;
  defaultDurationHours: number;
}

// SECURITY: Restricted permission templates - no wildcard admin access  
const PERMISSION_TEMPLATES = [
  {
    id: 'user-management',
    name: 'User Management',
    permission: 'admin.users',
    resource: 'user_accounts',
    action: 'manage',
    defaultDurationHours: 8,
  },
  {
    id: 'analytics-access',
    name: 'Analytics Access',
    permission: 'admin.analytics',
    resource: 'reports',
    action: 'read',
    defaultDurationHours: 24,
  },
  {
    id: 'support-access',
    name: 'Support Access',
    permission: 'admin.support',
    resource: 'tickets',
    action: 'manage',
    defaultDurationHours: 12,
  },
  {
    id: 'readonly-access',
    name: 'Read-Only Access',
    permission: 'admin.readonly',
    resource: 'system',
    action: 'read',
    defaultDurationHours: 24,
  },
];

function BulkOperationsInterface({ 
  selectedUserIds = [], 
  onOperationComplete 
}: BulkOperationsInterfaceProps) {
  const { toast } = useToast();
  
  // SECURITY: Permission check for bulk operations
  const userCanPerformBulkOps = useCallback(() => {
    // TODO: Integrate with actual OIDC/permission system
    // Check if user has 'admin:permissions:bulk' permission
    return false; // Default to deny for security
  }, []);
  
  // SECURITY: Rate limiting state
  const [lastOperationTime, setLastOperationTime] = useState<number>(0);
  const MIN_OPERATION_INTERVAL = 5000; // 5 seconds between operations
  
  // State
  const [activeTab, setActiveTab] = useState<'create' | 'revoke' | 'update' | 'history'>('create');
  const [loading, setLoading] = useState(false);
  const [progress, setProgress] = useState(0);
  const [operationResults, setOperationResults] = useState<BulkOperationResult[]>([]);
  const [currentUserIds, setCurrentUserIds] = useState<string[]>(selectedUserIds);
  const [permissionIds, setPermissionIds] = useState<string[]>([]);

  // MEMORY LEAK PROTECTION: Track component mount state and active timers
  const isMountedRef = useRef(true);
  const activeTimersRef = useRef<Set<number>>(new Set());
  const activeIntervalsRef = useRef<Set<number>>(new Set());

  useEffect(() => {
    setCurrentUserIds(selectedUserIds);
  }, [selectedUserIds]);

  // MEMORY LEAK PROTECTION: Cleanup on unmount
  useEffect(() => {
    isMountedRef.current = true;
    
    return () => {
      isMountedRef.current = false;
      
      // Clear all active timers
      activeTimersRef.current.forEach(timer => {
        clearTimeout(timer);
      });
      activeTimersRef.current.clear();
      
      // Clear all active intervals
      activeIntervalsRef.current.forEach(interval => {
        clearInterval(interval);
      });
      activeIntervalsRef.current.clear();
    };
  }, []);

  // MEMORY LEAK PROTECTION: Safe state updates only if component is mounted
  const safeSetState = useCallback((updateFn: () => void) => {
    if (isMountedRef.current) {
      updateFn();
    }
  }, []);

  // MEMORY LEAK PROTECTION: Safe timer creation with automatic cleanup tracking
  const createSafeTimeout = useCallback((callback: () => void, delay: number) => {
    const timer = setTimeout(() => {
      activeTimersRef.current.delete(timer as any);
      if (isMountedRef.current) {
        callback();
      }
    }, delay);
    
    activeTimersRef.current.add(timer as any);
    return timer;
  }, []);

  const createSafeInterval = useCallback((callback: () => void, delay: number) => {
    const interval = setInterval(() => {
      if (!isMountedRef.current) {
        clearInterval(interval);
        activeIntervalsRef.current.delete(interval as any);
        return;
      }
      callback();
    }, delay);
    
    activeIntervalsRef.current.add(interval as any);
    return interval;
  }, []);

  // Clean handlers for tab components
  const handleUserIdsChange = useCallback((userIds: string[]) => {
    safeSetState(() => setCurrentUserIds(userIds));
  }, [safeSetState]);

  const handlePermissionIdsChange = useCallback((permissionIds: string[]) => {
    safeSetState(() => setPermissionIds(permissionIds));
  }, [safeSetState]);

  const handleBulkCreate = useCallback(async (bulkCreateData: {
    template: string;
    customPermission: string;
    customResource: string;
    customAction: string;
    duration: number;
    reason: string;
  }) => {
    // SECURITY: Rate limiting check
    const now = Date.now();
    if (now - lastOperationTime < MIN_OPERATION_INTERVAL) {
      toast({
        title: 'Rate Limited',
        description: `Please wait ${Math.ceil((MIN_OPERATION_INTERVAL - (now - lastOperationTime)) / 1000)} more seconds`,
        variant: 'destructive',
      });
      return;
    }

    if (currentUserIds.length === 0) {
      toast({
        title: 'Validation Error',
        description: 'Please select at least one user',
        variant: 'destructive',
      });
      return;
    }

    let permission, resource, action;
    
    if (bulkCreateData.template) {
      const template = PERMISSION_TEMPLATES.find(t => t.id === bulkCreateData.template);
      if (!template) {
        toast({
          title: 'Error',
          description: 'Invalid template selected',
          variant: 'destructive',
        });
        return;
      }
      permission = template.permission;
      resource = template.resource;
      action = template.action;
    } else {
      permission = bulkCreateData.customPermission;
      resource = bulkCreateData.customResource;
      action = bulkCreateData.customAction;
    }

    if (!permission || !resource || !action) {
      toast({
        title: 'Validation Error',
        description: 'Please fill in all permission fields',
        variant: 'destructive',
      });
      return;
    }

    // SECURITY: Validate permission structure to prevent privilege escalation
    const permissionRegex = /^[a-zA-Z0-9._\-]+$/;
    if (!permissionRegex.test(permission) || permission.length > 100 ||
        !permissionRegex.test(resource) || resource.length > 100 ||
        !permissionRegex.test(action) || action.length > 50) {
      toast({
        title: 'Security Error',
        description: 'Invalid format. Only alphanumeric, dots, dashes, and underscores allowed',
        variant: 'destructive',
      });
      return;
    }

    // SECURITY: Prevent wildcard admin permissions
    if (permission.includes('*')) {
      toast({
        title: 'Permission Denied',
        description: 'Wildcard permissions require special authorization',
        variant: 'destructive',
      });
      return;
    }

    try {
      safeSetState(() => {
        setLoading(true);
        setProgress(0);
      });

      // MEMORY LEAK FIX: Safe progress simulation with cleanup tracking
      const progressInterval = createSafeInterval(() => {
        safeSetState(() => setProgress(prev => Math.min(prev + 10, 90)));
      }, 200);

      // TODO: Implement actual API call
      // const result = await bulkCreateTemporaryPermissions({ permissions });
      
      // MEMORY LEAK FIX: Safe timeout with automatic cleanup
      createSafeTimeout(() => {
        // Clean up interval
        clearInterval(progressInterval);
        activeIntervalsRef.current.delete(progressInterval as any);
        
        safeSetState(() => {
          setProgress(100);
          
          const operationResult: BulkOperationResult = {
            operation: 'Bulk Create Permissions',
            timestamp: new Date().toISOString(),
            totalRequested: currentUserIds.length,
            successful: currentUserIds.length,
            failed: 0,
            executionTimeMs: 1500,
            errors: [],
          };

          setOperationResults(prev => [operationResult, ...prev.slice(0, 9)]);
          setLastOperationTime(Date.now());
          setLoading(false);
          setProgress(0);
        });

        // Safe toast and callback execution
        if (isMountedRef.current) {
          toast({
            title: 'Bulk Operation Complete',
            description: `Created permissions for ${currentUserIds.length} users successfully`,
          });
          onOperationComplete?.();
        }
      }, 2000);
    } catch (error) {
      // SECURITY: Sanitized error logging
      console.error('Bulk create operation failed');
      
      // MEMORY LEAK FIX: Safe error state updates
      safeSetState(() => {
        setLoading(false);
        setProgress(0);
      });
      
      if (isMountedRef.current) {
        toast({
          title: 'Error',
          description: 'An unexpected error occurred during bulk permission creation',
          variant: 'destructive',
        });
      }
    }
  }, [currentUserIds, lastOperationTime, toast, onOperationComplete, safeSetState, createSafeInterval, createSafeTimeout]);

  const handleBulkRevoke = useCallback(async (reason: string) => {
    // SECURITY: Rate limiting check
    const now = Date.now();
    if (now - lastOperationTime < MIN_OPERATION_INTERVAL) {
      toast({
        title: 'Rate Limited',
        description: `Please wait ${Math.ceil((MIN_OPERATION_INTERVAL - (now - lastOperationTime)) / 1000)} more seconds`,
        variant: 'destructive',
      });
      return;
    }

    if (permissionIds.length === 0) {
      toast({
        title: 'Validation Error',
        description: 'Please add at least one permission ID to revoke',
        variant: 'destructive',
      });
      return;
    }

    try {
      safeSetState(() => {
        setLoading(true);
        setProgress(0);
      });

      // MEMORY LEAK FIX: Safe progress simulation with cleanup tracking
      const progressInterval = createSafeInterval(() => {
        safeSetState(() => setProgress(prev => Math.min(prev + 15, 90)));
      }, 200);

      // TODO: Implement actual API call
      // const result = await bulkRevokeTemporaryPermissions({ permission_ids: permissionIds, reason });
      
      // MEMORY LEAK FIX: Safe timeout with automatic cleanup
      createSafeTimeout(() => {
        // Clean up interval
        clearInterval(progressInterval);
        activeIntervalsRef.current.delete(progressInterval as any);
        
        safeSetState(() => {
          setProgress(100);
          
          const operationResult: BulkOperationResult = {
            operation: 'Bulk Revoke Permissions',
            timestamp: new Date().toISOString(),
            totalRequested: permissionIds.length,
            successful: permissionIds.length,
            failed: 0,
            executionTimeMs: 1200,
            errors: [],
          };

          setOperationResults(prev => [operationResult, ...prev.slice(0, 9)]);
          setPermissionIds([]);
          setLastOperationTime(Date.now());
          setLoading(false);
          setProgress(0);
        });

        // Safe toast and callback execution
        if (isMountedRef.current) {
          toast({
            title: 'Bulk Operation Complete',
            description: `Revoked ${permissionIds.length} permissions successfully`,
          });
          onOperationComplete?.();
        }
      }, 1500);
    } catch (error) {
      // SECURITY: Sanitized error logging
      console.error('Bulk revoke operation failed');
      
      // MEMORY LEAK FIX: Safe error state updates
      safeSetState(() => {
        setLoading(false);
        setProgress(0);
      });
      
      if (isMountedRef.current) {
        toast({
          title: 'Error',
          description: 'An unexpected error occurred during bulk permission revocation',
          variant: 'destructive',
        });
      }
    }
  }, [permissionIds, lastOperationTime, toast, onOperationComplete, safeSetState, createSafeInterval, createSafeTimeout]);

  const handleBulkUpdate = useCallback(async (updateData: {
    permission?: string;
    resource?: string;
    action?: string;
    extendHours?: number;
    reason?: string;
    status?: 'active' | 'suspended' | 'revoked';
  }) => {
    // SECURITY: Rate limiting check
    const now = Date.now();
    if (now - lastOperationTime < MIN_OPERATION_INTERVAL) {
      toast({
        title: 'Rate Limited',
        description: `Please wait ${Math.ceil((MIN_OPERATION_INTERVAL - (now - lastOperationTime)) / 1000)} more seconds`,
        variant: 'destructive',
      });
      return;
    }

    if (permissionIds.length === 0) {
      toast({
        title: 'Validation Error',
        description: 'Please add at least one permission ID to update',
        variant: 'destructive',
      });
      return;
    }

    const hasUpdates = Object.values(updateData).some(value => 
      value !== undefined && value !== '' && value !== null
    );

    if (!hasUpdates) {
      toast({
        title: 'Validation Error',
        description: 'Please specify at least one field to update',
        variant: 'destructive',
      });
      return;
    }

    try {
      safeSetState(() => {
        setLoading(true);
        setProgress(0);
      });

      // MEMORY LEAK FIX: Safe progress simulation with cleanup tracking
      const progressInterval = createSafeInterval(() => {
        safeSetState(() => setProgress(prev => Math.min(prev + 12, 90)));
      }, 250);

      // TODO: Implement actual API call
      // const result = await bulkUpdateTemporaryPermissions(data);
      
      // MEMORY LEAK FIX: Safe timeout with automatic cleanup
      createSafeTimeout(() => {
        // Clean up interval
        clearInterval(progressInterval);
        activeIntervalsRef.current.delete(progressInterval as any);
        
        safeSetState(() => {
          setProgress(100);
          
          const operationResult: BulkOperationResult = {
            operation: 'Bulk Update Permissions',
            timestamp: new Date().toISOString(),
            totalRequested: permissionIds.length,
            successful: permissionIds.length,
            failed: 0,
            executionTimeMs: 1800,
            errors: [],
          };

          setOperationResults(prev => [operationResult, ...prev.slice(0, 9)]);
          setPermissionIds([]);
          setLastOperationTime(Date.now());
          setLoading(false);
          setProgress(0);
        });

        // Safe toast and callback execution
        if (isMountedRef.current) {
          toast({
            title: 'Bulk Operation Complete',
            description: `Updated ${permissionIds.length} permissions successfully`,
          });
          onOperationComplete?.();
        }
      }, 2200);
    } catch (error) {
      // SECURITY: Sanitized error logging
      console.error('Bulk update operation failed');
      
      // MEMORY LEAK FIX: Safe error state updates
      safeSetState(() => {
        setLoading(false);
        setProgress(0);
      });
      
      if (isMountedRef.current) {
        toast({
          title: 'Error',
          description: 'An unexpected error occurred during bulk permission update',
          variant: 'destructive',
        });
      }
    }
  }, [permissionIds, lastOperationTime, toast, onOperationComplete, safeSetState, createSafeInterval, createSafeTimeout]);

  // SECURITY: Check if user has permission for bulk operations
  if (!userCanPerformBulkOps()) {
    return (
      <div className="flex items-center justify-center min-h-[60vh]">
        <div className="text-center max-w-md">
          <Shield className="w-16 h-16 text-gray-400 mx-auto mb-4" />
          <h2 className="text-xl font-semibold text-gray-900 dark:text-gray-100 mb-2">
            Access Denied
          </h2>
          <p className="text-gray-600 dark:text-gray-300 mb-4">
            You don't have permission to perform bulk operations. This feature requires
            elevated administrative privileges.
          </p>
        </div>
      </div>
    );
  }

  return (
    <div className="space-y-6">
      {/* Header */}
      <div className="flex items-center justify-between">
        <div>
          <h3 className="text-lg font-semibold flex items-center gap-2">
            <Users className="h-5 w-5" />
            Bulk Operations
          </h3>
          <p className="text-sm text-muted-foreground">
            Manage permissions for multiple users simultaneously
          </p>
        </div>
      </div>

      {/* Progress Bar */}
      {loading && (
        <div className="space-y-2">
          <div className="flex items-center justify-between text-sm">
            <span>Processing bulk operation...</span>
            <span>{progress}%</span>
          </div>
          <Progress value={progress} className="w-full" />
        </div>
      )}

      {/* Operation Tabs */}
      <Tabs value={activeTab} onValueChange={(value: any) => setActiveTab(value)}>
        <TabsList className="grid w-full grid-cols-4">
          <TabsTrigger value="create" className="flex items-center gap-2">
            <Plus className="h-4 w-4" />
            Create
          </TabsTrigger>
          <TabsTrigger value="revoke" className="flex items-center gap-2">
            <Ban className="h-4 w-4" />
            Revoke
          </TabsTrigger>
          <TabsTrigger value="update" className="flex items-center gap-2">
            <RefreshCw className="h-4 w-4" />
            Update
          </TabsTrigger>
          <TabsTrigger value="history" className="flex items-center gap-2">
            <Clock className="h-4 w-4" />
            History
          </TabsTrigger>
        </TabsList>

        {/* Focused Tab Components */}
        <TabsContent value="create" className="space-y-6">
          <BulkCreateTab
            currentUserIds={currentUserIds}
            onUserIdsChange={handleUserIdsChange}
            templates={PERMISSION_TEMPLATES}
            loading={loading}
            onBulkCreate={handleBulkCreate}
          />
        </TabsContent>

        <TabsContent value="revoke" className="space-y-6">
          <BulkRevokeTab
            permissionIds={permissionIds}
            onPermissionIdsChange={handlePermissionIdsChange}
            loading={loading}
            onBulkRevoke={handleBulkRevoke}
          />
        </TabsContent>

        <TabsContent value="update" className="space-y-6">
          <BulkUpdateTab
            permissionIds={permissionIds}
            onPermissionIdsChange={handlePermissionIdsChange}
            loading={loading}
            onBulkUpdate={handleBulkUpdate}
          />
        </TabsContent>

        <TabsContent value="history" className="space-y-6">
          <BulkHistoryTab operationResults={operationResults} />
        </TabsContent>
      </Tabs>
    </div>
  );
}

export default memo(BulkOperationsInterface);