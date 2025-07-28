import { Shield, Users, Key, FileText, Settings, Plus } from 'lucide-react';
import { IAMContent } from './IAMContent';

// Define types directly in component since we don't have the types file
interface UserWithPermissions {
  uid: string;
  email: string;
  name?: string;
  customPermissions: any[];
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
  const stats = [
    {
      title: 'Total Users',
      value: initialUsers.length,
      icon: Users,
      color: 'from-blue-500 to-blue-600'
    },
    {
      title: 'Active Roles',
      value: initialRoles.length,
      icon: Shield,
      color: 'from-green-500 to-green-600'
    },
    {
      title: 'Policies',
      value: initialPolicies.length,
      icon: FileText,
      color: 'from-purple-500 to-purple-600'
    },
    {
      title: 'Custom Permissions',
      value: initialUsers.reduce((acc, user) => acc + user.customPermissions.length, 0),
      icon: Key,
      color: 'from-orange-500 to-orange-600'
    }
  ];

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
      <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-4 gap-4">
        {stats.map((stat, index) => (
          <div key={index} className="pancake-card p-6">
            <div className="flex items-center justify-between">
              <div>
                <p className="text-sm text-muted-foreground">{stat.title}</p>
                <p className="text-2xl font-bold mt-1">{stat.value}</p>
              </div>
              <div className={`p-3 rounded-full bg-gradient-to-r ${stat.color}`}>
                <stat.icon className="h-6 w-6 text-white" />
              </div>
            </div>
          </div>
        ))}
      </div>

      {/* Interactive Content */}
      <IAMContent 
        users={initialUsers}
        roles={initialRoles}
        policies={initialPolicies}
      />
    </div>
  );
}