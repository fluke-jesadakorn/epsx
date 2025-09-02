'use client';

import React, { useState, useCallback } from 'react';
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '@/components/ui/card';
import { Button } from '@/components/ui/button';
import { Input } from '@/components/ui/input';
import { Label } from '@/components/ui/label';
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from '@/components/ui/select';
import { Badge } from '@/components/ui/badge';
import { Textarea } from '@/components/ui/textarea';
import { Checkbox } from '@/components/ui/checkbox';
import { Progress } from '@/components/ui/progress';
import { Alert, AlertDescription } from '@/components/ui/alert';
import { 
  Dialog, 
  DialogContent, 
  DialogDescription, 
  DialogFooter, 
  DialogHeader, 
  DialogTitle 
} from '@/components/ui/dialog';
import { 
  Table, 
  TableBody, 
  TableCell, 
  TableHead, 
  TableHeader, 
  TableRow 
} from '@/components/ui/table';
import { 
  Users, 
  Shield, 
  ShieldPlus, 
  ShieldX, 
  Clock, 
  AlertTriangle, 
  CheckCircle, 
  XCircle,
  Upload,
  Download,
  Play,
  Pause,
  RotateCcw,
  FileText
} from 'lucide-react';
import { useAdminGranularPermissions } from '@/hooks/useGranularPermissions';
import { 
  BulkPermissionRequest,
  BulkOperationResult,
  PermissionSource,
  PermissionTemplate
} from '@/types/granular-permissions';

interface BulkPermissionManagerProps {
  userIds: string[];
  onBulkOperation?: (result: BulkOperationResult) => void;
  availableTemplates?: PermissionTemplate[];
  className?: string;
}

interface BulkOperation {
  type: 'grant' | 'revoke' | 'template';
  permission?: string;
  platform?: string;
  resource?: string;
  action?: string;
  source?: PermissionSource;
  expires_at?: number;
  template_id?: string;
  reason?: string;
}

interface OperationProgress {
  total: number;
  completed: number;
  successful: number;
  failed: number;
  is_running: boolean;
  current_user?: string;
  details: {
    user_id: string;
    status: 'pending' | 'success' | 'failed';
    error?: string;
  }[];
}

