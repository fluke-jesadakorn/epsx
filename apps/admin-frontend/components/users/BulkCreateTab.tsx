'use client';

import { memo, useState, useCallback } from 'react';
import { Plus, XCircle } from 'lucide-react';
import { Button } from '@/components/ui/button';
import { Input } from '@/components/ui/input';
import { Label } from '@/components/ui/label';
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from '@/components/ui/select';
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '@/components/ui/card';
import { Badge } from '@/components/ui/badge';
import { useToast } from '@/components/ui/use-toast';

interface PermissionTemplate {
  id: string;
  name: string;
  permission: string;
  resource: string;
  action: string;
  defaultDurationHours: number;
}

interface BulkCreateTabProps {
  currentUserIds: string[];
  onUserIdsChange: (userIds: string[]) => void;
  templates: PermissionTemplate[];
  loading: boolean;
  onBulkCreate: (data: {
    template: string;
    customPermission: string;
    customResource: string;
    customAction: string;
    duration: number;
    reason: string;
  }) => void;
}

function BulkCreateTab({
  currentUserIds,
  onUserIdsChange,
  templates,
  loading,
  onBulkCreate,
}: BulkCreateTabProps) {
  const { toast } = useToast();
  const [userIdInput, setUserIdInput] = useState('');
  
  // Bulk Create State
  const [bulkCreateData, setBulkCreateData] = useState<{
    template: string;
    customPermission: string;
    customResource: string;
    customAction: string;
    duration: number;
    reason: string;
  }>({
    template: '',
    customPermission: '',
    customResource: '',
    customAction: '',
    duration: 8,
    reason: '',
  });

  const addUserId = useCallback(() => {
    const trimmedId = userIdInput.trim();
    
    // SECURITY: Validate user ID format
    if (!trimmedId) return;
    
    // SECURITY: Basic UUID format validation to prevent injection
    const uuidRegex = /^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$/i;
    const isEmail = /^[^\s@]+@[^\s@]+\.[^\s@]+$/.test(trimmedId);
    
    if (!uuidRegex.test(trimmedId) && !isEmail) {
      toast({
        title: 'Invalid User ID',
        description: 'User ID must be a valid UUID or email format',
        variant: 'destructive',
      });
      return;
    }
    
    // SECURITY: Prevent duplicate entries
    if (currentUserIds.includes(trimmedId)) {
      toast({
        title: 'Duplicate Entry',
        description: 'User ID already added',
        variant: 'destructive',
      });
      return;
    }
    
    // SECURITY: Limit total entries to prevent DoS
    if (currentUserIds.length >= 100) {
      toast({
        title: 'Limit Exceeded',
        description: 'Maximum 100 users allowed per bulk operation',
        variant: 'destructive',
      });
      return;
    }
    
    onUserIdsChange([...currentUserIds, trimmedId]);
    setUserIdInput('');
  }, [userIdInput, currentUserIds, onUserIdsChange, toast]);

  const removeUserId = useCallback((userId: string) => {
    onUserIdsChange(currentUserIds.filter(id => id !== userId));
  }, [currentUserIds, onUserIdsChange]);

  const handleCreate = useCallback(() => {
    onBulkCreate(bulkCreateData);
  }, [bulkCreateData, onBulkCreate]);

  return (
    <Card>
      <CardHeader>
        <CardTitle className="flex items-center gap-2">
          <Plus className="h-5 w-5" />
          Bulk Create Temporary Permissions
        </CardTitle>
        <CardDescription>
          Grant temporary permissions to multiple users at once
        </CardDescription>
      </CardHeader>
      <CardContent className="space-y-6">
        {/* User Selection */}
        <div className="space-y-4">
          <Label>Target Users ({currentUserIds.length} selected)</Label>
          <div className="flex gap-2">
            <Input
              placeholder="Enter user ID"
              value={userIdInput}
              onChange={(e) => setUserIdInput(e.target.value)}
              onKeyPress={(e) => e.key === 'Enter' && addUserId()}
            />
            <Button onClick={addUserId} size="sm">
              Add
            </Button>
          </div>
          {currentUserIds.length > 0 && (
            <div className="flex flex-wrap gap-2">
              {currentUserIds.map((userId) => (
                <Badge key={userId} variant="secondary" className="flex items-center gap-1">
                  {userId}
                  <button onClick={() => removeUserId(userId)}>
                    <XCircle className="h-3 w-3" />
                  </button>
                </Badge>
              ))}
            </div>
          )}
        </div>

        {/* Permission Configuration */}
        <div className="space-y-4">
          <Label>Permission Configuration</Label>
          
          {/* Template Selection */}
          <div className="space-y-2">
            <Label htmlFor="template">Use Template</Label>
            <Select
              value={bulkCreateData.template}
              onValueChange={(value) => setBulkCreateData(prev => ({ 
                ...prev, 
                template: value,
                customPermission: '',
                customResource: '',
                customAction: ''
              }))}
            >
              <SelectTrigger>
                <SelectValue placeholder="Select a permission template (optional)" />
              </SelectTrigger>
              <SelectContent>
                {templates.map((template) => (
                  <SelectItem key={template.id} value={template.id}>
                    <div>
                      <div className="font-medium">{template.name}</div>
                      <div className="text-xs text-muted-foreground">
                        {template.permission} on {template.resource}
                      </div>
                    </div>
                  </SelectItem>
                ))}
              </SelectContent>
            </Select>
          </div>

          {/* Custom Permission Fields */}
          {!bulkCreateData.template && (
            <div className="grid grid-cols-1 md:grid-cols-3 gap-4">
              <div className="space-y-2">
                <Label htmlFor="permission">Permission</Label>
                <Input
                  id="permission"
                  value={bulkCreateData.customPermission}
                  onChange={(e) => setBulkCreateData(prev => ({ 
                    ...prev, 
                    customPermission: e.target.value 
                  }))}
                  placeholder="e.g., admin.users"
                />
              </div>
              <div className="space-y-2">
                <Label htmlFor="resource">Resource</Label>
                <Input
                  id="resource"
                  value={bulkCreateData.customResource}
                  onChange={(e) => setBulkCreateData(prev => ({ 
                    ...prev, 
                    customResource: e.target.value 
                  }))}
                  placeholder="e.g., user_accounts"
                />
              </div>
              <div className="space-y-2">
                <Label htmlFor="action">Action</Label>
                <Input
                  id="action"
                  value={bulkCreateData.customAction}
                  onChange={(e) => setBulkCreateData(prev => ({ 
                    ...prev, 
                    customAction: e.target.value 
                  }))}
                  placeholder="e.g., manage"
                />
              </div>
            </div>
          )}

          {/* Duration and Reason */}
          <div className="grid grid-cols-1 md:grid-cols-2 gap-4">
            <div className="space-y-2">
              <Label htmlFor="duration">Duration (hours)</Label>
              <Input
                id="duration"
                type="number"
                min="1"
                max="168"
                value={bulkCreateData.duration}
                onChange={(e) => setBulkCreateData(prev => ({ 
                  ...prev, 
                  duration: parseInt(e.target.value) || 8 
                }))}
              />
            </div>
            <div className="space-y-2">
              <Label htmlFor="reason">Reason</Label>
              <Input
                id="reason"
                value={bulkCreateData.reason}
                onChange={(e) => setBulkCreateData(prev => ({ 
                  ...prev, 
                  reason: e.target.value 
                }))}
                placeholder="Why are these permissions needed?"
              />
            </div>
          </div>
        </div>

        <Button 
          onClick={handleCreate} 
          disabled={loading || currentUserIds.length === 0}
          className="w-full"
        >
          {loading ? 'Creating Permissions...' : `Create Permissions for ${currentUserIds.length} Users`}
        </Button>
      </CardContent>
    </Card>
  );
}

export default memo(BulkCreateTab);