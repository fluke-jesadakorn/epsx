"use client";

import React, { useState, useRef, useCallback } from 'react';
import { Card, CardContent, CardHeader, CardTitle } from '@epsx/ui';
import { Button } from '@epsx/ui';
import { Badge } from '@epsx/ui';
import { Input } from '@epsx/ui';
import { 
  Users, 
  Shield, 
  Search, 
  ChevronDown,
  ChevronUp,
  Plus,
  Trash2,
  Move,
  CheckCircle,
  AlertCircle
} from 'lucide-react';

interface DragItem {
  id: string;
  type: 'user' | 'profile' | 'permission';
  name: string;
  email?: string;
  description?: string;
  riskLevel?: 'low' | 'medium' | 'high';
  category?: string;
  currentAssignments?: string[];
}

interface DropZone {
  id: string;
  name: string;
  type: 'user-group' | 'permission-profile' | 'bulk-action';
  accepts: Array<'user' | 'profile' | 'permission'>;
  items: DragItem[];
  maxItems?: number;
  color: string;
}

interface Assignment {
  id: string;
  userId: string;
  profileId: string;
  status: 'pending' | 'applying' | 'success' | 'error';
  error?: string;
}

interface DragDropBulkAssignmentProps {
  users: DragItem[];
  profiles: DragItem[];
  permissions: DragItem[];
  onBulkAssign?: (assignments: Assignment[]) => Promise<void>;
  className?: string;
}

