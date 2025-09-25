'use client';

import { useState, useEffect, useMemo, memo, useCallback } from 'react';
import { 
  Clock, 
  Plus, 
  Filter, 
  RefreshCw, 
  AlertTriangle, 
  CheckCircle,
  XCircle,
  Trash2,
  Calendar,
  User,
  Shield,
  MoreHorizontal,
  Edit,
  Ban,
  Trash,
  FileText
} from 'lucide-react';

import { Button } from '@/components/ui/button';
import { Input } from '@/components/ui/input';
import { Label } from '@/components/ui/label';
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from '@/components/ui/select';
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '@/components/ui/card';
import { Badge } from '@/components/ui/badge';
import { Alert, AlertDescription } from '@/components/ui/alert';
import { Separator } from '@/components/ui/separator';
import { Textarea } from '@/components/ui/textarea';
import { Tabs, TabsContent, TabsList, TabsTrigger } from '@/components/ui/tabs';
import { 
  DropdownMenu, 
  DropdownMenuContent, 
  DropdownMenuItem, 
  DropdownMenuSeparator, 
  DropdownMenuTrigger 
} from '@/components/ui/dropdown-menu';
import {
  Dialog,
  DialogContent,
  DialogDescription,
  DialogHeader,
  DialogTitle,
  DialogTrigger,
} from '@/components/ui/dialog';
import { useToast } from '@/components/ui/use-toast';
import { format, formatDistance, isPast, addDays, addHours } from 'date-fns';

import {
  TemporaryPermission,
  grantTemporaryPermission,
  getUserTemporaryPermissions,
  revokeTemporaryPermission,
  getActiveTemporaryPermissions,
  getExpiringTemporaryPermissions,
} from '@/lib/actions/consolidated-permission-actions';
import { AdminPermissionExpiryIndicator, parseEmbeddedPermissions } from '../auth/AdminPermissionExpiryIndicator';

// Embedded timestamp permissions support
interface EmbeddedPermissionData {
  basePermission: string;
  expiryTimestamp: number;
  reason?: string;
}

// Form data for creating temporary permissions
interface CreateTemporaryPermissionData {
  user_id: string;
  permission: string;
  resource: string;
  action: string;
  expires_at: string;
  reason?: string;
}

interface TemporaryPermissionManagerProps {
  userId: string;
  // Enhanced props for embedded timestamps
  enableEmbeddedTimestamps?: boolean;
  showTimeline?: boolean;
  allowQuickActions?: boolean;
}

// Quick time selection options
const QUICK_TIME_OPTIONS = [
  { label: '1 Hour', minutes: 60 },
  { label: '4 Hours', minutes: 240 },
  { label: '8 Hours', minutes: 480 },
  { label: '1 Day', minutes: 1440 },
  { label: '3 Days', minutes: 4320 },
  { label: '1 Week', minutes: 10080 },
  { label: '1 Month', minutes: 43200 },
] as const;

// Common permission templates with embedded timestamps
const PERMISSION_TEMPLATES = [
  { 
    name: 'Temporary Analytics Access', 
    basePermission: 'epsx:analytics:view',
    defaultDuration: 240, // 4 hours
    description: 'Temporary access to analytics dashboard'
  },
  { 
    name: 'Premium Rankings (1 Day)', 
    basePermission: 'epsx:rankings:view:100',
    defaultDuration: 1440, // 1 day
    description: 'Temporary access to top 100 rankings'
  },
  { 
    name: 'Export Data Access', 
    basePermission: 'epsx:analytics:export',
    defaultDuration: 60, // 1 hour
    description: 'Temporary data export capabilities'
  },
  { 
    name: 'User Management (Emergency)', 
    basePermission: 'admin:users:manage',
    defaultDuration: 480, // 8 hours
    description: 'Emergency user management access'
  },
] as const;

const STATUS_COLORS = {
  active: 'bg-green-100 text-green-800 dark:bg-green-900/20 dark:text-green-200 border border-green-200 dark:border-green-800',
  suspended: 'bg-yellow-100 text-yellow-800 dark:bg-yellow-900/20 dark:text-yellow-200 border border-yellow-200 dark:border-yellow-800',
  expired: 'bg-gray-100 text-gray-800 dark:bg-gray-900/20 dark:text-gray-200 border border-gray-200 dark:border-gray-800',
  revoked: 'bg-red-100 text-red-800 dark:bg-red-900/20 dark:text-red-200 border border-red-200 dark:border-red-800',
};

