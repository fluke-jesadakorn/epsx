'use client';

import { useState } from 'react';
import { Card, CardContent, CardHeader, CardTitle } from '@/components/ui/card';
import { Button } from '@/components/ui/button';
import { Input } from '@/components/ui/input';
import { Badge } from '@/components/ui/badge';
import { Textarea } from '@/components/ui/textarea';
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from '@/components/ui/select';
import { Checkbox } from '@/components/ui/checkbox';
import { Table, TableBody, TableCell, TableHead, TableHeader, TableRow } from '@/components/ui/table';
import { Dialog, DialogContent, DialogHeader, DialogTitle, DialogTrigger } from '@/components/ui/dialog';
import { 
  Plus, 
  Edit2, 
  Trash2, 
  Shield, 
  Users, 
  Key,
  Eye,
  Search,
  Filter,
  MoreVertical
} from 'lucide-react';

interface Permission {
  id: string;
  name: string;
  description: string;
  resource: string;
  action: string;
}

interface Role {
  id: string;
  name: string;
  description: string;
  permissions: Permission[];
  userCount: number;
  isSystem: boolean;
  createdAt: string;
  updatedAt: string;
}

const MOCK_PERMISSIONS: Permission[] = [
  { id: '1', name: 'View Analytics', description: 'Access to analytics dashboard', resource: 'analytics', action: 'read' },
  { id: '2', name: 'Export Data', description: 'Export analytics and reports', resource: 'analytics', action: 'export' },
  { id: '3', name: 'Manage Users', description: 'Create, edit, and delete users', resource: 'users', action: 'manage' },
  { id: '4', name: 'View Users', description: 'View user profiles and information', resource: 'users', action: 'read' },
  { id: '5', name: 'Manage Billing', description: 'Access billing and payment settings', resource: 'billing', action: 'manage' },
  { id: '6', name: 'System Settings', description: 'Configure system-wide settings', resource: 'system', action: 'configure' },
  { id: '7', name: 'Audit Logs', description: 'View system audit logs', resource: 'audit', action: 'read' },
  { id: '8', name: 'Pattern Analysis', description: 'Access pattern recognition features', resource: 'patterns', action: 'analyze' }
];

const MOCK_ROLES: Role[] = [
  {
    id: '1',
    name: 'Admin',
    description: 'Full system access with all permissions',
    permissions: MOCK_PERMISSIONS,
    userCount: 3,
    isSystem: true,
    createdAt: '2024-01-01T00:00:00Z',
    updatedAt: '2024-01-15T00:00:00Z'
  },
  {
    id: '2',
    name: 'Analyst',
    description: 'Access to analytics and pattern recognition',
    permissions: MOCK_PERMISSIONS.filter(p => ['analytics', 'patterns'].includes(p.resource)),
    userCount: 12,
    isSystem: false,
    createdAt: '2024-01-05T00:00:00Z',
    updatedAt: '2024-01-20T00:00:00Z'
  },
  {
    id: '3',
    name: 'Viewer',
    description: 'Read-only access to basic features',
    permissions: MOCK_PERMISSIONS.filter(p => p.action === 'read'),
    userCount: 25,
    isSystem: false,
    createdAt: '2024-01-10T00:00:00Z',
    updatedAt: '2024-01-25T00:00:00Z'
  }
];

