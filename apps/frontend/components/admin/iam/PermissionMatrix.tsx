'use client';

import { useState, useEffect } from 'react';
import { Checkbox } from '@/components/ui/checkbox';
import { Table, TableBody, TableCell, TableHead, TableHeader, TableRow } from '@/components/ui/table';
import { 
  Search, 
  // Filter - removed unused import 
  Download, 
  Grid3X3, 
  List,
  Shield,
  // Key - removed unused import
  Users,
  AlertTriangle,
  CheckCircle,
  XCircle
} from 'lucide-react';
import { permissionService, Permission, Role, User } from '@/services/permissionService';
import { Card, CardContent, CardHeader, CardTitle, Button, Input, Badge, Select, SelectContent, SelectItem, SelectTrigger, SelectValue, Tabs, TabsContent, TabsList, TabsTrigger } from '@/components/ui';
export function PermissionMatrix() {
  const [view, setView] = useState<'matrix' | 'list'>('matrix');
  const [searchTerm, setSearchTerm] = useState('');
  const [selectedResource, setSelectedResource] = useState<string>('all');
  const [selectedRisk, setSelectedRisk] = useState<string>('all');
  const [permissions, setPermissions] = useState<Permission[]>([]);
  const [roles, setRoles] = useState<Role[]>([]);
  const [users, setUsers] = useState<User[]>([]);
  const [loading, setLoading] = useState(true);
  useEffect(() => {
    const loadData = async () => {
      try {
        setLoading(true);
        const [permissionsData, rolesData, usersData] = await Promise.all([
          permissionService.getPermissions(),
          permissionService.getRoles(),
          permissionService.getUsers()
        ]);
        setPermissions(permissionsData);
        setRoles(rolesData);
        setUsers(usersData);
      } catch (error) {
        console.error('Failed to load permission data:', error);
      } finally {
        setLoading(false);
      }
    };
    loadData();
  }, []);
  const resources = Array.from(new Set(permissions.map(p => p.resource)));
  const riskLevels = ['low', 'medium', 'high'];
  const filteredPermissions = permissions.filter(permission => {
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
    const role = roles.find(r => r.id === roleId);
    return role?.permissions.includes(permissionId) || false;
  };
  const getEffectivePermissions = (userId: string) => {
    const user = users.find(u => u.id === userId);
    if (!user) return new Set<string>();
    const rolePermissions = user.roles.flatMap(roleId => {
      const role = roles.find(r => r.id === roleId);
      return role?.permissions || [];
    });
    return new Set([...rolePermissions, ...user.directPermissions]);
  };
  if (loading) {
    return (
      <div className="flex items-center justify-center h-64">
        <div className="animate-spin rounded-full h-8 w-8 border-b-2 border-orange-500"></div>
      </div>
    );
  }
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
              roles={roles}
              hasPermission={hasPermission}
              getRiskColor={getRiskColor}
              getRiskIcon={getRiskIcon}
            />
          ) : (
            <RoleListView 
              permissions={filteredPermissions}
              roles={roles}
              hasPermission={hasPermission}
              getRiskColor={getRiskColor}
              getRiskIcon={getRiskIcon}
            />
          )}
        </TabsContent>
        <TabsContent value="users">
          <UserPermissionView 
            permissions={filteredPermissions}
            users={users}
            roles={roles}
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