export function DragDropBulkAssignment({
  users,
  profiles,
  permissions,
  onBulkAssign,
  className = ''
}: DragDropBulkAssignmentProps) {
  const [dropZones, setDropZones] = useState<DropZone[]>([
    {
      id: 'users',
      name: 'Selected Users',
      type: 'user-group',
      accepts: ['user'],
      items: [],
      color: 'border-blue-300 bg-blue-50'
    },
    {
      id: 'profiles',
      name: 'Permission Profiles to Assign',
      type: 'permission-profile',
      accepts: ['profile'],
      items: [],
      color: 'border-green-300 bg-green-50'
    },
    {
      id: 'permissions',
      name: 'Individual Permissions',
      type: 'bulk-action',
      accepts: ['permission'],
      items: [],
      color: 'border-purple-300 bg-purple-50'
    }
  ]);

  const [draggedItem, setDraggedItem] = useState<DragItem | null>(null);
  const [assignments, setAssignments] = useState<Assignment[]>([]);
  const [searchTerms, setSearchTerms] = useState({
    users: '',
    profiles: '',
    permissions: ''
  });
  const [collapsed, setCollapsed] = useState({
    users: false,
    profiles: false,
    permissions: false
  });

  const dragOverCounter = useRef<Map<string, number>>(new Map());

  const handleDragStart = (e: React.DragEvent, item: DragItem) => {
    setDraggedItem(item);
    e.dataTransfer.effectAllowed = 'copy';
    e.dataTransfer.setData('text/plain', JSON.stringify(item));
  };

  const handleDragOver = (e: React.DragEvent, zoneId: string) => {
    e.preventDefault();
    const zone = dropZones.find(z => z.id === zoneId);
    if (!zone || !draggedItem) return;

    if (zone.accepts.includes(draggedItem.type)) {
      e.dataTransfer.dropEffect = 'copy';
    } else {
      e.dataTransfer.dropEffect = 'none';
    }
  };

  const handleDragEnter = (e: React.DragEvent, zoneId: string) => {
    e.preventDefault();
    const currentCount = dragOverCounter.current.get(zoneId) || 0;
    dragOverCounter.current.set(zoneId, currentCount + 1);

    setDropZones(zones => zones.map(zone => 
      zone.id === zoneId 
        ? { ...zone, color: zone.color + ' ring-2 ring-blue-400' }
        : zone
    ));
  };

  const handleDragLeave = (e: React.DragEvent, zoneId: string) => {
    const currentCount = dragOverCounter.current.get(zoneId) || 0;
    const newCount = Math.max(0, currentCount - 1);
    dragOverCounter.current.set(zoneId, newCount);

    if (newCount === 0) {
      setDropZones(zones => zones.map(zone => 
        zone.id === zoneId 
          ? { ...zone, color: zone.color.replace(' ring-2 ring-blue-400', '') }
          : zone
      ));
    }
  };

  const handleDrop = (e: React.DragEvent, zoneId: string) => {
    e.preventDefault();
    dragOverCounter.current.set(zoneId, 0);

    const zone = dropZones.find(z => z.id === zoneId);
    if (!zone || !draggedItem) return;

    if (!zone.accepts.includes(draggedItem.type)) return;

    // Check if item already exists in zone
    const itemExists = zone.items.some(item => item.id === draggedItem.id);
    if (itemExists) return;

    // Check max items limit
    if (zone.maxItems && zone.items.length >= zone.maxItems) return;

    setDropZones(zones => zones.map(z => 
      z.id === zoneId 
        ? { 
            ...z, 
            items: [...z.items, draggedItem],
            color: z.color.replace(' ring-2 ring-blue-400', '')
          }
        : z
    ));

    setDraggedItem(null);
  };

  const removeFromZone = (zoneId: string, itemId: string) => {
    setDropZones(zones => zones.map(zone => 
      zone.id === zoneId 
        ? { ...zone, items: zone.items.filter(item => item.id !== itemId) }
        : zone
    ));
  };

  const clearZone = (zoneId: string) => {
    setDropZones(zones => zones.map(zone => 
      zone.id === zoneId ? { ...zone, items: [] } : zone
    ));
  };

  const generateAssignments = useCallback(() => {
    const userZone = dropZones.find(z => z.id === 'users');
    const profileZone = dropZones.find(z => z.id === 'profiles');
    
    if (!userZone?.items.length || !profileZone?.items.length) return [];

    const newAssignments: Assignment[] = [];
    
    userZone.items.forEach(user => {
      profileZone.items.forEach(profile => {
        newAssignments.push({
          id: `${user.id}-${profile.id}`,
          userId: user.id,
          profileId: profile.id,
          status: 'pending'
        });
      });
    });

    return newAssignments;
  }, [dropZones]);

  const handleExecuteAssignments = async () => {
    const newAssignments = generateAssignments();
    setAssignments(newAssignments);

    if (onBulkAssign) {
      try {
        await onBulkAssign(newAssignments);
        setAssignments(prev => prev.map(a => ({ ...a, status: 'success' })));
      } catch (error) {
        setAssignments(prev => prev.map(a => ({ 
          ...a, 
          status: 'error',
          error: error instanceof Error ? error.message : 'Unknown error'
        })));
      }
    }
  };

  const filteredItems = {
    users: users.filter(user => 
      user.name.toLowerCase().includes(searchTerms.users.toLowerCase()) ||
      user.email?.toLowerCase().includes(searchTerms.users.toLowerCase())
    ),
    profiles: profiles.filter(profile =>
      profile.name.toLowerCase().includes(searchTerms.profiles.toLowerCase()) ||
      profile.description?.toLowerCase().includes(searchTerms.profiles.toLowerCase())
    ),
    permissions: permissions.filter(permission =>
      permission.name.toLowerCase().includes(searchTerms.permissions.toLowerCase()) ||
      permission.description?.toLowerCase().includes(searchTerms.permissions.toLowerCase())
    )
  };

  const getRiskColor = (risk?: string) => {
    switch (risk) {
      case 'low': return 'text-green-600 bg-green-100';
      case 'medium': return 'text-orange-600 bg-orange-100';
      case 'high': return 'text-red-600 bg-red-100';
      default: return 'text-gray-600 bg-gray-100';
    }
  };

  return (
    <div className={`space-y-6 ${className}`}>
      {/* Header */}
      <Card>
        <CardHeader>
          <CardTitle className="flex items-center gap-2">
            <Move className="h-5 w-5" />
            Drag & Drop Bulk Assignment
          </CardTitle>
          <p className="text-sm text-gray-600">
            Drag users and permission profiles to the assignment zones to create bulk assignments
          </p>
        </CardHeader>
      </Card>

      <div className="grid grid-cols-1 lg:grid-cols-3 gap-6">
        {/* Source Items */}
        <div className="space-y-4">
          {/* Users */}
          <Card>
            <CardHeader className="pb-3">
              <div className="flex justify-between items-center">
                <div className="flex items-center gap-2">
                  <Users className="h-4 w-4" />
                  <span className="font-medium">Users ({filteredItems.users.length})</span>
                </div>
                <Button
                  variant="ghost"
                  size="sm"
                  onClick={() => setCollapsed(prev => ({ ...prev, users: !prev.users }))}
                >
                  {collapsed.users ? <ChevronDown className="h-4 w-4" /> : <ChevronUp className="h-4 w-4" />}
                </Button>
              </div>
              <div className="relative">
                <Search className="absolute left-2 top-1/2 transform -translate-y-1/2 h-4 w-4 text-gray-400" />
                <Input
                  placeholder="Search users..."
                  value={searchTerms.users}
                  onChange={(e) => setSearchTerms(prev => ({ ...prev, users: e.target.value }))}
                  className="pl-8 h-8"
                />
              </div>
            </CardHeader>
            {!collapsed.users && (
              <CardContent className="pt-0">
                <div className="space-y-2 max-h-64 overflow-y-auto">
                  {filteredItems.users.map(user => (
                    <div
                      key={user.id}
                      draggable
                      onDragStart={(e) => handleDragStart(e, user)}
                      className="flex items-center gap-3 p-2 border rounded-lg cursor-move hover:bg-gray-50 transition-colors"
                    >
                      <Users className="h-4 w-4 text-blue-500 flex-shrink-0" />
                      <div className="flex-1 min-w-0">
                        <p className="font-medium text-sm truncate">{user.name}</p>
                        <p className="text-xs text-gray-600 truncate">{user.email}</p>
                      </div>
                    </div>
                  ))}
                </div>
              </CardContent>
            )}
          </Card>

          {/* Profiles */}
          <Card>
            <CardHeader className="pb-3">
              <div className="flex justify-between items-center">
                <div className="flex items-center gap-2">
                  <Shield className="h-4 w-4" />
                  <span className="font-medium">Profiles ({filteredItems.profiles.length})</span>
                </div>
                <Button
                  variant="ghost"
                  size="sm"
                  onClick={() => setCollapsed(prev => ({ ...prev, profiles: !prev.profiles }))}
                >
                  {collapsed.profiles ? <ChevronDown className="h-4 w-4" /> : <ChevronUp className="h-4 w-4" />}
                </Button>
              </div>
              <div className="relative">
                <Search className="absolute left-2 top-1/2 transform -translate-y-1/2 h-4 w-4 text-gray-400" />
                <Input
                  placeholder="Search profiles..."
                  value={searchTerms.profiles}
                  onChange={(e) => setSearchTerms(prev => ({ ...prev, profiles: e.target.value }))}
                  className="pl-8 h-8"
                />
              </div>
            </CardHeader>
            {!collapsed.profiles && (
              <CardContent className="pt-0">
                <div className="space-y-2 max-h-64 overflow-y-auto">
                  {filteredItems.profiles.map(profile => (
                    <div
                      key={profile.id}
                      draggable
                      onDragStart={(e) => handleDragStart(e, profile)}
                      className="flex items-center gap-3 p-2 border rounded-lg cursor-move hover:bg-gray-50 transition-colors"
                    >
                      <Shield className="h-4 w-4 text-green-500 flex-shrink-0" />
                      <div className="flex-1 min-w-0">
                        <div className="flex items-center gap-2">
                          <p className="font-medium text-sm truncate">{profile.name}</p>
                          {profile.riskLevel && (
                            <Badge className={`text-xs ${getRiskColor(profile.riskLevel)}`}>
                              {profile.riskLevel}
                            </Badge>
                          )}
                        </div>
                        {profile.description && (
                          <p className="text-xs text-gray-600 truncate">{profile.description}</p>
                        )}
                      </div>
                    </div>
                  ))}
                </div>
              </CardContent>
            )}
          </Card>
        </div>

        {/* Drop Zones */}
        <div className="space-y-4">
          {dropZones.slice(0, 2).map(zone => (
            <Card key={zone.id} className={`transition-all ${zone.color}`}>
              <CardHeader className="pb-3">
                <div className="flex justify-between items-center">
                  <div className="flex items-center gap-2">
                    {zone.type === 'user-group' ? (
                      <Users className="h-4 w-4" />
                    ) : (
                      <Shield className="h-4 w-4" />
                    )}
                    <span className="font-medium">{zone.name} ({zone.items.length})</span>
                  </div>
                  {zone.items.length > 0 && (
                    <Button
                      variant="outline"
                      size="sm"
                      onClick={() => clearZone(zone.id)}
                    >
                      <Trash2 className="h-3 w-3" />
                    </Button>
                  )}
                </div>
              </CardHeader>
              <CardContent
                onDragOver={(e) => handleDragOver(e, zone.id)}
                onDragEnter={(e) => handleDragEnter(e, zone.id)}
                onDragLeave={(e) => handleDragLeave(e, zone.id)}
                onDrop={(e) => handleDrop(e, zone.id)}
                className="pt-0 min-h-32"
              >
                {zone.items.length === 0 ? (
                  <div className="flex items-center justify-center h-24 text-gray-400 border-2 border-dashed border-gray-300 rounded-lg">
                    <div className="text-center">
                      <Plus className="h-8 w-8 mx-auto mb-2" />
                      <p className="text-sm">Drop {zone.accepts.join(' or ')} here</p>
                    </div>
                  </div>
                ) : (
                  <div className="space-y-2 max-h-48 overflow-y-auto">
                    {zone.items.map(item => (
                      <div key={item.id} className="flex items-center gap-2 p-2 bg-white border rounded-lg">
                        <div className="flex-1 min-w-0">
                          <p className="font-medium text-sm truncate">{item.name}</p>
                          {item.email && (
                            <p className="text-xs text-gray-600 truncate">{item.email}</p>
                          )}
                        </div>
                        <Button
                          variant="ghost"
                          size="sm"
                          onClick={() => removeFromZone(zone.id, item.id)}
                          className="h-6 w-6 p-0"
                        >
                          <Trash2 className="h-3 w-3" />
                        </Button>
                      </div>
                    ))}
                  </div>
                )}
              </CardContent>
            </Card>
          ))}
        </div>

        {/* Assignment Preview & Actions */}
        <div className="space-y-4">
          <Card>
            <CardHeader>
              <CardTitle className="text-lg">Assignment Preview</CardTitle>
            </CardHeader>
            <CardContent>
              {(() => {
                const previewAssignments = generateAssignments();
                return (
                  <div>
                    <div className="mb-4">
                      <p className="text-sm text-gray-600">
                        {previewAssignments.length} assignments will be created
                      </p>
                      <div className="mt-2 space-y-1 max-h-32 overflow-y-auto">
                        {previewAssignments.slice(0, 5).map(assignment => {
                          const user = users.find(u => u.id === assignment.userId);
                          const profile = profiles.find(p => p.id === assignment.profileId);
                          return (
                            <div key={assignment.id} className="text-xs text-gray-600">
                              {user?.name} → {profile?.name}
                            </div>
                          );
                        })}
                        {previewAssignments.length > 5 && (
                          <div className="text-xs text-gray-500">
                            +{previewAssignments.length - 5} more assignments
                          </div>
                        )}
                      </div>
                    </div>

                    <Button
                      onClick={handleExecuteAssignments}
                      disabled={previewAssignments.length === 0}
                      className="w-full"
                    >
                      Execute Assignments
                    </Button>
                  </div>
                );
              })()}
            </CardContent>
          </Card>

          {/* Assignment Status */}
          {assignments.length > 0 && (
            <Card>
              <CardHeader>
                <CardTitle className="text-lg">Assignment Status</CardTitle>
              </CardHeader>
              <CardContent>
                <div className="space-y-2">
                  {assignments.slice(0, 10).map(assignment => {
                    const user = users.find(u => u.id === assignment.userId);
                    const profile = profiles.find(p => p.id === assignment.profileId);
                    
                    return (
                      <div key={assignment.id} className="flex items-center justify-between p-2 border rounded">
                        <div className="flex-1 min-w-0">
                          <p className="text-sm font-medium truncate">
                            {user?.name} → {profile?.name}
                          </p>
                          {assignment.error && (
                            <p className="text-xs text-red-600 truncate">{assignment.error}</p>
                          )}
                        </div>
                        <div className="flex items-center">
                          {assignment.status === 'pending' && (
                            <AlertCircle className="h-4 w-4 text-yellow-500" />
                          )}
                          {assignment.status === 'applying' && (
                            <div className="animate-spin rounded-full h-4 w-4 border-2 border-blue-500 border-t-transparent" />
                          )}
                          {assignment.status === 'success' && (
                            <CheckCircle className="h-4 w-4 text-green-500" />
                          )}
                          {assignment.status === 'error' && (
                            <AlertCircle className="h-4 w-4 text-red-500" />
                          )}
                        </div>
                      </div>
                    );
                  })}
                  {assignments.length > 10 && (
                    <p className="text-xs text-gray-500 text-center">
                      +{assignments.length - 10} more assignments
                    </p>
                  )}
                </div>
              </CardContent>
            </Card>
          )}
        </div>
      </div>
    </div>
  );
}