export function BulkPermissionManager({ 
  userIds, 
  onBulkOperation,
  availableTemplates = [],
  className = ''
}: BulkPermissionManagerProps) {
  const { 
    bulkGrantPermissions, 
    bulkRevokePermissions, 
    applyPermissionTemplate,
    loading, 
    error 
  } = useAdminGranularPermissions();

  const [operation, setOperation] = useState<BulkOperation>({
    type: 'grant',
    platform: 'epsx',
    source: 'Admin'
  });
  const [isConfirmDialogOpen, setIsConfirmDialogOpen] = useState(false);
  const [progress, setProgress] = useState<OperationProgress | null>(null);
  const [isPreviewMode, setIsPreviewMode] = useState(false);
  const [selectedUsers, setSelectedUsers] = useState<Set<string>>(new Set(userIds));

  // Mock user data for display (in real app, this would come from props or API)
  const mockUsers = userIds.map(id => ({
    user_id: id,
    email: `user-${id}@example.com`,
    display_name: `User ${id}`
  }));

  // Toggle user selection
  const toggleUser = useCallback((userId: string) => {
    const newSelected = new Set(selectedUsers);
    if (newSelected.has(userId)) {
      newSelected.delete(userId);
    } else {
      newSelected.add(userId);
    }
    setSelectedUsers(newSelected);
  }, [selectedUsers]);

  // Select all users
  const selectAllUsers = useCallback(() => {
    setSelectedUsers(new Set(userIds));
  }, [userIds]);

  // Deselect all users
  const deselectAllUsers = useCallback(() => {
    setSelectedUsers(new Set());
  }, []);

  // Build operation preview
  const getOperationPreview = useCallback(() => {
    const selectedUserIds = Array.from(selectedUsers);
    
    switch (operation.type) {
      case 'grant':
        return {
          title: 'Grant Permission',
          description: `Grant "${operation.platform}:${operation.resource}:${operation.action}" to ${selectedUserIds.length} users`,
          permission: `${operation.platform}:${operation.resource || '*'}:${operation.action || '*'}`,
          details: `Source: ${operation.source}, Expires: ${operation.expires_at ? new Date(operation.expires_at * 1000).toLocaleString() : 'Never'}`
        };
      case 'revoke':
        return {
          title: 'Revoke Permission',
          description: `Revoke "${operation.platform}:${operation.resource}:${operation.action}" from ${selectedUserIds.length} users`,
          permission: `${operation.platform}:${operation.resource || '*'}:${operation.action || '*'}`,
          details: `This action cannot be undone`
        };
      case 'template':
        const template = availableTemplates.find(t => t.id === operation.template_id);
        return {
          title: 'Apply Template',
          description: `Apply template "${template?.name}" to ${selectedUserIds.length} users`,
          permission: template?.permissions.join(', ') || 'Unknown permissions',
          details: `${template?.permissions.length || 0} permissions will be granted`
        };
      default:
        return {
          title: 'Unknown Operation',
          description: 'Invalid operation',
          permission: '',
          details: ''
        };
    }
  }, [operation, selectedUsers, availableTemplates]);

  // Execute bulk operation
  const executeBulkOperation = useCallback(async () => {
    const selectedUserIds = Array.from(selectedUsers);
    if (selectedUserIds.length === 0) return;

    // Initialize progress
    setProgress({
      total: selectedUserIds.length,
      completed: 0,
      successful: 0,
      failed: 0,
      is_running: true,
      details: selectedUserIds.map(id => ({ user_id: id, status: 'pending' }))
    });

    try {
      let result: BulkOperationResult;

      switch (operation.type) {
        case 'grant':
          if (!operation.resource || !operation.action) {
            throw new Error('Resource and action are required for grant operation');
          }
          
          const grantRequest: BulkPermissionRequest = {
            user_ids: selectedUserIds,
            permission: `${operation.platform}:${operation.resource}:${operation.action}`,
            expires_at: operation.expires_at,
            source: operation.source!,
            reason: operation.reason
          };
          
          result = await bulkGrantPermissions(grantRequest);
          break;

        case 'revoke':
          if (!operation.resource || !operation.action) {
            throw new Error('Resource and action are required for revoke operation');
          }
          
          result = await bulkRevokePermissions({
            user_ids: selectedUserIds,
            permission: `${operation.platform}:${operation.resource}:${operation.action}`,
            reason: operation.reason
          });
          break;

        case 'template':
          if (!operation.template_id) {
            throw new Error('Template ID is required for template operation');
          }
          
          result = await applyPermissionTemplate(operation.template_id, selectedUserIds);
          break;

        default:
          throw new Error('Invalid operation type');
      }

      // Update progress with final results
      setProgress(prev => prev ? {
        ...prev,
        completed: result.total_requested,
        successful: result.successful,
        failed: result.failed,
        is_running: false,
        details: result.details.map(d => ({
          user_id: d.user_id,
          status: d.success ? 'success' : 'failed',
          error: d.error
        }))
      } : null);

      onBulkOperation?.(result);

    } catch (err) {
      console.error('Bulk operation failed:', err);
      setProgress(prev => prev ? { ...prev, is_running: false } : null);
    }

    setIsConfirmDialogOpen(false);
  }, [operation, selectedUsers, bulkGrantPermissions, bulkRevokePermissions, applyPermissionTemplate, onBulkOperation]);

  const preview = getOperationPreview();
  const progressPercentage = progress ? (progress.completed / progress.total) * 100 : 0;

  return (
    <div className={`space-y-6 ${className}`}>
      {/* Header */}
      <Card>
        <CardHeader>
          <CardTitle className="flex items-center gap-2">
            <Users className="h-5 w-5" />
            Bulk Permission Management
          </CardTitle>
          <CardDescription>
            Manage permissions for multiple users at once
          </CardDescription>
        </CardHeader>
      </Card>

      {/* User Selection */}
      <Card>
        <CardHeader>
          <div className="flex items-center justify-between">
            <CardTitle className="text-lg">Select Users ({selectedUsers.size}/{userIds.length})</CardTitle>
            <div className="flex gap-2">
              <Button variant="outline" size="sm" onClick={selectAllUsers}>
                Select All
              </Button>
              <Button variant="outline" size="sm" onClick={deselectAllUsers}>
                Clear All
              </Button>
            </div>
          </div>
        </CardHeader>
        <CardContent>
          <div className="max-h-60 overflow-y-auto">
            <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-2">
              {mockUsers.map((user) => (
                <div 
                  key={user.user_id} 
                  className="flex items-center space-x-2 p-2 border rounded cursor-pointer hover:bg-muted"
                  onClick={() => toggleUser(user.user_id)}
                >
                  <Checkbox 
                    checked={selectedUsers.has(user.user_id)}
                    onChange={() => toggleUser(user.user_id)}
                  />
                  <div className="flex-1 min-w-0">
                    <div className="text-sm font-medium truncate">
                      {user.display_name || user.email}
                    </div>
                    <div className="text-xs text-muted-foreground truncate">
                      {user.user_id}
                    </div>
                  </div>
                </div>
              ))}
            </div>
          </div>
        </CardContent>
      </Card>

      {/* Operation Configuration */}
      <Card>
        <CardHeader>
          <CardTitle className="text-lg">Operation Configuration</CardTitle>
        </CardHeader>
        <CardContent className="space-y-4">
          {/* Operation Type */}
          <div>
            <Label>Operation Type</Label>
            <Select 
              value={operation.type} 
              onValueChange={(value) => setOperation(prev => ({ ...prev, type: value as 'grant' | 'revoke' | 'template' }))}
            >
              <SelectTrigger>
                <SelectValue />
              </SelectTrigger>
              <SelectContent>
                <SelectItem value="grant">Grant Permission</SelectItem>
                <SelectItem value="revoke">Revoke Permission</SelectItem>
                {availableTemplates.length > 0 && (
                  <SelectItem value="template">Apply Template</SelectItem>
                )}
              </SelectContent>
            </Select>
          </div>

          {operation.type === 'template' ? (
            /* Template Selection */
            <div>
              <Label>Permission Template</Label>
              <Select 
                value={operation.template_id || ''} 
                onValueChange={(value) => setOperation(prev => ({ ...prev, template_id: value }))}
              >
                <SelectTrigger>
                  <SelectValue placeholder="Select a template" />
                </SelectTrigger>
                <SelectContent>
                  {availableTemplates.map((template) => (
                    <SelectItem key={template.id} value={template.id}>
                      <div className="flex flex-col">
                        <span>{template.name}</span>
                        <span className="text-xs text-muted-foreground">
                          {template.permissions.length} permissions
                        </span>
                      </div>
                    </SelectItem>
                  ))}
                </SelectContent>
              </Select>
            </div>
          ) : (
            /* Permission Configuration */
            <>
              <div className="grid grid-cols-3 gap-2">
                <div>
                  <Label>Platform</Label>
                  <Select 
                    value={operation.platform || 'epsx'} 
                    onValueChange={(value) => setOperation(prev => ({ ...prev, platform: value }))}
                  >
                    <SelectTrigger>
                      <SelectValue />
                    </SelectTrigger>
                    <SelectContent>
                      <SelectItem value="epsx">EPSX</SelectItem>
                      <SelectItem value="epsx-pay">EPSX Pay</SelectItem>
                      <SelectItem value="epsx-token">EPSX Token</SelectItem>
                      <SelectItem value="admin">Admin</SelectItem>
                    </SelectContent>
                  </Select>
                </div>
                <div>
                  <Label>Resource</Label>
                  <Input
                    value={operation.resource || ''}
                    onChange={(e) => setOperation(prev => ({ ...prev, resource: e.target.value }))}
                    placeholder="users, analytics, etc."
                  />
                </div>
                <div>
                  <Label>Action</Label>
                  <Input
                    value={operation.action || ''}
                    onChange={(e) => setOperation(prev => ({ ...prev, action: e.target.value }))}
                    placeholder="view, manage, *, etc."
                  />
                </div>
              </div>

              {operation.type === 'grant' && (
                <>
                  <div className="grid grid-cols-2 gap-4">
                    <div>
                      <Label>Source</Label>
                      <Select 
                        value={operation.source || 'Admin'} 
                        onValueChange={(value) => setOperation(prev => ({ ...prev, source: value as PermissionSource }))}
                      >
                        <SelectTrigger>
                          <SelectValue />
                        </SelectTrigger>
                        <SelectContent>
                          <SelectItem value="Admin">Admin</SelectItem>
                          <SelectItem value="Subscription">Subscription</SelectItem>
                          <SelectItem value="Trial">Trial</SelectItem>
                          <SelectItem value="System">System</SelectItem>
                        </SelectContent>
                      </Select>
                    </div>
                    <div>
                      <Label>Expires At (Optional)</Label>
                      <Input
                        type="datetime-local"
                        value={operation.expires_at ? new Date(operation.expires_at * 1000).toISOString().slice(0, 16) : ''}
                        onChange={(e) => setOperation(prev => ({ 
                          ...prev, 
                          expires_at: e.target.value ? Math.floor(new Date(e.target.value).getTime() / 1000) : undefined 
                        }))}
                      />
                    </div>
                  </div>
                </>
              )}
            </>
          )}

          {/* Reason */}
          <div>
            <Label>Reason (Optional)</Label>
            <Textarea
              value={operation.reason || ''}
              onChange={(e) => setOperation(prev => ({ ...prev, reason: e.target.value }))}
              placeholder="Reason for this bulk operation..."
              rows={2}
            />
          </div>

          {/* Preview */}
          <div className="p-3 bg-muted rounded">
            <div className="text-sm font-medium mb-2">Operation Preview:</div>
            <div className="space-y-1">
              <div className="text-sm"><strong>Action:</strong> {preview.title}</div>
              <div className="text-sm"><strong>Targets:</strong> {selectedUsers.size} users</div>
              <div className="text-sm"><strong>Permission:</strong> <code className="text-xs">{preview.permission}</code></div>
              <div className="text-sm"><strong>Details:</strong> {preview.details}</div>
            </div>
          </div>
        </CardContent>
      </Card>

      {/* Progress */}
      {progress && (
        <Card>
          <CardHeader>
            <CardTitle className="flex items-center gap-2">
              {progress.is_running ? (
                <RotateCcw className="h-5 w-5 animate-spin" />
              ) : (
                <CheckCircle className="h-5 w-5 text-green-500" />
              )}
              Operation Progress
            </CardTitle>
          </CardHeader>
          <CardContent>
            <div className="space-y-4">
              <div>
                <div className="flex justify-between text-sm mb-2">
                  <span>Progress: {progress.completed}/{progress.total}</span>
                  <span>{Math.round(progressPercentage)}%</span>
                </div>
                <Progress value={progressPercentage} />
              </div>

              <div className="grid grid-cols-3 gap-4 text-center">
                <div>
                  <div className="text-2xl font-bold text-green-600">{progress.successful}</div>
                  <div className="text-sm text-muted-foreground">Successful</div>
                </div>
                <div>
                  <div className="text-2xl font-bold text-red-600">{progress.failed}</div>
                  <div className="text-sm text-muted-foreground">Failed</div>
                </div>
                <div>
                  <div className="text-2xl font-bold text-gray-600">{progress.total - progress.completed}</div>
                  <div className="text-sm text-muted-foreground">Remaining</div>
                </div>
              </div>

              {/* Detailed Results */}
              {progress.details.some(d => d.status !== 'pending') && (
                <div className="max-h-40 overflow-y-auto">
                  <Table>
                    <TableHeader>
                      <TableRow>
                        <TableHead>User</TableHead>
                        <TableHead>Status</TableHead>
                        <TableHead>Error</TableHead>
                      </TableRow>
                    </TableHeader>
                    <TableBody>
                      {progress.details.filter(d => d.status !== 'pending').map((detail) => (
                        <TableRow key={detail.user_id}>
                          <TableCell>{detail.user_id}</TableCell>
                          <TableCell>
                            {detail.status === 'success' ? (
                              <Badge variant="default" className="bg-green-100 text-green-800">
                                Success
                              </Badge>
                            ) : (
                              <Badge variant="destructive">Failed</Badge>
                            )}
                          </TableCell>
                          <TableCell>
                            {detail.error && (
                              <span className="text-xs text-red-600">{detail.error}</span>
                            )}
                          </TableCell>
                        </TableRow>
                      ))}
                    </TableBody>
                  </Table>
                </div>
              )}
            </div>
          </CardContent>
        </Card>
      )}

      {/* Actions */}
      <Card>
        <CardContent className="pt-6">
          <div className="flex gap-2 justify-end">
            <Button
              variant="outline"
              onClick={() => setIsPreviewMode(!isPreviewMode)}
            >
              <FileText className="h-4 w-4" />
              {isPreviewMode ? 'Hide Preview' : 'Show Preview'}
            </Button>
            <Button
              onClick={() => setIsConfirmDialogOpen(true)}
              disabled={selectedUsers.size === 0 || loading || progress?.is_running}
            >
              {operation.type === 'grant' ? (
                <ShieldPlus className="h-4 w-4" />
              ) : operation.type === 'revoke' ? (
                <ShieldX className="h-4 w-4" />
              ) : (
                <Upload className="h-4 w-4" />
              )}
              Execute Operation
            </Button>
          </div>
        </CardContent>
      </Card>

      {/* Confirmation Dialog */}
      <Dialog open={isConfirmDialogOpen} onOpenChange={setIsConfirmDialogOpen}>
        <DialogContent>
          <DialogHeader>
            <DialogTitle className="flex items-center gap-2">
              <AlertTriangle className="h-5 w-5 text-yellow-500" />
              Confirm Bulk Operation
            </DialogTitle>
            <DialogDescription>
              This action will affect {selectedUsers.size} users. Please review the details below.
            </DialogDescription>
          </DialogHeader>
          
          <div className="space-y-4">
            <Alert>
              <AlertTriangle className="h-4 w-4" />
              <AlertDescription>
                <strong>{preview.title}:</strong> {preview.description}
              </AlertDescription>
            </Alert>

            <div className="p-3 border rounded">
              <div className="text-sm space-y-1">
                <div><strong>Permission:</strong> <code>{preview.permission}</code></div>
                <div><strong>Users Affected:</strong> {selectedUsers.size}</div>
                <div><strong>Details:</strong> {preview.details}</div>
                {operation.reason && <div><strong>Reason:</strong> {operation.reason}</div>}
              </div>
            </div>
          </div>

          <DialogFooter>
            <Button variant="outline" onClick={() => setIsConfirmDialogOpen(false)}>
              Cancel
            </Button>
            <Button 
              onClick={executeBulkOperation}
              disabled={loading}
              variant={operation.type === 'revoke' ? 'destructive' : 'default'}
            >
              {operation.type === 'grant' ? 'Grant Permissions' : 
               operation.type === 'revoke' ? 'Revoke Permissions' : 
               'Apply Template'}
            </Button>
          </DialogFooter>
        </DialogContent>
      </Dialog>
    </div>
  );
}

export default BulkPermissionManager;