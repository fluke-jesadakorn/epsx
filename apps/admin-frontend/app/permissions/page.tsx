'use client';

import { useState, useEffect, useCallback } from 'react';
import { useAuth } from '@/lib/auth';
import { 
  getAllUsers, 
  grantEmbeddedTimestampPermission,
  revokeEmbeddedTimestampPermission,
  updateEmbeddedTimestampPermission,
  getUserPermissionsWithExpiry,
  EmbeddedPermissionData,
  UserOperationResult
} from '@/lib/actions/embedded-permission-actions';
import { AdminFeatureGate } from '@/components/features/AdminFeatureGate';
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '@/components/ui/card';
import { Button } from '@/components/ui/button';
import { Input } from '@/components/ui/input';
import { Label } from '@/components/ui/label';
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from '@/components/ui/select';
import { Textarea } from '@/components/ui/textarea';
import { Badge } from '@/components/ui/badge';
import { Separator } from '@/components/ui/separator';
import { Tabs, TabsContent, TabsList, TabsTrigger } from '@/components/ui/tabs';
import { Dialog, DialogContent, DialogDescription, DialogHeader, DialogTitle, DialogTrigger } from '@/components/ui/dialog';
import { Alert, AlertDescription } from '@/components/ui/alert';
import { toast } from '@/hooks/use-toast';
import { 
  Users, 
  Plus, 
  Clock, 
  Shield, 
  Search,
  Edit,
  Trash2,
  AlertCircle,
  CheckCircle,
  XCircle,
  Calendar,
  User,
  Settings,
  Crown,
  Zap
} from 'lucide-react';
import { formatDistanceToNow } from 'date-fns';
import { cn } from '@/lib/utils';

interface User {
  id: string;
  email: string;
  permissions: string[];
  role: string;
  created_at: string;
}

interface UserPermissionWithExpiry {
  permission: string;
  basePermission: string;
  platform: string;
  resource: string;
  action: string;
  expiresAt?: number;
  isExpired: boolean;
  timeRemaining?: number;
  grantedAt?: number;
  reason?: string;
}

// Platform and permission presets
const PLATFORMS = [
  { value: 'epsx', label: 'EPSX Analytics' },
  { value: 'epsx-pay', label: 'EPSX Pay' },
  { value: 'epsx-token', label: 'EPSX Token' },
  { value: 'admin', label: 'Admin Platform' }
];

const PERMISSION_TEMPLATES = {
  'epsx': {
    'Analytics Viewer': 'epsx:analytics:view',
    'Premium Analytics': 'epsx:analytics:premium',
    'Rankings 25 Limit': 'epsx:rankings:view:25',
    'Rankings 100 Limit': 'epsx:rankings:view:100',
    'Unlimited Rankings': 'epsx:rankings:view:unlimited',
    'Real-time Data': 'epsx:realtime:access',
    'Export Basic': 'epsx:export:basic',
    'Export Advanced': 'epsx:export:advanced'
  },
  'epsx-pay': {
    'Transaction View': 'epsx-pay:transactions:read',
    'Payment Process': 'epsx-pay:payments:create',
    'Subscription Manage': 'epsx-pay:subscriptions:manage'
  },
  'epsx-token': {
    'Token View': 'epsx-token:tokens:view',
    'Token Trade': 'epsx-token:trading:execute',
    'Wallet Manage': 'epsx-token:wallet:manage'
  },
  'admin': {
    'User Management': 'admin:users:manage',
    'System Configure': 'admin:system:configure',
    'Analytics Admin': 'admin:analytics:manage',
    'Full Admin Access': 'admin:*:*'
  }
};

const DURATION_PRESETS = [
  { label: '1 Hour', hours: 1 },
  { label: '6 Hours', hours: 6 },
  { label: '1 Day', hours: 24 },
  { label: '3 Days', hours: 72 },
  { label: '1 Week', hours: 168 },
  { label: '1 Month', hours: 720 },
  { label: '3 Months', hours: 2160 },
  { label: '6 Months', hours: 4320 },
  { label: '1 Year', hours: 8760 }
];