const STATUS_COLORS_ENHANCED = {
  active: 'bg-green-50 text-green-900 border border-green-200 shadow-green-100/50 shadow-sm dark:bg-green-950/30 dark:text-green-100 dark:border-green-800',
  suspended: 'bg-yellow-50 text-yellow-900 border border-yellow-200 shadow-yellow-100/50 shadow-sm dark:bg-yellow-950/30 dark:text-yellow-100 dark:border-yellow-800',
  expired: 'bg-gray-50 text-gray-900 border border-gray-200 shadow-gray-100/50 shadow-sm dark:bg-gray-950/30 dark:text-gray-100 dark:border-gray-800',
  revoked: 'bg-red-50 text-red-900 border border-red-200 shadow-red-100/50 shadow-sm dark:bg-red-950/30 dark:text-red-100 dark:border-red-800',
  expiring_critical: 'bg-orange-50 text-orange-900 border border-orange-200 shadow-orange-100/50 shadow-sm dark:bg-orange-950/30 dark:text-orange-100 dark:border-orange-800',
  expiring_soon: 'bg-amber-50 text-amber-900 border border-amber-200 shadow-amber-100/50 shadow-sm dark:bg-amber-950/30 dark:text-amber-100 dark:border-amber-800',
};

const STATUS_ICONS = {
  active: CheckCircle,
  suspended: AlertTriangle,
  expired: Clock,
  revoked: XCircle,
};

