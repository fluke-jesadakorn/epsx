'use client';

import { Shield, Users, Key, FileText, Settings as _Settings, Plus } from 'lucide-react';
import { IAMContent } from './IAMContent';
import { StatsGrid } from '@/components/ui/StatsCard';

// Define types directly in component since we don't have the types file
interface UserWithPermissions {
  uid: string;
  email: string;
  name?: string;
  customPermissions: Array<{ id: string; name: string; granted: boolean }>;
}

interface Role {
  id: string;
  name: string;
  description?: string;
}

interface Policy {
  id: string;
  name: string;
  description?: string;
}

interface IAMDashboardProps {
  initialUsers: UserWithPermissions[];
  initialRoles: Role[];
  initialPolicies: Policy[];
}

export function IAMDashboard({ initialUsers, initialRoles, initialPolicies }: IAMDashboardProps) {
  // Create stats with icon string identifiers instead of components
  const statsData = [
    {
      title: 'Total Users',
      value: initialUsers.length,
      iconType: 'Users',
      gradient: 'from-blue-500 to-blue-600'
    },
    {
      title: 'Active Roles',
      value: initialRoles.length,
      iconType: 'Shield',
      gradient: 'from-green-500 to-green-600'
    },
    {
      title: 'Policies',
      value: initialPolicies.length,
      iconType: 'FileText',
      gradient: 'from-purple-500 to-purple-600'
    },
    {
      title: 'Custom Permissions',
      value: initialUsers.reduce((acc, user) => acc + user.customPermissions.length, 0),
      iconType: 'Key',
      gradient: 'from-orange-500 to-orange-600'
    }
  ];

  // Map icon strings to actual components
  const iconMap = {
    Users,
    Shield,
    FileText,
    Key
  };

  // Convert to stats with icon components
  const stats = statsData.map(stat => ({
    ...stat,
    icon: iconMap[stat.iconType as keyof typeof iconMap]
  }));

  return (
    <div className="p-6 space-y-6">
      {/* Header */}
      <div className="flex items-center justify-between">
        <div>
          <h1 className="text-3xl font-bold bg-gradient-to-r from-orange-500 to-yellow-500 bg-clip-text text-transparent">
            IAM Dashboard
          </h1>
          <p className="text-muted-foreground mt-2">
            Manage users, roles, policies, and permissions
          </p>
        </div>
        <button className="btn-primary flex items-center gap-2">
          <Plus className="h-4 w-4" />
          Create New
        </button>
      </div>

      {/* Stats Cards */}
      <StatsGrid 
        stats={stats}
        variant="simple"
        columns={{ default: 1, md: 2, lg: 4 }}
      />

      {/* Interactive Content */}
      <IAMContent 
        users={initialUsers}
        roles={initialRoles}
        policies={initialPolicies}
      />
    </div>
  );
}