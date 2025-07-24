'use client';

import { useState } from 'react';
import { Card, CardContent, CardHeader, CardTitle } from '@/components/ui/card';
import { Button } from '@/components/ui/button';
import { Input } from '@/components/ui/input';
import { Badge } from '@/components/ui/badge';
import { Checkbox } from '@/components/ui/checkbox';
import { Table, TableBody, TableCell, TableHead, TableHeader, TableRow } from '@/components/ui/table';
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from '@/components/ui/select';
import { Tabs, TabsContent, TabsList, TabsTrigger } from '@/components/ui/tabs';
import { 
  Search, 
  Filter, 
  Download, 
  Grid3X3, 
  List,
  Shield,
  Key,
  Users,
  AlertTriangle,
  CheckCircle,
  XCircle
} from 'lucide-react';

interface Permission {
  id: string;
  name: string;
  description: string;
  resource: string;
  action: string;
  risk: 'low' | 'medium' | 'high';
}

interface Role {
  id: string;
  name: string;
  permissions: string[];
  userCount: number;
  isSystem: boolean;
}

interface User {
  id: string;
  name: string;
  email: string;
  roles: string[];
  directPermissions: string[];
}

const MOCK_PERMISSIONS: Permission[] = [
  { id: '1', name: 'View Analytics', description: 'Access analytics dashboard', resource: 'analytics', action: 'read', risk: 'low' },
  { id: '2', name: 'Export Analytics', description: 'Export analytics data', resource: 'analytics', action: 'export', risk: 'medium' },
  { id: '3', name: 'Delete Analytics', description: 'Delete analytics data', resource: 'analytics', action: 'delete', risk: 'high' },
  { id: '4', name: 'View Users', description: 'View user profiles', resource: 'users', action: 'read', risk: 'low' },
  { id: '5', name: 'Create Users', description: 'Create new users', resource: 'users', action: 'create', risk: 'medium' },
  { id: '6', name: 'Delete Users', description: 'Delete user accounts', resource: 'users', action: 'delete', risk: 'high' },
  { id: '7', name: 'Manage Billing', description: 'Access billing settings', resource: 'billing', action: 'manage', risk: 'high' },
  { id: '8', name: 'System Config', description: 'Configure system settings', resource: 'system', action: 'configure', risk: 'high' },
  { id: '9', name: 'View Audit Logs', description: 'Access audit logs', resource: 'audit', action: 'read', risk: 'medium' },
  { id: '10', name: 'Pattern Analysis', description: 'Run pattern analysis', resource: 'patterns', action: 'analyze', risk: 'low' }
];

const MOCK_ROLES: Role[] = [
  { id: '1', name: 'Admin', permissions: ['1', '2', '3', '4', '5', '6', '7', '8', '9', '10'], userCount: 3, isSystem: true },
  { id: '2', name: 'Analyst', permissions: ['1', '2', '9', '10'], userCount: 12, isSystem: false },
  { id: '3', name: 'User Manager', permissions: ['4', '5'], userCount: 5, isSystem: false },
  { id: '4', name: 'Viewer', permissions: ['1', '4'], userCount: 25, isSystem: false }
];

const MOCK_USERS: User[] = [
  { id: '1', name: 'John Admin', email: 'john@example.com', roles: ['1'], directPermissions: [] },
  { id: '2', name: 'Jane Analyst', email: 'jane@example.com', roles: ['2'], directPermissions: ['3'] },
  { id: '3', name: 'Bob Viewer', email: 'bob@example.com', roles: ['4'], directPermissions: [] }
];

