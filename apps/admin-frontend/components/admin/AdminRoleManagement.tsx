'use client';

import { useState, useEffect } from 'react';
import { 
  Shield, Users, Plus, Minus, Eye, Search, Clock, 
  AlertTriangle, CheckCircle, User, UserCheck,
  Settings, BarChart, CreditCard, Server, Code,
  Puzzle, ClipboardCheck, HeadphonesIcon, Key
} from 'lucide-react';
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '@epsx/ui';
import { Button } from '@/components/ui/button';
import { Input } from '@epsx/ui';
import { Badge } from '@/components/ui/badge';
import { Tabs, TabsContent, TabsList, TabsTrigger } from '@epsx/ui';
import { Label } from '@epsx/ui';
import { Separator } from '@/components/ui/separator';
import { useToast } from '@/components/ui/use-toast';

// Admin module definitions matching backend
interface AdminModule {
  code: string;
  name: string;
  description: string;
  category: string;
  icon: string;
  color: string;
  sort_order: number;
  is_active: boolean;
}

interface UserModuleAssignment {
  firebase_uid: string;
  modules: string[];
  module_details: AdminModule[];
  is_admin: boolean;
  total_modules: number;
}

interface ModuleAssignmentRequest {
  firebase_uid: string;
  module_codes: string[];
  granted_by: string;
  granted_reason: string;
}

// Icon mapping for admin modules
const getModuleIcon = (iconName: string) => {
  const iconMap: Record<string, any> = {
    'users': User,
    'shield-check': Shield,
    'key': Key,
    'chart-bar': BarChart,
    'credit-card': CreditCard,
    'server': Server,
    'code': Code,
    'puzzle': Puzzle,
    'clipboard-check': ClipboardCheck,
    'support': HeadphonesIcon,
  };
  return iconMap[iconName] || Shield;
};

// Color mapping for modules
const getModuleColorClass = (color: string) => {
  const colorMap: Record<string, string> = {
    'blue': 'bg-blue-50 border-blue-200 text-blue-800',
    'green': 'bg-green-50 border-green-200 text-green-800',
    'purple': 'bg-purple-50 border-purple-200 text-purple-800',
    'yellow': 'bg-yellow-50 border-yellow-200 text-yellow-800',
    'emerald': 'bg-emerald-50 border-emerald-200 text-emerald-800',
    'red': 'bg-red-50 border-red-200 text-red-800',
    'indigo': 'bg-indigo-50 border-indigo-200 text-indigo-800',
    'pink': 'bg-pink-50 border-pink-200 text-pink-800',
    'orange': 'bg-orange-50 border-orange-200 text-orange-800',
    'cyan': 'bg-cyan-50 border-cyan-200 text-cyan-800',
  };
  return colorMap[color] || 'bg-gray-50 border-gray-200 text-gray-800';
};