export function RoleManager() {
  const [roles, setRoles] = useState<Role[]>(MOCK_ROLES);
  const [searchTerm, setSearchTerm] = useState('');
  const [selectedRole, setSelectedRole] = useState<Role | null>(null);
  const [isCreateDialogOpen, setIsCreateDialogOpen] = useState(false);
  const [editingRole, setEditingRole] = useState<Partial<Role>>({});

  const filteredRoles = roles.filter(role =>
    role.name.toLowerCase().includes(searchTerm.toLowerCase()) ||
    role.description.toLowerCase().includes(searchTerm.toLowerCase())
  );

  const handleCreateRole = () => {
    const newRole: Role = {
      id: Date.now().toString(),
      name: editingRole.name || '',
      description: editingRole.description || '',
      permissions: editingRole.permissions || [],
      userCount: 0,
      isSystem: false,
      createdAt: new Date().toISOString(),
      updatedAt: new Date().toISOString()
    };

    setRoles([...roles, newRole]);
    setIsCreateDialogOpen(false);
    setEditingRole({});
  };

  const handleUpdateRole = (roleId: string, updates: Partial<Role>) => {
    setRoles(roles.map(role =>
      role.id === roleId
        ? { ...role, ...updates, updatedAt: new Date().toISOString() }
        : role
    ));
  };

  const handleDeleteRole = (roleId: string) => {
    const role = roles.find(r => r.id === roleId);
    if (role?.isSystem) {
      alert('Cannot delete system roles');
      return;
    }
    
    if (role?.userCount > 0) {
      alert('Cannot delete roles that are assigned to users');
      return;
    }

    setRoles(roles.filter(role => role.id !== roleId));
  };

  const getRoleTypeLabel = (role: Role) => {
    if (role.isSystem) {
      return <Badge variant="secondary">System</Badge>;
    }
    return <Badge variant="outline">Custom</Badge>;
  };

  return (
    <div className="space-y-6">
      {/* Header */}
      <div className="flex items-center justify-between">
        <div>
          <h2 className="text-2xl font-bold">Role Management</h2>
          <p className="text-muted-foreground">
            Manage user roles and permissions across the system
          </p>
        </div>

        <Dialog open={isCreateDialogOpen} onOpenChange={setIsCreateDialogOpen}>
          <DialogTrigger asChild>
            <Button onClick={() => setEditingRole({})}>
              <Plus className="h-4 w-4 mr-2" />
              Create Role
            </Button>
          </DialogTrigger>
          <DialogContent className="max-w-2xl">
            <DialogHeader>
              <DialogTitle>Create New Role</DialogTitle>
            </DialogHeader>
            <RoleForm
              role={editingRole}
              permissions={MOCK_PERMISSIONS}
              onChange={setEditingRole}
              onSave={handleCreateRole}
              onCancel={() => setIsCreateDialogOpen(false)}
            />
          </DialogContent>
        </Dialog>
      </div>

      {/* Search and Filters */}
      <Card>
        <CardContent className="p-4">
          <div className="flex items-center gap-4">
            <div className="flex-1 relative">
              <Search className="absolute left-3 top-1/2 transform -translate-y-1/2 h-4 w-4 text-muted-foreground" />
              <Input
                placeholder="Search roles..."
                value={searchTerm}
                onChange={(e) => setSearchTerm(e.target.value)}
                className="pl-9"
              />
            </div>
            <Button variant="outline" size="sm">
              <Filter className="h-4 w-4 mr-2" />
              Filter
            </Button>
          </div>
        </CardContent>
      </Card>

      {/* Roles Table */}
      <Card>
        <CardHeader>
          <CardTitle className="flex items-center gap-2">
            <Shield className="h-5 w-5" />
            Roles ({filteredRoles.length})
          </CardTitle>
        </CardHeader>
        <CardContent>
          <Table>
            <TableHeader>
              <TableRow>
                <TableHead>Role</TableHead>
                <TableHead>Type</TableHead>
                <TableHead>Permissions</TableHead>
                <TableHead>Users</TableHead>
                <TableHead>Last Updated</TableHead>
                <TableHead>Actions</TableHead>
              </TableRow>
            </TableHeader>
            <TableBody>
              {filteredRoles.map((role) => (
                <TableRow key={role.id}>
                  <TableCell>
                    <div>
                      <div className="font-medium">{role.name}</div>
                      <div className="text-sm text-muted-foreground">
                        {role.description}
                      </div>
                    </div>
                  </TableCell>
                  
                  <TableCell>
                    {getRoleTypeLabel(role)}
                  </TableCell>
                  
                  <TableCell>
                    <div className="flex items-center gap-1">
                      <Key className="h-4 w-4 text-muted-foreground" />
                      <span className="text-sm">{role.permissions.length}</span>
                    </div>
                  </TableCell>
                  
                  <TableCell>
                    <div className="flex items-center gap-1">
                      <Users className="h-4 w-4 text-muted-foreground" />
                      <span className="text-sm">{role.userCount}</span>
                    </div>
                  </TableCell>
                  
                  <TableCell>
                    <span className="text-sm text-muted-foreground">
                      {new Date(role.updatedAt).toLocaleDateString()}
                    </span>
                  </TableCell>
                  
                  <TableCell>
                    <div className="flex items-center gap-2">
                      <Button
                        variant="ghost"
                        size="sm"
                        onClick={() => setSelectedRole(role)}
                      >
                        <Eye className="h-4 w-4" />
                      </Button>
                      
                      {!role.isSystem && (
                        <>
                          <Button
                            variant="ghost"
                            size="sm"
                          >
                            <Edit2 className="h-4 w-4" />
                          </Button>
                          
                          <Button
                            variant="ghost"
                            size="sm"
                            onClick={() => handleDeleteRole(role.id)}
                            disabled={role.userCount > 0}
                          >
                            <Trash2 className="h-4 w-4" />
                          </Button>
                        </>
                      )}
                    </div>
                  </TableCell>
                </TableRow>
              ))}
            </TableBody>
          </Table>
        </CardContent>
      </Card>

      {/* Role Details Dialog */}
      {selectedRole && (
        <Dialog open={!!selectedRole} onOpenChange={() => setSelectedRole(null)}>
          <DialogContent className="max-w-2xl">
            <DialogHeader>
              <DialogTitle>{selectedRole.name} Role Details</DialogTitle>
            </DialogHeader>
            <RoleDetails role={selectedRole} />
          </DialogContent>
        </Dialog>
      )}
    </div>
  );
}

// Role Form Component
interface RoleFormProps {
  role: Partial<Role>;
  permissions: Permission[];
  onChange: (role: Partial<Role>) => void;
  onSave: () => void;
  onCancel: () => void;
}