export function PermissionMatrix() {
  const [view, setView] = useState<'matrix' | 'list'>('matrix');
  const [searchTerm, setSearchTerm] = useState('');
  const [selectedResource, setSelectedResource] = useState<string>('all');
  const [selectedRisk, setSelectedRisk] = useState<string>('all');

  const resources = Array.from(new Set(MOCK_PERMISSIONS.map(p => p.resource)));
  const riskLevels = ['low', 'medium', 'high'];

  const filteredPermissions = MOCK_PERMISSIONS.filter(permission => {
    const matchesSearch = permission.name.toLowerCase().includes(searchTerm.toLowerCase()) ||
                         permission.description.toLowerCase().includes(searchTerm.toLowerCase());
    const matchesResource = selectedResource === 'all' || permission.resource === selectedResource;
    const matchesRisk = selectedRisk === 'all' || permission.risk === selectedRisk;
    
    return matchesSearch && matchesResource && matchesRisk;
  });

  const getRiskColor = (risk: string) => {
    switch (risk) {
      case 'low': return 'text-green-600 bg-green-50';
      case 'medium': return 'text-yellow-600 bg-yellow-50';
      case 'high': return 'text-red-600 bg-red-50';
      default: return 'text-gray-600 bg-gray-50';
    }
  };

  const getRiskIcon = (risk: string) => {
    switch (risk) {
      case 'low': return <CheckCircle className="h-4 w-4" />;
      case 'medium': return <AlertTriangle className="h-4 w-4" />;
      case 'high': return <XCircle className="h-4 w-4" />;
      default: return null;
    }
  };

  const hasPermission = (roleId: string, permissionId: string) => {
    const role = MOCK_ROLES.find(r => r.id === roleId);
    return role?.permissions.includes(permissionId) || false;
  };

  const getEffectivePermissions = (userId: string) => {
    const user = MOCK_USERS.find(u => u.id === userId);
    if (!user) return new Set<string>();

    const rolePermissions = user.roles.flatMap(roleId => {
      const role = MOCK_ROLES.find(r => r.id === roleId);
      return role?.permissions || [];
    });

    return new Set([...rolePermissions, ...user.directPermissions]);
  };

  return (
    <div className="space-y-6">
      {/* Header */}
      <div className="flex items-center justify-between">
        <div>
          <h2 className="text-2xl font-bold">Permission Matrix</h2>
          <p className="text-muted-foreground">
            View and manage permissions across roles and users
          </p>
        </div>

        <div className="flex items-center gap-3">
          <div className="flex items-center gap-1 bg-muted rounded-lg p-1">
            <Button
              variant={view === 'matrix' ? 'default' : 'ghost'}
              size="sm"
              onClick={() => setView('matrix')}
            >
              <Grid3X3 className="h-4 w-4" />
            </Button>
            <Button
              variant={view === 'list' ? 'default' : 'ghost'}
              size="sm"
              onClick={() => setView('list')}
            >
              <List className="h-4 w-4" />
            </Button>
          </div>

          <Button variant="outline" size="sm">
            <Download className="h-4 w-4 mr-2" />
            Export
          </Button>
        </div>
      </div>

      {/* Filters */}
      <Card>
        <CardContent className="p-4">
          <div className="flex items-center gap-4">
            <div className="flex-1 relative">
              <Search className="absolute left-3 top-1/2 transform -translate-y-1/2 h-4 w-4 text-muted-foreground" />
              <Input
                placeholder="Search permissions..."
                value={searchTerm}
                onChange={(e) => setSearchTerm(e.target.value)}
                className="pl-9"
              />
            </div>

            <Select value={selectedResource} onValueChange={setSelectedResource}>
              <SelectTrigger className="w-40">
                <SelectValue placeholder="Resource" />
              </SelectTrigger>
              <SelectContent>
                <SelectItem value="all">All Resources</SelectItem>
                {resources.map(resource => (
                  <SelectItem key={resource} value={resource}>
                    {resource.charAt(0).toUpperCase() + resource.slice(1)}
                  </SelectItem>
                ))}
              </SelectContent>
            </Select>

            <Select value={selectedRisk} onValueChange={setSelectedRisk}>
              <SelectTrigger className="w-32">
                <SelectValue placeholder="Risk" />
              </SelectTrigger>
              <SelectContent>
                <SelectItem value="all">All Risk</SelectItem>
                {riskLevels.map(risk => (
                  <SelectItem key={risk} value={risk}>
                    {risk.charAt(0).toUpperCase() + risk.slice(1)}
                  </SelectItem>
                ))}
              </SelectContent>
            </Select>
          </div>
        </CardContent>
      </Card>

      {/* Permission Matrix/List */}
      <Tabs defaultValue="roles" className="space-y-4">
        <TabsList>
          <TabsTrigger value="roles">
            <Shield className="h-4 w-4 mr-2" />
            Role Permissions
          </TabsTrigger>
          <TabsTrigger value="users">
            <Users className="h-4 w-4 mr-2" />
            User Permissions
          </TabsTrigger>
        </TabsList>

        <TabsContent value="roles">
          {view === 'matrix' ? (
            <RoleMatrixView 
              permissions={filteredPermissions}
              roles={MOCK_ROLES}
              hasPermission={hasPermission}
              getRiskColor={getRiskColor}
              getRiskIcon={getRiskIcon}
            />
          ) : (
            <RoleListView 
              permissions={filteredPermissions}
              roles={MOCK_ROLES}
              hasPermission={hasPermission}
              getRiskColor={getRiskColor}
              getRiskIcon={getRiskIcon}
            />
          )}
        </TabsContent>

        <TabsContent value="users">
          <UserPermissionView 
            permissions={filteredPermissions}
            users={MOCK_USERS}
            roles={MOCK_ROLES}
            getEffectivePermissions={getEffectivePermissions}
            getRiskColor={getRiskColor}
            getRiskIcon={getRiskIcon}
          />
        </TabsContent>
      </Tabs>
    </div>
  );
}