function PermissionGrantDialog({ user, onPermissionGranted }: { 
  user: User; 
  onPermissionGranted: () => void; 
}) {
  const [isOpen, setIsOpen] = useState(false);
  const [isLoading, setIsLoading] = useState(false);
  const [platform, setPlatform] = useState('epsx');
  const [permissionTemplate, setPermissionTemplate] = useState('');
  const [customPermission, setCustomPermission] = useState('');
  const [useCustomPermission, setUseCustomPermission] = useState(false);
  const [duration, setDuration] = useState<number>(24); // Default 1 day
  const [customDate, setCustomDate] = useState('');
  const [useCustomDate, setUseCustomDate] = useState(false);
  const [reason, setReason] = useState('');

  const handleGrantPermission = async () => {
    if (!user.id) return;

    const basePermission = useCustomPermission ? customPermission : permissionTemplate;
    if (!basePermission.trim()) {
      toast({
        title: 'Error',
        description: 'Please select a permission template or enter a custom permission.',
        variant: 'destructive'
      });
      return;
    }

    // Calculate expiry timestamp
    let expiryTimestamp: number;
    if (useCustomDate && customDate) {
      expiryTimestamp = Math.floor(new Date(customDate).getTime() / 1000);
    } else {
      expiryTimestamp = Math.floor((Date.now() + (duration * 60 * 60 * 1000)) / 1000);
    }

    // Parse platform, resource, action from permission
    const parts = basePermission.split(':');
    const permissionData: EmbeddedPermissionData = {
      userId: user.id,
      basePermission: basePermission.trim(),
      platform: parts[0] || platform,
      resource: parts[1] || 'general',
      action: parts[2] || 'access',
      expiryTimestamp,
      reason: reason.trim() || undefined,
      metadata: {
        grantedBy: 'admin-interface',
        grantedAt: Date.now()
      }
    };

    setIsLoading(true);
    try {
      const result = await grantEmbeddedTimestampPermission(permissionData);
      
      if (result.success) {
        toast({
          title: 'Permission Granted',
          description: `Successfully granted ${basePermission} to ${user.email}`,
        });
        onPermissionGranted();
        setIsOpen(false);
        // Reset form
        setPermissionTemplate('');
        setCustomPermission('');
        setUseCustomPermission(false);
        setReason('');
        setDuration(24);
        setCustomDate('');
        setUseCustomDate(false);
      } else {
        toast({
          title: 'Grant Failed',
          description: result.message || 'Failed to grant permission',
          variant: 'destructive'
        });
      }
    } catch (error) {
      console.error('Error granting permission:', error);
      toast({
        title: 'Grant Failed',
        description: 'An unexpected error occurred',
        variant: 'destructive'
      });
    } finally {
      setIsLoading(false);
    }
  };

  return (
    <Dialog open={isOpen} onOpenChange={setIsOpen}>
      <DialogTrigger asChild>
        <Button size="sm" className="ml-2">
          <Plus className="h-4 w-4 mr-1" />
          Grant Permission
        </Button>
      </DialogTrigger>
      
      <DialogContent className="max-w-2xl max-h-[90vh] overflow-y-auto">
        <DialogHeader>
          <DialogTitle className="flex items-center space-x-2">
            <Shield className="h-5 w-5" />
            <span>Grant Embedded Timestamp Permission</span>
          </DialogTitle>
          <DialogDescription>
            Grant a time-limited permission to {user.email}
          </DialogDescription>
        </DialogHeader>
        
        <div className="space-y-6">
          {/* Platform Selection */}
          <div className="space-y-2">
            <Label htmlFor="platform">Platform</Label>
            <Select value={platform} onValueChange={setPlatform}>
              <SelectTrigger>
                <SelectValue placeholder="Select platform" />
              </SelectTrigger>
              <SelectContent>
                {PLATFORMS.map(p => (
                  <SelectItem key={p.value} value={p.value}>
                    {p.label}
                  </SelectItem>
                ))}
              </SelectContent>
            </Select>
          </div>

          {/* Permission Selection */}
          <div className="space-y-4">
            <div className="flex items-center space-x-4">
              <div className="flex items-center space-x-2">
                <input
                  type="radio"
                  id="use-template"
                  name="permission-type"
                  checked={!useCustomPermission}
                  onChange={(e) => setUseCustomPermission(!e.target.checked)}
                  className="w-4 h-4"
                />
                <Label htmlFor="use-template">Use Template</Label>
              </div>
              <div className="flex items-center space-x-2">
                <input
                  type="radio"
                  id="use-custom"
                  name="permission-type"
                  checked={useCustomPermission}
                  onChange={(e) => setUseCustomPermission(e.target.checked)}
                  className="w-4 h-4"
                />
                <Label htmlFor="use-custom">Custom Permission</Label>
              </div>
            </div>

            {!useCustomPermission ? (
              <div className="space-y-2">
                <Label>Permission Template</Label>
                <Select value={permissionTemplate} onValueChange={setPermissionTemplate}>
                  <SelectTrigger>
                    <SelectValue placeholder="Select permission template" />
                  </SelectTrigger>
                  <SelectContent>
                    {Object.entries(PERMISSION_TEMPLATES[platform as keyof typeof PERMISSION_TEMPLATES] || {}).map(([name, perm]) => (
                      <SelectItem key={perm} value={perm}>
                        {name} ({perm})
                      </SelectItem>
                    ))}
                  </SelectContent>
                </Select>
              </div>
            ) : (
              <div className="space-y-2">
                <Label htmlFor="custom-permission">Custom Permission</Label>
                <Input
                  id="custom-permission"
                  value={customPermission}
                  onChange={(e) => setCustomPermission(e.target.value)}
                  placeholder="e.g., epsx:analytics:premium"
                />
              </div>
            )}
          </div>

          {/* Expiry Time Selection */}
          <div className="space-y-4">
            <div className="flex items-center space-x-4">
              <div className="flex items-center space-x-2">
                <input
                  type="radio"
                  id="use-duration"
                  name="time-type"
                  checked={!useCustomDate}
                  onChange={(e) => setUseCustomDate(!e.target.checked)}
                  className="w-4 h-4"
                />
                <Label htmlFor="use-duration">Duration</Label>
              </div>
              <div className="flex items-center space-x-2">
                <input
                  type="radio"
                  id="use-date"
                  name="time-type"
                  checked={useCustomDate}
                  onChange={(e) => setUseCustomDate(e.target.checked)}
                  className="w-4 h-4"
                />
                <Label htmlFor="use-date">Specific Date</Label>
              </div>
            </div>

            {!useCustomDate ? (
              <div className="space-y-2">
                <Label>Duration</Label>
                <Select value={duration.toString()} onValueChange={(val) => setDuration(parseInt(val))}>
                  <SelectTrigger>
                    <SelectValue placeholder="Select duration" />
                  </SelectTrigger>
                  <SelectContent>
                    {DURATION_PRESETS.map(preset => (
                      <SelectItem key={preset.hours} value={preset.hours.toString()}>
                        {preset.label}
                      </SelectItem>
                    ))}
                  </SelectContent>
                </Select>
              </div>
            ) : (
              <div className="space-y-2">
                <Label htmlFor="expiry-date">Expiry Date & Time</Label>
                <Input
                  id="expiry-date"
                  type="datetime-local"
                  value={customDate}
                  onChange={(e) => setCustomDate(e.target.value)}
                />
              </div>
            )}
          </div>

          {/* Reason */}
          <div className="space-y-2">
            <Label htmlFor="reason">Reason (Optional)</Label>
            <Textarea
              id="reason"
              value={reason}
              onChange={(e) => setReason(e.target.value)}
              placeholder="Why are you granting this permission?"
              rows={3}
            />
          </div>

          {/* Preview */}
          <Alert>
            <AlertCircle className="h-4 w-4" />
            <AlertDescription>
              <div className="space-y-1">
                <div><strong>Permission:</strong> {useCustomPermission ? customPermission : permissionTemplate}</div>
                <div><strong>Expires:</strong> {
                  useCustomDate && customDate 
                    ? formatDistanceToNow(new Date(customDate), { addSuffix: true })
                    : `in ${duration} hour${duration !== 1 ? 's' : ''}`
                }</div>
                {reason && <div><strong>Reason:</strong> {reason}</div>}
              </div>
            </AlertDescription>
          </Alert>

          {/* Actions */}
          <div className="flex justify-end space-x-2">
            <Button variant="outline" onClick={() => setIsOpen(false)}>
              Cancel
            </Button>
            <Button onClick={handleGrantPermission} disabled={isLoading}>
              {isLoading ? 'Granting...' : 'Grant Permission'}
            </Button>
          </div>
        </div>
      </DialogContent>
    </Dialog>
  );
}