export function AdminRoleManagement() {
  const { toast } = useToast();
  const [modules, setModules] = useState<AdminModule[]>([]);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);
  
  // User management state
  const [searchEmail, setSearchEmail] = useState('');
  const [selectedUser, setSelectedUser] = useState<string>('');
  const [userAssignments, setUserAssignments] = useState<UserModuleAssignment | null>(null);
  const [loadingUser, setLoadingUser] = useState(false);
  
  // Assignment management state
  const [selectedModules, setSelectedModules] = useState<string[]>([]);
  const [assignmentReason, setAssignmentReason] = useState('');
  const [isAssigning, setIsAssigning] = useState(false);

  useEffect(() => {
    loadAdminModules();
  }, []);

  const loadAdminModules = async () => {
    try {
      setLoading(true);
      setError(null);

      const response = await fetch('/api/proxy/admin/admin-modules', {
        headers: { 'Content-Type': 'application/json' }
      });

      if (!response.ok) {
        throw new Error(`Failed to load admin modules: ${response.statusText}`);
      }

      const data = await response.json();
      setModules(data || []);
    } catch (err: any) {
      console.error('Failed to load admin modules:', err);
      setError(err.message || 'Failed to load admin modules');
    } finally {
      setLoading(false);
    }
  };

  const loadUserAssignments = async (firebaseUid: string) => {
    if (!firebaseUid.trim()) return;

    try {
      setLoadingUser(true);
      
      const response = await fetch(`/api/proxy/admin/admin-modules/users/${encodeURIComponent(firebaseUid)}`, {
        headers: { 'Content-Type': 'application/json' }
      });

      if (!response.ok) {
        if (response.status === 404) {
          setUserAssignments({
            firebase_uid: firebaseUid,
            modules: [],
            module_details: [],
            is_admin: false,
            total_modules: 0
          });
          return;
        }
        throw new Error(`Failed to load user assignments: ${response.statusText}`);
      }

      const data = await response.json();
      setUserAssignments(data);
    } catch (err: any) {
      console.error('Failed to load user assignments:', err);
      toast({
        title: "Error",
        description: err.message || 'Failed to load user assignments',
        variant: "destructive",
      });
    } finally {
      setLoadingUser(false);
    }
  };

  const handleUserSearch = async () => {
    if (!searchEmail.trim()) {
      toast({
        title: "Validation Error",
        description: "Please enter a user email address",
        variant: "destructive",
      });
      return;
    }

    setSelectedUser(searchEmail.trim());
    await loadUserAssignments(searchEmail.trim());
  };

  const handleAssignModules = async () => {
    if (!selectedUser) {
      toast({
        title: "Validation Error", 
        description: "Please select a user first",
        variant: "destructive",
      });
      return;
    }

    if (selectedModules.length === 0) {
      toast({
        title: "Validation Error",
        description: "Please select at least one module to assign",
        variant: "destructive",
      });
      return;
    }

    try {
      setIsAssigning(true);

      const request: ModuleAssignmentRequest = {
        firebase_uid: selectedUser,
        module_codes: selectedModules,
        granted_by: 'admin@dev.local', // TODO: Get from current user context
        granted_reason: assignmentReason || 'Module assignment via admin panel'
      };

      const response = await fetch('/api/proxy/admin/admin-modules/assign', {
        method: 'POST',
        headers: { 
          'Content-Type': 'application/json'
        },
        body: JSON.stringify(request)
      });

      if (!response.ok) {
        throw new Error(`Assignment failed: ${response.statusText}`);
      }

      const result = await response.json();
      
      toast({
        title: "Success",
        description: `Successfully assigned ${selectedModules.length} module(s) to ${selectedUser}`,
      });

      // Refresh user assignments
      await loadUserAssignments(selectedUser);
      
      // Clear selection
      setSelectedModules([]);
      setAssignmentReason('');
      
    } catch (err: any) {
      console.error('Failed to assign modules:', err);
      toast({
        title: "Assignment Error",
        description: err.message || 'Failed to assign modules',
        variant: "destructive",
      });
    } finally {
      setIsAssigning(false);
    }
  };

  const toggleModuleSelection = (moduleCode: string) => {
    setSelectedModules(prev => 
      prev.includes(moduleCode)
        ? prev.filter(code => code !== moduleCode)
        : [...prev, moduleCode]
    );
  };

  if (loading) {
    return (
      <div className="flex items-center justify-center p-8">
        <div className="text-center">
          <Shield className="mx-auto h-8 w-8 animate-spin text-blue-600" />
          <p className="mt-2 text-sm text-gray-600">Loading admin modules...</p>
        </div>
      </div>
    );
  }

  if (error) {
    return (
      <Card>
        <CardHeader>
          <CardTitle className="flex items-center text-red-600">
            <AlertTriangle className="h-5 w-5 mr-2" />
            Error Loading Admin Modules
          </CardTitle>
        </CardHeader>
        <CardContent>
          <p className="text-red-600">{error}</p>
          <Button onClick={loadAdminModules} className="mt-4">
            Retry
          </Button>
        </CardContent>
      </Card>
    );
  }

  return (
    <div className="space-y-6">
      {/* Header */}
      <div className="flex items-center justify-between">
        <div>
          <h1 className="text-3xl font-bold">Admin Role Management</h1>
          <p className="text-gray-600 mt-1">Manage granular admin module assignments</p>
        </div>
        <div className="flex items-center space-x-2">
          <Badge variant="outline" className="text-green-600 border-green-200">
            {modules.length} Active Modules
          </Badge>
        </div>
      </div>

      <Tabs defaultValue="assign" className="space-y-6">
        <TabsList>
          <TabsTrigger value="assign" className="flex items-center">
            <UserCheck className="h-4 w-4 mr-2" />
            Assign Modules
          </TabsTrigger>
          <TabsTrigger value="overview" className="flex items-center">
            <Eye className="h-4 w-4 mr-2" />
            Module Overview
          </TabsTrigger>
        </TabsList>

        <TabsContent value="assign" className="space-y-6">
          <div className="grid grid-cols-1 lg:grid-cols-2 gap-6">
            {/* User Selection Panel */}
            <Card>
              <CardHeader>
                <CardTitle className="flex items-center">
                  <Search className="h-5 w-5 mr-2" />
                  User Selection
                </CardTitle>
                <CardDescription>
                  Search for a user to manage their admin module assignments
                </CardDescription>
              </CardHeader>
              <CardContent className="space-y-4">
                <div className="flex space-x-2">
                  <Input
                    type="email"
                    placeholder="Enter user email (e.g. jesadakorn.kirtnu@gmail.com)"
                    value={searchEmail}
                    onChange={(e) => setSearchEmail(e.target.value)}
                    onKeyPress={(e) => e.key === 'Enter' && handleUserSearch()}
                  />
                  <Button 
                    onClick={handleUserSearch}
                    disabled={loadingUser || !searchEmail.trim()}
                  >
                    {loadingUser ? 'Loading...' : 'Search'}
                  </Button>
                </div>

                {selectedUser && (
                  <div className="border rounded-lg p-4 bg-blue-50">
                    <div className="flex items-center justify-between">
                      <div>
                        <h3 className="font-medium text-blue-900">Selected User</h3>
                        <p className="text-sm text-blue-700">{selectedUser}</p>
                      </div>
                      {userAssignments && (
                        <Badge variant={userAssignments.is_admin ? "default" : "secondary"}>
                          {userAssignments.total_modules} Modules
                        </Badge>
                      )}
                    </div>
                  </div>
                )}

                {/* Current User Assignments */}
                {userAssignments && (
                  <div className="space-y-3">
                    <Label>Current Assignments ({userAssignments.total_modules})</Label>
                    <div className="grid grid-cols-1 gap-2 max-h-40 overflow-y-auto">
                      {userAssignments.module_details.length > 0 ? (
                        userAssignments.module_details.map((module) => {
                          const IconComponent = getModuleIcon(module.icon);
                          return (
                            <div 
                              key={module.code}
                              className={`flex items-center p-2 rounded border ${getModuleColorClass(module.color)}`}
                            >
                              <IconComponent className="h-4 w-4 mr-2" />
                              <span className="text-sm font-medium">{module.name}</span>
                            </div>
                          );
                        })
                      ) : (
                        <p className="text-sm text-gray-500 italic">No modules assigned</p>
                      )}
                    </div>
                  </div>
                )}
              </CardContent>
            </Card>

            {/* Module Assignment Panel */}
            <Card>
              <CardHeader>
                <CardTitle className="flex items-center">
                  <Plus className="h-5 w-5 mr-2" />
                  Assign Modules
                </CardTitle>
                <CardDescription>
                  Select modules to assign to the user
                </CardDescription>
              </CardHeader>
              <CardContent className="space-y-4">
                <div className="space-y-3">
                  <Label>Available Modules ({modules.length})</Label>
                  <div className="grid grid-cols-1 gap-2 max-h-60 overflow-y-auto">
                    {modules.map((module) => {
                      const IconComponent = getModuleIcon(module.icon);
                      const isSelected = selectedModules.includes(module.code);
                      const isAssigned = userAssignments?.modules.includes(module.code) || false;
                      
                      return (
                        <div
                          key={module.code}
                          className={`flex items-center p-3 rounded border cursor-pointer transition-colors ${
                            isSelected
                              ? 'bg-blue-100 border-blue-300'
                              : isAssigned
                              ? 'bg-gray-100 border-gray-300 opacity-50'
                              : 'bg-white border-gray-200 hover:bg-gray-50'
                          }`}
                          onClick={() => !isAssigned && toggleModuleSelection(module.code)}
                        >
                          <IconComponent className="h-4 w-4 mr-3 text-gray-600" />
                          <div className="flex-1">
                            <div className="flex items-center justify-between">
                              <span className="text-sm font-medium">{module.name}</span>
                              {isAssigned && <Badge variant="secondary" size="sm">Assigned</Badge>}
                              {isSelected && <CheckCircle className="h-4 w-4 text-blue-600" />}
                            </div>
                            <p className="text-xs text-gray-500 mt-1">{module.description}</p>
                          </div>
                        </div>
                      );
                    })}
                  </div>
                </div>

                <Separator />

                <div className="space-y-3">
                  <Label htmlFor="reason">Assignment Reason</Label>
                  <Input
                    id="reason"
                    placeholder="Enter reason for assignment (optional)"
                    value={assignmentReason}
                    onChange={(e) => setAssignmentReason(e.target.value)}
                  />
                </div>

                <Button 
                  onClick={handleAssignModules}
                  disabled={!selectedUser || selectedModules.length === 0 || isAssigning}
                  className="w-full"
                >
                  {isAssigning ? 'Assigning...' : `Assign ${selectedModules.length} Module(s)`}
                </Button>
              </CardContent>
            </Card>
          </div>
        </TabsContent>

        <TabsContent value="overview" className="space-y-6">
          <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-6">
            {modules.map((module) => {
              const IconComponent = getModuleIcon(module.icon);
              return (
                <Card key={module.code} className={getModuleColorClass(module.color)}>
                  <CardHeader className="pb-3">
                    <CardTitle className="flex items-center text-base">
                      <IconComponent className="h-5 w-5 mr-2" />
                      {module.name}
                    </CardTitle>
                    <Badge variant="outline" size="sm" className="w-fit">
                      {module.category}
                    </Badge>
                  </CardHeader>
                  <CardContent>
                    <p className="text-sm">{module.description}</p>
                  </CardContent>
                </Card>
              );
            })}
          </div>
        </TabsContent>
      </Tabs>
    </div>
  );
}