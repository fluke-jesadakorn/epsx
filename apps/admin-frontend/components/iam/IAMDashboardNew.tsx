'use client';

import React, { useState } from 'react';
import { Tabs, TabsContent, TabsList, TabsTrigger } from '../ui/tabs';
import { DashboardOverview } from './dashboard/DashboardOverview';
import { UserManagement } from './users/UserManagement';
import { PermissionTemplates } from './templates/PermissionTemplates';
import { ActivityLogs } from './logs/ActivityLogs';
import { Users, Shield, Activity, BarChart3 } from 'lucide-react';

export const IAMDashboardNew: React.FC = () => {
  const [activeTab, setActiveTab] = useState('overview');

  return (
    <div className="container mx-auto p-6">
      <div className="mb-8">
        <h1 className="text-3xl font-bold tracking-tight">Identity & Access Management</h1>
        <p className="text-muted-foreground">
          Manage users, permissions, and access control across your organization
        </p>
      </div>

      <Tabs value={activeTab} onValueChange={setActiveTab} className="space-y-6">
        <TabsList className="grid w-full grid-cols-4">
          <TabsTrigger value="overview" className="flex items-center gap-2">
            <BarChart3 className="h-4 w-4" />
            Overview
          </TabsTrigger>
          <TabsTrigger value="users" className="flex items-center gap-2">
            <Users className="h-4 w-4" />
            Users
          </TabsTrigger>
          <TabsTrigger value="templates" className="flex items-center gap-2">
            <Shield className="h-4 w-4" />
            Templates
          </TabsTrigger>
          <TabsTrigger value="logs" className="flex items-center gap-2">
            <Activity className="h-4 w-4" />
            Activity Logs
          </TabsTrigger>
        </TabsList>

        <TabsContent value="overview" className="space-y-6">
          <DashboardOverview />
        </TabsContent>

        <TabsContent value="users" className="space-y-6">
          <UserManagement />
        </TabsContent>

        <TabsContent value="templates" className="space-y-6">
          <PermissionTemplates />
        </TabsContent>

        <TabsContent value="logs" className="space-y-6">
          <ActivityLogs />
        </TabsContent>
      </Tabs>
    </div>
  );
};