function UserPermissionsCard({ user, onPermissionChanged }: { 
  user: User; 
  onPermissionChanged: () => void; 
}) {
  const [permissions, setPermissions] = useState<UserPermissionWithExpiry[]>([]);
  const [isLoading, setIsLoading] = useState(false);

  const loadUserPermissions = useCallback(async () => {
    setIsLoading(true);
    try {
      const result = await getUserPermissionsWithExpiry(user.id);
      if (result.success && result.data) {
        setPermissions(result.data);
      }
    } catch (error) {
      console.error('Error loading user permissions:', error);
    } finally {
      setIsLoading(false);
    }
  }, [user.id]);

  useEffect(() => {
    loadUserPermissions();
  }, [loadUserPermissions]);

  const handleRevokePermission = async (permission: string) => {
    if (!confirm(`Are you sure you want to revoke "${permission}"?`)) return;

    try {
      const result = await revokeEmbeddedTimestampPermission({
        userId: user.id,
        permission
      });

      if (result.success) {
        toast({
          title: 'Permission Revoked',
          description: `Successfully revoked ${permission} from ${user.email}`,
        });
        loadUserPermissions();
        onPermissionChanged();
      } else {
        toast({
          title: 'Revocation Failed',
          description: result.message || 'Failed to revoke permission',
          variant: 'destructive'
        });
      }
    } catch (error) {
      console.error('Error revoking permission:', error);
      toast({
        title: 'Revocation Failed',
        description: 'An unexpected error occurred',
        variant: 'destructive'
      });
    }
  };

  return (
    <Card>
      <CardHeader className="flex flex-row items-center justify-between space-y-0 pb-2">
        <div>
          <CardTitle className="text-sm font-medium flex items-center space-x-2">
            <User className="h-4 w-4" />
            <span>{user.email}</span>
          </CardTitle>
          <CardDescription className="flex items-center space-x-2 mt-1">
            <Badge variant="outline">{user.role}</Badge>
            <span className="text-xs text-gray-500">
              {permissions.length} permission{permissions.length !== 1 ? 's' : ''}
            </span>
          </CardDescription>
        </div>
        
        <PermissionGrantDialog 
          user={user} 
          onPermissionGranted={() => {
            loadUserPermissions();
            onPermissionChanged();
          }}
        />
      </CardHeader>
      
      <CardContent>
        {isLoading ? (
          <div className="text-center py-4 text-gray-500">Loading permissions...</div>
        ) : permissions.length === 0 ? (
          <div className="text-center py-4 text-gray-500">No permissions assigned</div>
        ) : (
          <div className="space-y-2">
            {permissions.map((perm, index) => (
              <div 
                key={`${perm.permission}-${index}`}
                className={cn(
                  'flex items-center justify-between p-3 rounded-lg border',
                  perm.isExpired 
                    ? 'bg-red-50 border-red-200' 
                    : perm.timeRemaining && perm.timeRemaining < 24 * 60 * 60 * 1000
                    ? 'bg-yellow-50 border-yellow-200'
                    : 'bg-gray-50 border-gray-200'
                )}
              >
                <div className="flex-1 min-w-0">
                  <div className="flex items-center space-x-2 mb-1">
                    <Badge 
                      variant="outline" 
                      className="text-xs"
                    >
                      {perm.platform}
                    </Badge>
                    {perm.isExpired && (
                      <Badge variant="destructive" className="text-xs">
                        <XCircle className="h-3 w-3 mr-1" />
                        Expired
                      </Badge>
                    )}
                  </div>
                  
                  <div className="font-mono text-sm font-medium text-gray-900">
                    {perm.basePermission}
                  </div>
                  
                  {perm.expiresAt && (
                    <div className="text-xs text-gray-500 flex items-center mt-1">
                      <Clock className="h-3 w-3 mr-1" />
                      {perm.isExpired ? 'Expired' : 'Expires'} {formatDistanceToNow(new Date(perm.expiresAt * 1000), { addSuffix: true })}
                    </div>
                  )}
                  
                  {perm.reason && (
                    <div className="text-xs text-gray-600 mt-1">
                      Reason: {perm.reason}
                    </div>
                  )}
                </div>
                
                <div className="flex items-center space-x-2">
                  <Button
                    variant="outline"
                    size="sm"
                    onClick={() => handleRevokePermission(perm.permission)}
                    className="h-6 px-2"
                  >
                    <Trash2 className="h-3 w-3" />
                  </Button>
                </div>
              </div>
            ))}
          </div>
        )}
      </CardContent>
    </Card>
  );
}