function RoleForm({ role, permissions, onChange, onSave, onCancel }: RoleFormProps) {
  const selectedPermissionIds = role.permissions?.map(p => p.id) || [];

  const handlePermissionToggle = (permission: Permission, checked: boolean) => {
    const currentPermissions = role.permissions || [];
    
    if (checked) {
      onChange({
        ...role,
        permissions: [...currentPermissions, permission]
      });
    } else {
      onChange({
        ...role,
        permissions: currentPermissions.filter(p => p.id !== permission.id)
      });
    }
  };

  const groupedPermissions = permissions.reduce((acc, permission) => {
    const resource = permission.resource;
    if (!acc[resource]) {
      acc[resource] = [];
    }
    acc[resource].push(permission);
    return acc;
  }, {} as Record<string, Permission[]>);

  return (
    <div className="space-y-6">
      {/* Basic Information */}
      <div className="space-y-4">
        <div>
          <label className="text-sm font-medium">Role Name</label>
          <Input
            value={role.name || ''}
            onChange={(e) => onChange({ ...role, name: e.target.value })}
            placeholder="Enter role name"
          />
        </div>
        
        <div>
          <label className="text-sm font-medium">Description</label>
          <Textarea
            value={role.description || ''}
            onChange={(e) => onChange({ ...role, description: e.target.value })}
            placeholder="Enter role description"
            rows={3}
          />
        </div>
      </div>

      {/* Permissions */}
      <div>
        <h4 className="text-sm font-medium mb-3">Permissions</h4>
        <div className="space-y-4 max-h-64 overflow-y-auto">
          {Object.entries(groupedPermissions).map(([resource, resourcePermissions]) => (
            <div key={resource} className="space-y-2">
              <h5 className="text-sm font-medium text-muted-foreground capitalize">
                {resource}
              </h5>
              {resourcePermissions.map((permission) => (
                <div key={permission.id} className="flex items-start space-x-3 pl-4">
                  <Checkbox
                    id={permission.id}
                    checked={selectedPermissionIds.includes(permission.id)}
                    onCheckedChange={(checked) => 
                      handlePermissionToggle(permission, checked as boolean)
                    }
                  />
                  <div className="grid gap-1.5 leading-none">
                    <label
                      htmlFor={permission.id}
                      className="text-sm font-medium leading-none peer-disabled:cursor-not-allowed peer-disabled:opacity-70"
                    >
                      {permission.name}
                    </label>
                    <p className="text-xs text-muted-foreground">
                      {permission.description}
                    </p>
                  </div>
                </div>
              ))}
            </div>
          ))}
        </div>
      </div>

      {/* Actions */}
      <div className="flex justify-end gap-3">
        <Button variant="outline" onClick={onCancel}>
          Cancel
        </Button>
        <Button onClick={onSave} disabled={!role.name || !role.permissions?.length}>
          Create Role
        </Button>
      </div>
    </div>
  );
}

// Role Details Component
interface RoleDetailsProps {
  role: Role;
}

function RoleDetails({ role }: RoleDetailsProps) {
  const groupedPermissions = role.permissions.reduce((acc, permission) => {
    const resource = permission.resource;
    if (!acc[resource]) {
      acc[resource] = [];
    }
    acc[resource].push(permission);
    return acc;
  }, {} as Record<string, Permission[]>);

  return (
    <div className="space-y-6">
      {/* Basic Info */}
      <div className="grid grid-cols-2 gap-4">
        <div>
          <label className="text-sm font-medium text-muted-foreground">Role Name</label>
          <p className="text-sm mt-1">{role.name}</p>
        </div>
        <div>
          <label className="text-sm font-medium text-muted-foreground">Type</label>
          <div className="mt-1">
            {role.isSystem ? (
              <Badge variant="secondary">System</Badge>
            ) : (
              <Badge variant="outline">Custom</Badge>
            )}
          </div>
        </div>
        <div>
          <label className="text-sm font-medium text-muted-foreground">Users Assigned</label>
          <p className="text-sm mt-1">{role.userCount}</p>
        </div>
        <div>
          <label className="text-sm font-medium text-muted-foreground">Last Updated</label>
          <p className="text-sm mt-1">{new Date(role.updatedAt).toLocaleDateString()}</p>
        </div>
      </div>

      <div>
        <label className="text-sm font-medium text-muted-foreground">Description</label>
        <p className="text-sm mt-1">{role.description}</p>
      </div>

      {/* Permissions */}
      <div>
        <h4 className="text-sm font-medium mb-3">Permissions ({role.permissions.length})</h4>
        <div className="space-y-4">
          {Object.entries(groupedPermissions).map(([resource, resourcePermissions]) => (
            <div key={resource} className="space-y-2">
              <h5 className="text-sm font-medium text-muted-foreground capitalize">
                {resource}
              </h5>
              <div className="pl-4 space-y-1">
                {resourcePermissions.map((permission) => (
                  <div key={permission.id} className="text-sm">
                    <span className="font-medium">{permission.name}</span>
                    <span className="text-muted-foreground"> - {permission.description}</span>
                  </div>
                ))}
              </div>
            </div>
          ))}
        </div>
      </div>
    </div>
  );
}