// Role Matrix View
interface RoleMatrixViewProps {
  permissions: Permission[];
  roles: Role[];
  hasPermission: (roleId: string, permissionId: string) => boolean;
  getRiskColor: (risk: string) => string;
  getRiskIcon: (risk: string) => JSX.Element | null;
}

function RoleMatrixView({ permissions, roles, hasPermission, getRiskColor, getRiskIcon }: RoleMatrixViewProps) {
  return (
    <Card>
      <CardHeader>
        <CardTitle>Role-Permission Matrix</CardTitle>
      </CardHeader>
      <CardContent>
        <div className="overflow-x-auto">
          <Table>
            <TableHeader>
              <TableRow>
                <TableHead className="w-80">Permission</TableHead>
                <TableHead className="w-20">Risk</TableHead>
                {roles.map(role => (
                  <TableHead key={role.id} className="text-center min-w-24">
                    <div className="space-y-1">
                      <div>{role.name}</div>
                      <div className="text-xs text-muted-foreground">
                        {role.userCount} users
                      </div>
                    </div>
                  </TableHead>
                ))}
              </TableRow>
            </TableHeader>
            <TableBody>
              {permissions.map(permission => (
                <TableRow key={permission.id}>
                  <TableCell>
                    <div>
                      <div className="font-medium">{permission.name}</div>
                      <div className="text-sm text-muted-foreground">
                        {permission.description}
                      </div>
                      <Badge variant="outline" className="text-xs mt-1">
                        {permission.resource}:{permission.action}
                      </Badge>
                    </div>
                  </TableCell>
                  
                  <TableCell>
                    <div className={`inline-flex items-center gap-1 px-2 py-1 rounded-full text-xs ${getRiskColor(permission.risk)}`}>
                      {getRiskIcon(permission.risk)}
                      {permission.risk}
                    </div>
                  </TableCell>
                  
                  {roles.map(role => (
                    <TableCell key={role.id} className="text-center">
                      <Checkbox
                        checked={hasPermission(role.id, permission.id)}
                        disabled={role.isSystem}
                      />
                    </TableCell>
                  ))}
                </TableRow>
              ))}
            </TableBody>
          </Table>
        </div>
      </CardContent>
    </Card>
  );
}