export default function AdminPermissionsPage() {
  const { user } = useAuth();
  const [users, setUsers] = useState<User[]>([]);
  const [isLoading, setIsLoading] = useState(true);
  const [searchQuery, setSearchQuery] = useState('');
  const [selectedRole, setSelectedRole] = useState('all');
  const [refreshKey, setRefreshKey] = useState(0);

  const loadUsers = useCallback(async () => {
    setIsLoading(true);
    try {
      const result = await getAllUsers();
      if (result.success && result.data) {
        setUsers(result.data);
      }
    } catch (error) {
      console.error('Error loading users:', error);
      toast({
        title: 'Load Failed',
        description: 'Failed to load users',
        variant: 'destructive'
      });
    } finally {
      setIsLoading(false);
    }
  }, []);

  useEffect(() => {
    loadUsers();
  }, [loadUsers, refreshKey]);

  const handlePermissionChanged = useCallback(() => {
    setRefreshKey(prev => prev + 1);
  }, []);

  // Filter users based on search and role
  const filteredUsers = users.filter(u => {
    const matchesSearch = !searchQuery || 
      u.email.toLowerCase().includes(searchQuery.toLowerCase());
    const matchesRole = selectedRole === 'all' || u.role === selectedRole;
    return matchesSearch && matchesRole;
  });

  // Get unique roles
  const roles = ['all', ...Array.from(new Set(users.map(u => u.role)))];

  return (
    <AdminFeatureGate requiredPermissions={['admin:users:manage']}>
      <div className="container mx-auto p-6 max-w-7xl">
        <div className="flex items-center justify-between mb-6">
          <div>
            <h1 className="text-3xl font-bold text-gray-900">Permission Management</h1>
            <p className="text-gray-600 mt-1">
              Manage embedded timestamp permissions for all users
            </p>
          </div>
          
          <div className="flex items-center space-x-2">
            <Badge variant="secondary" className="flex items-center space-x-1">
              <Users className="h-3 w-3" />
              <span>{filteredUsers.length} users</span>
            </Badge>
          </div>
        </div>

        {/* Filters */}
        <Card className="mb-6">
          <CardContent className="p-4">
            <div className="flex items-center space-x-4">
              <div className="flex-1">
                <div className="relative">
                  <Search className="absolute left-3 top-1/2 transform -translate-y-1/2 h-4 w-4 text-gray-400" />
                  <Input
                    placeholder="Search users by email..."
                    value={searchQuery}
                    onChange={(e) => setSearchQuery(e.target.value)}
                    className="pl-9"
                  />
                </div>
              </div>
              
              <Select value={selectedRole} onValueChange={setSelectedRole}>
                <SelectTrigger className="w-48">
                  <SelectValue placeholder="Filter by role" />
                </SelectTrigger>
                <SelectContent>
                  {roles.map(role => (
                    <SelectItem key={role} value={role}>
                      {role === 'all' ? 'All Roles' : role}
                    </SelectItem>
                  ))}
                </SelectContent>
              </Select>
            </div>
          </CardContent>
        </Card>

        {/* Users List */}
        {isLoading ? (
          <div className="text-center py-12">
            <div className="inline-flex items-center justify-center w-16 h-16 rounded-full bg-gray-100 mb-4">
              <Users className="h-8 w-8 text-gray-400" />
            </div>
            <div className="text-lg font-medium text-gray-900">Loading users...</div>
          </div>
        ) : filteredUsers.length === 0 ? (
          <div className="text-center py-12">
            <div className="inline-flex items-center justify-center w-16 h-16 rounded-full bg-gray-100 mb-4">
              <Users className="h-8 w-8 text-gray-400" />
            </div>
            <div className="text-lg font-medium text-gray-900">No users found</div>
            <p className="text-gray-500">Try adjusting your search or filters.</p>
          </div>
        ) : (
          <div className="grid grid-cols-1 lg:grid-cols-2 gap-6">
            {filteredUsers.map(user => (
              <UserPermissionsCard
                key={user.id}
                user={user}
                onPermissionChanged={handlePermissionChanged}
              />
            ))}
          </div>
        )}
      </div>
    </AdminFeatureGate>
  );
}