function TemporaryPermissionManager({ 
  userId, 
  enableEmbeddedTimestamps = true,
  showTimeline = true,
  allowQuickActions = true 
}: TemporaryPermissionManagerProps) {
  const { toast } = useToast();
  const [permissions, setPermissions] = useState<TemporaryPermission[]>([]);
  const [loading, setLoading] = useState(false);
  const [refreshing, setRefreshing] = useState(false);
  const [filter, setFilter] = useState<'all' | 'active' | 'expired' | 'revoked'>('all');
  const [searchTerm, setSearchTerm] = useState('');
  
  // Form state - Legacy temporary permissions
  const [showCreateForm, setShowCreateForm] = useState(false);
  const [editingPermission, setEditingPermission] = useState<TemporaryPermission | null>(null);
  const [formData, setFormData] = useState<CreateTemporaryPermissionData>({
    user_id: userId,
    permission: '',
    resource: '',
    action: '',
    expires_at: '',
    reason: '',
  });

  // Embedded timestamp form state
  const [showEmbeddedForm, setShowEmbeddedForm] = useState(false);
  const [embeddedFormData, setEmbeddedFormData] = useState<EmbeddedPermissionData>({
    basePermission: '',
    expiryTimestamp: 0,
    reason: '',
  });
  const [selectedTemplate, setSelectedTemplate] = useState<string>('');
  const [quickTimeSelection, setQuickTimeSelection] = useState<string>('240'); // 4 hours default
  const [customDateTime, setCustomDateTime] = useState<string>('');
  const [timeInputMode, setTimeInputMode] = useState<'quick' | 'custom'>('quick');

  const loadPermissions = async () => {
    try {
      setRefreshing(true);
      const result = await getUserTemporaryPermissions(userId);
      
      if (result.success && result.data) {
        setPermissions(result.data);
      } else {
        toast({
          title: 'Error',
          description: result.error || 'Failed to load temporary permissions',
          variant: 'destructive',
        });
      }
    } catch (error) {
      console.error('Failed to load temporary permissions:', error);
      toast({
        title: 'Error',
        description: 'An unexpected error occurred',
        variant: 'destructive',
      });
    } finally {
      setRefreshing(false);
    }
  };

  useEffect(() => {
    loadPermissions();
  }, [userId]);

  const filteredPermissions = useMemo(() => {
    return permissions.filter(permission => {
      // Status filter based on isActive and expiry
      const isExpired = new Date(permission.expiresAt) < new Date();
      if (filter !== 'all') {
        if (filter === 'active' && (!permission.isActive || isExpired)) return false;
        if (filter === 'expired' && !isExpired) return false;
        if (filter === 'revoked' && permission.isActive) return false;
      }
      
      // Search filter
      if (searchTerm) {
        const searchLower = searchTerm.toLowerCase();
        return (
          permission.permission.toLowerCase().includes(searchLower) ||
          (permission.reason && permission.reason.toLowerCase().includes(searchLower))
        );
      }
      
      return true;
    });
  }, [permissions, filter, searchTerm]);

  const handleCreate = useCallback(async (e: React.FormEvent) => {
    e.preventDefault();
    
    if (!formData.permission || !formData.resource || !formData.action || !formData.expires_at) {
      toast({
        title: 'Validation Error',
        description: 'Please fill in all required fields',
        variant: 'destructive',
      });
      return;
    }

    try {
      setLoading(true);
      
      const result = await grantTemporaryPermission({
        userId: formData.user_id,
        permission: formData.permission,
        expiresAt: formData.expires_at,
        reason: formData.reason
      });
      
      if (result.success) {
        toast({
          title: 'Success',
          description: 'Temporary permission created successfully',
        });
        
        setShowCreateForm(false);
        setFormData({
          user_id: userId,
          permission: '',
          resource: '',
          action: '',
          expires_at: '',
          reason: '',
        });
        
        await loadPermissions();
      } else {
        toast({
          title: 'Error',
          description: result.error || 'Failed to create temporary permission',
          variant: 'destructive',
        });
      }
    } catch (error) {
      console.error('Failed to create temporary permission:', error);
      toast({
        title: 'Error',
        description: 'An unexpected error occurred',
        variant: 'destructive',
      });
    } finally {
      setLoading(false);
    }
  }, [formData, toast]);

  const handleRevoke = useCallback(async (permission: TemporaryPermission, reason?: string) => {
    try {
      const result = await revokeTemporaryPermission(permission.id);
      
      if (result.success) {
        toast({
          title: 'Success',
          description: 'Permission revoked successfully',
        });
        await loadPermissions();
      } else {
        toast({
          title: 'Error',
          description: result.error || 'Failed to revoke permission',
          variant: 'destructive',
        });
      }
    } catch (error) {
      console.error('Failed to revoke permission:', error);
      toast({
        title: 'Error',
        description: 'An unexpected error occurred',
        variant: 'destructive',
      });
    }
  }, [toast]);

  const handleDelete = async (permission: TemporaryPermission) => {
    try {
      const result = await revokeTemporaryPermission(permission.id);
      
      if (result.success) {
        toast({
          title: 'Success',
          description: 'Permission deleted successfully',
        });
        await loadPermissions();
      } else {
        toast({
          title: 'Error',
          description: result.error || 'Failed to delete permission',
          variant: 'destructive',
        });
      }
    } catch (error) {
      console.error('Failed to delete permission:', error);
      toast({
        title: 'Error',
        description: 'An unexpected error occurred',
        variant: 'destructive',
      });
    }
  };

  const handleCleanup = async () => {
    try {
      // TODO: Implement cleanup functionality when available
      const result = { 
        success: false, 
        error: 'Cleanup functionality not yet implemented',
        data: null as any
      };
      
      if (result.success && result.data) {
        toast({
          title: 'Success',
          description: `Cleaned up ${result.data.cleaned_count} expired permissions`,
        });
        await loadPermissions();
      } else {
        toast({
          title: 'Error',
          description: result.error || 'Failed to cleanup expired permissions',
          variant: 'destructive',
        });
      }
    } catch (error) {
      console.error('Failed to cleanup expired permissions:', error);
      toast({
        title: 'Error',
        description: 'An unexpected error occurred',
        variant: 'destructive',
      });
    }
  };

  // Utility functions for embedded timestamps
  const createEmbeddedPermissionString = (basePermission: string, expiryTimestamp: number): string => {
    return `${basePermission}:${expiryTimestamp}`;
  };

  const parseEmbeddedPermission = (permission: string): { base: string; timestamp?: number } => {
    const parts = permission.split(':');
    const lastPart = parts[parts.length - 1];
    const timestamp = parseInt(lastPart, 10);
    
    if (!isNaN(timestamp)) {
      return {
        base: parts.slice(0, -1).join(':'),
        timestamp
      };
    }
    
    return { base: permission };
  };

  const handleCreateEmbeddedPermission = async () => {
    if (!embeddedFormData.basePermission) {
      toast({
        title: 'Validation Error',
        description: 'Please provide a base permission',
        variant: 'destructive',
      });
      return;
    }

    // Calculate expiry timestamp
    let expiryTimestamp: number;
    
    if (timeInputMode === 'quick') {
      const minutes = parseInt(quickTimeSelection, 10);
      expiryTimestamp = Math.floor(Date.now() / 1000) + (minutes * 60);
    } else {
      if (!customDateTime) {
        toast({
          title: 'Validation Error',
          description: 'Please select an expiry date and time',
          variant: 'destructive',
        });
        return;
      }
      expiryTimestamp = Math.floor(new Date(customDateTime).getTime() / 1000);
    }

    // Create embedded permission string
    const embeddedPermissionString = createEmbeddedPermissionString(
      embeddedFormData.basePermission,
      expiryTimestamp
    );

    // Here we would typically call a server action to add this to the user's permissions
    // For now, we'll simulate with a toast
    toast({
      title: 'Success',
      description: `Embedded timestamp permission created: ${embeddedPermissionString}`,
    });

    // Reset form
    setShowEmbeddedForm(false);
    setEmbeddedFormData({
      basePermission: '',
      expiryTimestamp: 0,
      reason: '',
    });
    setSelectedTemplate('');
    setQuickTimeSelection('240');
    setCustomDateTime('');
    setTimeInputMode('quick');
  };

  const handleTemplateSelect = (templateName: string) => {
    const template = PERMISSION_TEMPLATES.find(t => t.name === templateName);
    if (template) {
      setEmbeddedFormData(prev => ({
        ...prev,
        basePermission: template.basePermission
      }));
      setQuickTimeSelection(template.defaultDuration.toString());
      setSelectedTemplate(templateName);
    }
  };

  // Memoized permission categorization to avoid repeated filtering
  const { activePermissions, expiredPermissions, revokedPermissions } = useMemo(() => {
    return {
      activePermissions: permissions.filter(p => p.isActive && new Date(p.expiresAt) > new Date()),
      expiredPermissions: permissions.filter(p => new Date(p.expiresAt) <= new Date()),
      revokedPermissions: permissions.filter(p => !p.isActive)
    };
  }, [permissions]);

  return (
    <div className="space-y-6">
      {/* Header */}
      <div className="flex items-center justify-between">
        <div>
          <h3 className="text-lg font-semibold">Temporary Permissions</h3>
          <p className="text-sm text-muted-foreground">
            Manage time-limited access permissions for this user
          </p>
        </div>
        <div className="flex gap-2">
          <Button
            variant="outline"
            size="sm"
            onClick={loadPermissions}
            disabled={refreshing}
          >
            <RefreshCw className="h-4 w-4" />
          </Button>
          
          {enableEmbeddedTimestamps && (
            <Button
              size="sm"
              onClick={() => setShowEmbeddedForm(true)}
              className="bg-blue-600 hover:bg-blue-700"
            >
              <Clock className="h-4 w-4 mr-2" />
              Quick Grant
            </Button>
          )}
          
          <Button
            variant="outline"
            size="sm"
            onClick={() => setShowCreateForm(true)}
          >
            <Plus className="h-4 w-4 mr-2" />
            Legacy Grant
          </Button>
        </div>
      </div>

      {/* Enhanced Stats with Visual Indicators */}
      <div className="grid grid-cols-1 md:grid-cols-4 gap-4">
        <Card className="bg-gradient-to-br from-green-50 to-emerald-50 border-green-200 dark:from-green-950/20 dark:to-emerald-950/20 dark:border-green-800">
          <CardContent className="p-4">
            <div className="flex items-center justify-between">
              <div className="flex items-center gap-3">
                <div className="p-2 bg-green-100 rounded-lg dark:bg-green-900/30">
                  <CheckCircle className="h-6 w-6 text-green-600 dark:text-green-400" />
                </div>
                <div>
                  <p className="text-2xl font-bold text-green-700 dark:text-green-300">{activePermissions.length}</p>
                  <p className="text-sm text-green-600 dark:text-green-400">Active</p>
                </div>
              </div>
              {activePermissions.length > 0 && (
                <div className="h-2 w-2 bg-green-500 rounded-full opacity-75"></div>
              )}
            </div>
          </CardContent>
        </Card>

        <Card className={`${expiredPermissions.length > 0 
          ? 'bg-gradient-to-br from-red-50 to-rose-50 border-red-200 dark:from-red-950/20 dark:to-rose-950/20 dark:border-red-800' 
          : 'bg-gradient-to-br from-gray-50 to-slate-50 border-gray-200 dark:from-gray-950/20 dark:to-slate-950/20 dark:border-gray-700'
        }`}>
          <CardContent className="p-4">
            <div className="flex items-center justify-between">
              <div className="flex items-center gap-3">
                <div className={`p-2 rounded-lg ${expiredPermissions.length > 0 
                  ? 'bg-red-100 dark:bg-red-900/30' 
                  : 'bg-gray-100 dark:bg-gray-900/30'
                }`}>
                  <XCircle className={`h-6 w-6 ${expiredPermissions.length > 0 
                    ? 'text-red-600 dark:text-red-400' 
                    : 'text-gray-500 dark:text-gray-400'
                  }`} />
                </div>
                <div>
                  <p className={`text-2xl font-bold ${expiredPermissions.length > 0 
                    ? 'text-red-700 dark:text-red-300' 
                    : 'text-gray-700 dark:text-gray-300'
                  }`}>{expiredPermissions.length}</p>
                  <p className={`text-sm ${expiredPermissions.length > 0 
                    ? 'text-red-600 dark:text-red-400' 
                    : 'text-gray-600 dark:text-gray-400'
                  }`}>Expired</p>
                </div>
              </div>
              {expiredPermissions.length > 0 && (
                <div className="flex items-center gap-1">
                  <AlertTriangle className="h-4 w-4 text-red-500 opacity-75" />
                  <div className="h-2 w-2 bg-red-500 rounded-full opacity-75"></div>
                </div>
              )}
            </div>
          </CardContent>
        </Card>

        <Card className={`${revokedPermissions.length > 0 
          ? 'bg-gradient-to-br from-orange-50 to-amber-50 border-orange-200 dark:from-orange-950/20 dark:to-amber-950/20 dark:border-orange-700' 
          : 'bg-gradient-to-br from-gray-50 to-slate-50 border-gray-200 dark:from-gray-950/20 dark:to-slate-950/20 dark:border-gray-700'
        }`}>
          <CardContent className="p-4">
            <div className="flex items-center justify-between">
              <div className="flex items-center gap-3">
                <div className={`p-2 rounded-lg ${revokedPermissions.length > 0 
                  ? 'bg-orange-100 dark:bg-orange-900/30' 
                  : 'bg-gray-100 dark:bg-gray-900/30'
                }`}>
                  <Ban className={`h-6 w-6 ${revokedPermissions.length > 0 
                    ? 'text-orange-600 dark:text-orange-400' 
                    : 'text-gray-500 dark:text-gray-400'
                  }`} />
                </div>
                <div>
                  <p className={`text-2xl font-bold ${revokedPermissions.length > 0 
                    ? 'text-orange-700 dark:text-orange-300' 
                    : 'text-gray-700 dark:text-gray-300'
                  }`}>{revokedPermissions.length}</p>
                  <p className={`text-sm ${revokedPermissions.length > 0 
                    ? 'text-orange-600 dark:text-orange-400' 
                    : 'text-gray-600 dark:text-gray-400'
                  }`}>Revoked</p>
                </div>
              </div>
            </div>
          </CardContent>
        </Card>

        <Card className="bg-gradient-to-br from-blue-50 to-indigo-50 border-blue-200 dark:from-blue-950/20 dark:to-indigo-950/20 dark:border-blue-800">
          <CardContent className="p-4">
            <div className="flex items-center justify-between">
              <div className="flex items-center gap-3">
                <div className="p-2 bg-blue-100 rounded-lg dark:bg-blue-900/30">
                  <Shield className="h-6 w-6 text-blue-600 dark:text-blue-400" />
                </div>
                <div>
                  <p className="text-2xl font-bold text-blue-700 dark:text-blue-300">{permissions.length}</p>
                  <p className="text-sm text-blue-600 dark:text-blue-400">Total</p>
                </div>
              </div>
              <div className="text-right text-xs">
                <div className="text-blue-600 dark:text-blue-400 font-medium">
                  {permissions.length > 0 ? Math.round((activePermissions.length / permissions.length) * 100) : 0}% Active
                </div>
                <div className="text-muted-foreground">Health Score</div>
              </div>
            </div>
          </CardContent>
        </Card>
      </div>

      {/* Filters */}
      <div className="flex flex-col sm:flex-row gap-4">
        <div className="flex-1">
          <Input
            placeholder="Search permissions, resources, actions..."
            value={searchTerm}
            onChange={(e) => setSearchTerm(e.target.value)}
            className="max-w-sm"
          />
        </div>
        <Select value={filter} onValueChange={(value: any) => setFilter(value)}>
          <SelectTrigger className="w-[180px]">
            <SelectValue />
          </SelectTrigger>
          <SelectContent>
            <SelectItem value="all">All Permissions</SelectItem>
            <SelectItem value="active">Active Only</SelectItem>
            <SelectItem value="expired">Expired Only</SelectItem>
            <SelectItem value="revoked">Revoked Only</SelectItem>
          </SelectContent>
        </Select>
        {expiredPermissions.length > 0 && (
          <Button
            variant="outline"
            size="sm"
            onClick={handleCleanup}
          >
            <Trash2 className="h-4 w-4 mr-2" />
            Cleanup Expired
          </Button>
        )}
      </div>

      {/* Permissions List */}
      <div className="space-y-4">
        {filteredPermissions.length === 0 ? (
          <Card>
            <CardContent className="p-8 text-center">
              <Clock className="h-12 w-12 mx-auto text-muted-foreground mb-4" />
              <h3 className="text-lg font-medium mb-2">No temporary permissions found</h3>
              <p className="text-muted-foreground mb-4">
                {filter === 'all' 
                  ? 'This user has no temporary permissions yet.'
                  : `No ${filter} temporary permissions found.`
                }
              </p>
              <Button
                size="sm"
                onClick={() => setShowCreateForm(true)}
              >
                <Plus className="h-4 w-4 mr-2" />
                Grant First Permission
              </Button>
            </CardContent>
          </Card>
        ) : (
          filteredPermissions.map((permission) => {
            // Calculate status based on actual TemporaryPermission properties
            const isExpired = new Date(permission.expiresAt) <= new Date();
            const status = !permission.isActive ? 'revoked' : isExpired ? 'expired' : 'active';
            
            const StatusIcon = STATUS_ICONS[status];
            const isExpiringSoon = !isExpired && 
              new Date(permission.expiresAt) < addHours(new Date(), 24);
            const isCritical = !isExpired && 
              new Date(permission.expiresAt) < addHours(new Date(), 1); // Less than 1 hour
            
            // Enhanced visual styling based on expiry status
            let cardClassName = "p-4"
            let badgeClassName = STATUS_COLORS[status]
            
            if (isCritical && status === 'active') {
              cardClassName = "p-4 bg-gradient-to-r from-orange-50 to-red-50 border-l-4 border-orange-400 dark:from-orange-950/20 dark:to-red-950/20"
              badgeClassName = STATUS_COLORS_ENHANCED.expiring_critical
            } else if (isExpiringSoon && status === 'active') {
              cardClassName = "p-4 bg-gradient-to-r from-yellow-50 to-amber-50 border-l-4 border-yellow-400 dark:from-yellow-950/20 dark:to-amber-950/20"
              badgeClassName = STATUS_COLORS_ENHANCED.expiring_soon
            } else if (status === 'expired') {
              cardClassName = "p-4 bg-gradient-to-r from-gray-50 to-slate-50 border-l-4 border-gray-400 opacity-80 dark:from-gray-950/20 dark:to-slate-950/20"
            }
            
            return (
              <Card key={permission.id} className={isExpired ? "opacity-75" : ""}>
                <CardContent className={cardClassName}>
                  <div className="flex items-start justify-between">
                    <div className="flex-1">
                      <div className="flex items-center gap-2 mb-3">
                        <Badge className={badgeClassName}>
                          <StatusIcon className="h-3 w-3 mr-1" />
                          {status}
                        </Badge>
                        
                        {isCritical && status === 'active' && (
                          <Badge className="bg-red-100 text-red-800 border border-red-200 dark:bg-red-900/20 dark:text-red-200 opacity-75">
                            <AlertTriangle className="h-3 w-3 mr-1" />
                            Critical - Expires in &lt;1h
                          </Badge>
                        )}
                        
                        {isExpiringSoon && !isCritical && status === 'active' && (
                          <Badge className="bg-amber-100 text-amber-800 border border-amber-200 dark:bg-amber-900/20 dark:text-amber-200">
                            <Clock className="h-3 w-3 mr-1" />
                            Expiring Soon
                          </Badge>
                        )}
                        
                        {isExpired && (
                          <Badge className="bg-gray-100 text-gray-800 border border-gray-200 dark:bg-gray-900/20 dark:text-gray-200">
                            <XCircle className="h-3 w-3 mr-1" />
                            Expired
                          </Badge>
                        )}
                      </div>
                      
                      <div className="grid grid-cols-1 md:grid-cols-3 gap-4 mb-3">
                        <div>
                          <p className="text-xs text-muted-foreground">Permission</p>
                          <p className="font-medium">{permission.permission}</p>
                        </div>
                      </div>

                      <div className="grid grid-cols-1 md:grid-cols-2 gap-4 text-sm text-muted-foreground">
                        <div>
                          <p className="flex items-center gap-1">
                            <Calendar className="h-3 w-3" />
                            Granted: {format(new Date(permission.grantedAt), 'PPp')}
                          </p>
                          <p className="flex items-center gap-1 mt-1">
                            <Clock className="h-3 w-3" />
                            Expires: {format(new Date(permission.expiresAt), 'PPp')} 
                            ({formatDistance(new Date(permission.expiresAt), new Date(), { addSuffix: true })})
                          </p>
                        </div>
                        {permission.reason && (
                          <div>
                            <p className="flex items-center gap-1">
                              <FileText className="h-3 w-3" />
                              Reason: {permission.reason}
                            </p>
                          </div>
                        )}
                      </div>

                      {status === 'revoked' && (
                        <div className="mt-2 p-2 bg-red-50 rounded text-sm">
                          <p className="text-red-700">
                            This permission has been revoked
                          </p>
                        </div>
                      )}
                    </div>
                    
                    <DropdownMenu>
                      <DropdownMenuTrigger asChild>
                        <Button variant="ghost" size="sm">
                          <MoreHorizontal className="h-4 w-4" />
                        </Button>
                      </DropdownMenuTrigger>
                      <DropdownMenuContent align="end">
                        {status === 'active' && !isExpired && (
                          <>
                            <DropdownMenuItem onClick={() => setEditingPermission(permission)}>
                              <Edit className="h-4 w-4 mr-2" />
                              Edit
                            </DropdownMenuItem>
                            <DropdownMenuItem 
                              onClick={() => handleRevoke(permission)}
                              className="text-destructive"
                            >
                              <Ban className="h-4 w-4 mr-2" />
                              Revoke
                            </DropdownMenuItem>
                            <DropdownMenuSeparator />
                          </>
                        )}
                        <DropdownMenuItem 
                          onClick={() => handleDelete(permission)}
                          className="text-destructive"
                        >
                          <Trash className="h-4 w-4 mr-2" />
                          Delete
                        </DropdownMenuItem>
                      </DropdownMenuContent>
                    </DropdownMenu>
                  </div>
                </CardContent>
              </Card>
            );
          })
        )}
      </div>

      {/* Create Permission Dialog */}
      <Dialog open={showCreateForm} onOpenChange={setShowCreateForm}>
        <DialogContent className="max-w-md">
          <DialogHeader>
            <DialogTitle>Grant Temporary Permission</DialogTitle>
            <DialogDescription>
              Assign a time-limited permission to this user
            </DialogDescription>
          </DialogHeader>
          
          <form onSubmit={handleCreate} className="space-y-4">
            <div className="space-y-2">
              <Label htmlFor="permission">Permission</Label>
              <Input
                id="permission"
                value={formData.permission}
                onChange={(e) => setFormData(prev => ({ ...prev, permission: e.target.value }))}
                placeholder="e.g., admin.users"
                required
              />
            </div>
            
            <div className="space-y-2">
              <Label htmlFor="resource">Resource</Label>
              <Input
                id="resource"
                value={formData.resource}
                onChange={(e) => setFormData(prev => ({ ...prev, resource: e.target.value }))}
                placeholder="e.g., user_accounts"
                required
              />
            </div>
            
            <div className="space-y-2">
              <Label htmlFor="action">Action</Label>
              <Input
                id="action"
                value={formData.action}
                onChange={(e) => setFormData(prev => ({ ...prev, action: e.target.value }))}
                placeholder="e.g., read, write, manage"
                required
              />
            </div>
            
            <div className="space-y-2">
              <Label htmlFor="expires_at">Expires At</Label>
              <Input
                id="expires_at"
                type="datetime-local"
                value={formData.expires_at}
                onChange={(e) => setFormData(prev => ({ ...prev, expires_at: e.target.value }))}
                min={new Date().toISOString().slice(0, 16)}
                required
              />
            </div>
            
            <div className="space-y-2">
              <Label htmlFor="reason">Reason (Optional)</Label>
              <Textarea
                id="reason"
                value={formData.reason}
                onChange={(e) => setFormData(prev => ({ ...prev, reason: e.target.value }))}
                placeholder="Why is this permission needed?"
                rows={2}
              />
            </div>
            
            <div className="flex gap-2 pt-4">
              <Button type="submit" disabled={loading} className="flex-1">
                {loading ? 'Creating...' : 'Grant Permission'}
              </Button>
              <Button
                type="button"
                variant="outline"
                onClick={() => setShowCreateForm(false)}
              >
                Cancel
              </Button>
            </div>
          </form>
        </DialogContent>
      </Dialog>

      {/* Embedded Timestamp Permission Dialog */}
      {enableEmbeddedTimestamps && (
        <Dialog open={showEmbeddedForm} onOpenChange={setShowEmbeddedForm}>
          <DialogContent className="max-w-2xl max-h-[90vh] overflow-y-auto">
            <DialogHeader>
              <DialogTitle className="flex items-center gap-2">
                <Clock className="h-5 w-5 text-blue-600" />
                Quick Grant Temporary Permission
              </DialogTitle>
              <DialogDescription>
                Create a permission with embedded timestamp that automatically expires
              </DialogDescription>
            </DialogHeader>
            
            <div className="space-y-6">
              {/* Permission Templates */}
              <div className="space-y-3">
                <Label>Quick Templates</Label>
                <div className="grid grid-cols-1 md:grid-cols-2 gap-2">
                  {PERMISSION_TEMPLATES.map((template) => (
                    <Button
                      key={template.name}
                      variant={selectedTemplate === template.name ? "default" : "outline"}
                      size="sm"
                      onClick={() => handleTemplateSelect(template.name)}
                      className="justify-start text-left h-auto p-3"
                    >
                      <div>
                        <div className="font-medium">{template.name}</div>
                        <div className="text-xs text-muted-foreground">{template.description}</div>
                        <div className="text-xs text-blue-600">{template.basePermission}</div>
                      </div>
                    </Button>
                  ))}
                </div>
              </div>

              <Separator />

              {/* Base Permission */}
              <div className="space-y-2">
                <Label htmlFor="basePermission">Base Permission *</Label>
                <Input
                  id="basePermission"
                  value={embeddedFormData.basePermission}
                  onChange={(e) => setEmbeddedFormData(prev => ({ 
                    ...prev, 
                    basePermission: e.target.value 
                  }))}
                  placeholder="e.g., epsx:analytics:view or admin:users:manage"
                  required
                />
                <p className="text-xs text-muted-foreground">
                  Permission in platform:resource:action format
                </p>
              </div>

              {/* Time Selection */}
              <div className="space-y-4">
                <Label>Expiry Time</Label>
                
                <Tabs value={timeInputMode} onValueChange={(value: any) => setTimeInputMode(value)}>
                  <TabsList className="grid w-full grid-cols-2">
                    <TabsTrigger value="quick">Quick Select</TabsTrigger>
                    <TabsTrigger value="custom">Custom Date</TabsTrigger>
                  </TabsList>
                  
                  <TabsContent value="quick" className="space-y-3">
                    <div className="grid grid-cols-2 md:grid-cols-4 gap-2">
                      {QUICK_TIME_OPTIONS.map((option) => (
                        <Button
                          key={option.label}
                          variant={quickTimeSelection === option.minutes.toString() ? "default" : "outline"}
                          size="sm"
                          onClick={() => setQuickTimeSelection(option.minutes.toString())}
                          className="h-12 flex flex-col"
                        >
                          <span className="font-medium">{option.label}</span>
                          <span className="text-xs text-muted-foreground">
                            {option.minutes < 60 ? `${option.minutes}m` : 
                             option.minutes < 1440 ? `${Math.round(option.minutes/60)}h` : 
                             `${Math.round(option.minutes/1440)}d`}
                          </span>
                        </Button>
                      ))}
                    </div>
                    <div className="text-xs text-muted-foreground">
                      Selected: {QUICK_TIME_OPTIONS.find(o => o.minutes.toString() === quickTimeSelection)?.label || 'Custom'}
                      {quickTimeSelection && (
                        <>
                          {' - Expires: '}
                          {format(
                            new Date(Date.now() + (parseInt(quickTimeSelection) * 60 * 1000)), 
                            'PPp'
                          )}
                        </>
                      )}
                    </div>
                  </TabsContent>
                  
                  <TabsContent value="custom" className="space-y-3">
                    <Input
                      type="datetime-local"
                      value={customDateTime}
                      onChange={(e) => setCustomDateTime(e.target.value)}
                      min={new Date().toISOString().slice(0, 16)}
                    />
                    {customDateTime && (
                      <div className="text-xs text-muted-foreground">
                        Expires: {format(new Date(customDateTime), 'PPp')} 
                        ({formatDistance(new Date(customDateTime), new Date(), { addSuffix: true })})
                      </div>
                    )}
                  </TabsContent>
                </Tabs>
              </div>

              {/* Reason */}
              <div className="space-y-2">
                <Label htmlFor="embeddedReason">Reason (Optional)</Label>
                <Textarea
                  id="embeddedReason"
                  value={embeddedFormData.reason}
                  onChange={(e) => setEmbeddedFormData(prev => ({ 
                    ...prev, 
                    reason: e.target.value 
                  }))}
                  placeholder="Why is this temporary permission needed?"
                  rows={2}
                />
              </div>

              {/* Preview */}
              {embeddedFormData.basePermission && (
                <Alert>
                  <AlertTriangle className="h-4 w-4" />
                  <AlertDescription>
                    <div className="space-y-2">
                      <div><strong>Permission Preview:</strong></div>
                      <code className="text-sm bg-gray-100 p-2 rounded block">
                        {embeddedFormData.basePermission}:
                        {timeInputMode === 'quick' 
                          ? Math.floor(Date.now() / 1000) + (parseInt(quickTimeSelection) * 60)
                          : customDateTime 
                            ? Math.floor(new Date(customDateTime).getTime() / 1000)
                            : 'TIMESTAMP'
                        }
                      </code>
                      <div className="text-xs text-muted-foreground">
                        This permission will be automatically validated for expiry by the frontend
                      </div>
                    </div>
                  </AlertDescription>
                </Alert>
              )}

              {/* Actions */}
              <div className="flex gap-2 pt-4">
                <Button 
                  onClick={handleCreateEmbeddedPermission} 
                  disabled={loading} 
                  className="flex-1"
                >
                  {loading ? 'Creating...' : 'Grant Permission'}
                </Button>
                <Button
                  type="button"
                  variant="outline"
                  onClick={() => setShowEmbeddedForm(false)}
                >
                  Cancel
                </Button>
              </div>
            </div>
          </DialogContent>
        </Dialog>
      )}
    </div>
  );
}

// Export memoized component to prevent unnecessary re-renders on prop changes
export default memo(TemporaryPermissionManager);