// Role List View
interface RoleListViewProps extends RoleMatrixViewProps {}

function RoleListView({ permissions, roles, hasPermission, getRiskColor, getRiskIcon }: RoleListViewProps) {
  return (
    <div className="grid grid-cols-1 md:grid-cols-2 gap-6">
      {roles.map(role => (
        <Card key={role.id}>
          <CardHeader>
            <div className="flex items-center justify-between">
              <CardTitle className="text-lg">{role.name}</CardTitle>
              <div className="flex items-center gap-2">
                <Badge variant={role.isSystem ? 'secondary' : 'outline'}>
                  {role.isSystem ? 'System' : 'Custom'}
                </Badge>
                <Badge variant="outline">
                  {role.userCount} users
                </Badge>
              </div>
            </div>
          </CardHeader>
          <CardContent>
            <div className="space-y-2">
              {permissions
                .filter(p => hasPermission(role.id, p.id))
                .map(permission => (
                  <div key={permission.id} className="flex items-center justify-between p-2 bg-muted rounded-lg">
                    <div className="flex-1">
                      <div className="font-medium text-sm">{permission.name}</div>
                      <div className="text-xs text-muted-foreground">
                        {permission.resource}:{permission.action}
                      </div>
                    </div>
                    <div className={`inline-flex items-center gap-1 px-2 py-1 rounded-full text-xs ${getRiskColor(permission.risk)}`}>
                      {getRiskIcon(permission.risk)}
                      {permission.risk}
                    </div>
                  </div>
                ))}
            </div>
          </CardContent>
        </Card>
      ))}
    </div>
  );
}

// User Permission View
interface UserPermissionViewProps {
  permissions: Permission[];
  users: User[];
  roles: Role[];
  getEffectivePermissions: (userId: string) => Set<string>;
  getRiskColor: (risk: string) => string;
  getRiskIcon: (risk: string) => JSX.Element | null;
}

function UserPermissionView({ 
  permissions, 
  users, 
  roles, 
  getEffectivePermissions, 
  getRiskColor, 
  getRiskIcon 
}: UserPermissionViewProps) {
  return (
    <div className="space-y-6">
      {users.map(user => {
        const effectivePermissions = getEffectivePermissions(user.id);
        const userRoles = roles.filter(role => user.roles.includes(role.id));
        
        return (
          <Card key={user.id}>
            <CardHeader>
              <div className="flex items-center justify-between">
                <div>
                  <CardTitle className="text-lg">{user.name}</CardTitle>
                  <p className="text-sm text-muted-foreground">{user.email}</p>
                </div>
                <div className="flex items-center gap-2">
                  {userRoles.map(role => (
                    <Badge key={role.id} variant="outline">
                      {role.name}
                    </Badge>
                  ))}
                  {user.directPermissions.length > 0 && (
                    <Badge variant="secondary">
                      +{user.directPermissions.length} direct
                    </Badge>
                  )}
                </div>
              </div>
            </CardHeader>
            <CardContent>
              <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-3">
                {permissions
                  .filter(p => effectivePermissions.has(p.id))
                  .map(permission => {
                    const isDirect = user.directPermissions.includes(permission.id);
                    
                    return (
                      <div key={permission.id} className="flex items-center justify-between p-3 bg-muted rounded-lg">
                        <div className="flex-1">
                          <div className="font-medium text-sm">{permission.name}</div>
                          <div className="text-xs text-muted-foreground">
                            {permission.resource}:{permission.action}
                          </div>
                          {isDirect && (
                            <Badge variant="secondary" className="text-xs mt-1">
                              Direct
                            </Badge>
                          )}
                        </div>
                        <div className={`inline-flex items-center gap-1 px-2 py-1 rounded-full text-xs ${getRiskColor(permission.risk)}`}>
                          {getRiskIcon(permission.risk)}
                          {permission.risk}
                        </div>
                      </div>
                    );
                  })}
              </div>
            </CardContent>
          </Card>
        );
      })}
    </div>
  );
}