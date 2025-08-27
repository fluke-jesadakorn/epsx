'use client';

import { useState } from 'react';
import { Card, CardContent, CardHeader, CardTitle } from '@/components/ui/card';
import { Badge } from '@/components/ui/badge';
import { Table, TableBody, TableCell, TableHead, TableHeader, TableRow } from '@/components/ui/table';
import { 
  Shield, 
  Users, 
  Eye,
  CheckCircle,
  XCircle
} from 'lucide-react';
import { 
  Role, 
  AVAILABLE_FEATURES, 
  SIMPLE_ROLES, 
  ROLE_FEATURE_MATRIX,
  checkFeatureAccess
} from '@/config/iam/default-roles';
import { Tabs, TabsContent, TabsList, TabsTrigger } from '@/components/ui/tabs';

export function PermissionMatrix() {
  const [selectedRole, setSelectedRole] = useState<string>('all');

  const roleIcons = {
    admin: Shield,
    user: Users,
    guest: Eye
  };

  const getRoleIcon = (roleId: string) => {
    const IconComponent = roleIcons[roleId as keyof typeof roleIcons] || Users;
    return <IconComponent className="h-4 w-4" />;
  };

  return (
    <div className="space-y-6">
      {/* Header */}
      <div className="flex items-center justify-between">
        <div>
          <h2 className="text-2xl font-bold">Simple Role Matrix</h2>
          <p className="text-muted-foreground">
            Simple role-based access control with 3 roles and 7 features
          </p>
        </div>
      </div>

      <Tabs defaultValue="matrix" className="space-y-4">
        <TabsList>
          <TabsTrigger value="matrix">
            <Shield className="h-4 w-4 mr-2" />
            Role-Feature Matrix
          </TabsTrigger>
          <TabsTrigger value="roles">
            <Users className="h-4 w-4 mr-2" />
            Role Details
          </TabsTrigger>
        </TabsList>

        <TabsContent value="matrix">
          <Card>
            <CardHeader>
              <CardTitle>Role-Feature Matrix</CardTitle>
            </CardHeader>
            <CardContent>
              <div className="overflow-x-auto">
                <Table>
                  <TableHeader>
                    <TableRow>
                      <TableHead className="w-60">Feature</TableHead>
                      {SIMPLE_ROLES.map(role => (
                        <TableHead key={role.id} className="text-center min-w-24">
                          <div className="flex items-center justify-center gap-2">
                            {getRoleIcon(role.id)}
                            <div className="space-y-1">
                              <div className="font-semibold">{role.name}</div>
                              <div 
                                className="w-3 h-3 rounded-full mx-auto"
                                style={{ backgroundColor: role.color }}
                              />
                            </div>
                          </div>
                        </TableHead>
                      ))}
                    </TableRow>
                  </TableHeader>
                  <TableBody>
                    {AVAILABLE_FEATURES.map(feature => (
                      <TableRow key={feature}>
                        <TableCell>
                          <div>
                            <div className="font-medium">{feature.replace('_', ' ').toUpperCase()}</div>
                            <div className="text-sm text-muted-foreground">
                              {getFeatureDescription(feature)}
                            </div>
                          </div>
                        </TableCell>
                        {SIMPLE_ROLES.map(role => (
                          <TableCell key={role.id} className="text-center">
                            {checkFeatureAccess(Role[role.id.toUpperCase() as keyof typeof Role], feature) ? (
                              <CheckCircle className="h-5 w-5 text-green-600 mx-auto" />
                            ) : (
                              <XCircle className="h-5 w-5 text-red-400 mx-auto" />
                            )}
                          </TableCell>
                        ))}
                      </TableRow>
                    ))}
                  </TableBody>
                </Table>
              </div>
            </CardContent>
          </Card>
        </TabsContent>

        <TabsContent value="roles">
          <div className="grid grid-cols-1 md:grid-cols-3 gap-6">
            {SIMPLE_ROLES.map(role => (
              <Card key={role.id}>
                <CardHeader>
                  <div className="flex items-center gap-3">
                    {getRoleIcon(role.id)}
                    <div>
                      <CardTitle className="text-lg">{role.name}</CardTitle>
                      <Badge 
                        variant="outline" 
                        style={{ 
                          borderColor: role.color,
                          color: role.color 
                        }}
                      >
                        {role.id.toUpperCase()}
                      </Badge>
                    </div>
                  </div>
                </CardHeader>
                <CardContent>
                  <div className="space-y-3">
                    <p className="text-sm text-muted-foreground">
                      {role.description}
                    </p>
                    
                    <div>
                      <h4 className="font-semibold text-sm mb-2">Features ({role.features.length})</h4>
                      <div className="space-y-1">
                        {role.features.map(feature => (
                          <div key={feature} className="flex items-center gap-2 text-sm">
                            <CheckCircle className="h-3 w-3 text-green-600" />
                            <span>{feature.replace('_', ' ')}</span>
                          </div>
                        ))}
                      </div>
                    </div>

                    {role.features.length < AVAILABLE_FEATURES.length && (
                      <div>
                        <h4 className="font-semibold text-sm mb-2">Restricted</h4>
                        <div className="space-y-1">
                          {AVAILABLE_FEATURES
                            .filter(feature => !role.features.includes(feature))
                            .map(feature => (
                              <div key={feature} className="flex items-center gap-2 text-sm text-muted-foreground">
                                <XCircle className="h-3 w-3 text-red-400" />
                                <span>{feature.replace('_', ' ')}</span>
                              </div>
                            ))}
                        </div>
                      </div>
                    )}
                  </div>
                </CardContent>
              </Card>
            ))}
          </div>
        </TabsContent>
      </Tabs>

      {/* Role Hierarchy */}
      <Card>
        <CardHeader>
          <CardTitle>Role Hierarchy</CardTitle>
        </CardHeader>
        <CardContent>
          <div className="flex items-center justify-center gap-8 py-4">
            <div className="text-center">
              <div className="flex items-center justify-center w-16 h-16 rounded-full bg-red-100 mb-2">
                <Shield className="h-8 w-8 text-red-600" />
              </div>
              <div className="font-semibold">Admin</div>
              <div className="text-xs text-muted-foreground">Full Access</div>
            </div>
            
            <div className="text-2xl text-muted-foreground">{'>'}</div>
            
            <div className="text-center">
              <div className="flex items-center justify-center w-16 h-16 rounded-full bg-green-100 mb-2">
                <Users className="h-8 w-8 text-green-600" />
              </div>
              <div className="font-semibold">User</div>
              <div className="text-xs text-muted-foreground">Premium Features</div>
            </div>
            
            <div className="text-2xl text-muted-foreground">{'>'}</div>
            
            <div className="text-center">
              <div className="flex items-center justify-center w-16 h-16 rounded-full bg-gray-100 mb-2">
                <Eye className="h-8 w-8 text-gray-600" />
              </div>
              <div className="font-semibold">Guest</div>
              <div className="text-xs text-muted-foreground">View Only</div>
            </div>
          </div>
        </CardContent>
      </Card>
    </div>
  );
}

// Helper function to get feature descriptions
function getFeatureDescription(feature: string): string {
  const descriptions = {
    view_eps: 'View EPS growth analytics and stock rankings',
    export_data: 'Export analytics data to various formats',
    realtime: 'Access real-time market data and live updates',
    profile: 'Manage user profile and account settings',
    notifications: 'Receive and manage system notifications',
    billing: 'Access billing information and payment management',
    advanced_filters: 'Use advanced filtering and search capabilities'
  };
  
  return descriptions[feature as keyof typeof descriptions] || 'Feature access';
}