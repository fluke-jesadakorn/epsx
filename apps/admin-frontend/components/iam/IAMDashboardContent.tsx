'use client';

import { Activity, BarChart3, ChevronRight, Shield, Users } from 'lucide-react';
import React, { useState } from 'react';
import { DashboardOverview } from './dashboard/DashboardOverview';
import { ActivityLogs } from './logs/ActivityLogs';
import { PermissionTemplates } from './templates/PermissionTemplates';
import { UserManagement } from './users/UserManagement';

export const IAMDashboardContent: React.FC = () => {
  const [activeSection, setActiveSection] = useState('overview');

  const sections = [
    {
      id: 'overview',
      label: 'Overview',
      icon: BarChart3,
      description: 'Dashboard overview and statistics',
    },
    {
      id: 'users',
      label: 'User Management',
      icon: Users,
      description: 'Manage user accounts and permissions',
    },
    {
      id: 'templates',
      label: 'Permission Templates',
      icon: Shield,
      description: 'Create and manage permission templates',
    },
    {
      id: 'logs',
      label: 'Activity Logs',
      icon: Activity,
      description: 'View system activity and audit logs',
    },
  ];

  const renderContent = () => {
    switch (activeSection) {
      case 'overview':
        return <DashboardOverview />;
      case 'users':
        return <UserManagement />;
      case 'templates':
        return <PermissionTemplates />;
      case 'logs':
        return <ActivityLogs />;
      default:
        return <DashboardOverview />;
    }
  };

  return (
    <div className="space-y-6">
      {/* Section Header */}
      <div className="flex items-center justify-between">
        <div>
          <h1 className="text-2xl font-bold text-gray-900 dark:text-white">
            Identity & Access Management
          </h1>
          <p className="text-gray-600 dark:text-gray-400 mt-1">
            Manage users, permissions, and access control across your
            organization
          </p>
        </div>
        <div className="flex items-center gap-2 text-sm text-gray-500 dark:text-gray-400">
          <Shield className="h-4 w-4" />
          <span>IAM Console</span>
        </div>
      </div>

      {/* Section Navigation */}
      <div className="bg-white dark:bg-gray-800 rounded-lg border border-gray-200 dark:border-gray-700 p-6">
        <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-4 gap-4">
          {sections.map((section, index) => {
            const Icon = section.icon;
            const isActive = activeSection === section.id;

            return (
              <button
                key={section.id}
                onClick={() => setActiveSection(section.id)}
                className={`
                  submenu-item group p-4 rounded-lg text-left transition-all duration-200 border
                  ${
                    isActive
                      ? 'bg-gradient-to-br from-blue-50 to-indigo-50 dark:from-blue-900/20 dark:to-indigo-900/20 text-blue-700 dark:text-blue-300 border-blue-200 dark:border-blue-800 shadow-md'
                      : 'bg-gray-50 dark:bg-gray-700/50 text-gray-700 dark:text-gray-300 hover:bg-gray-100 dark:hover:bg-gray-700 border-gray-200 dark:border-gray-600 hover:border-gray-300 dark:hover:border-gray-500'
                  }
                  transform hover:scale-[1.02] active:scale-[0.98]
                `}
                style={{
                  animationDelay: `${index * 100}ms`,
                }}
              >
                <div className="flex items-center gap-3 mb-3">
                  <div
                    className={`
                    p-2.5 rounded-lg transition-all duration-200
                    ${
                      isActive
                        ? 'bg-blue-100 dark:bg-blue-800/30 shadow-sm'
                        : 'bg-white dark:bg-gray-600 group-hover:bg-gray-200 dark:group-hover:bg-gray-500'
                    }
                  `}
                  >
                    <Icon
                      className={`h-5 w-5 transition-colors ${isActive ? 'text-blue-600 dark:text-blue-400' : 'text-gray-600 dark:text-gray-400'}`}
                    />
                  </div>
                  <ChevronRight
                    className={`h-4 w-4 transition-all duration-200 ${isActive ? 'text-blue-600 dark:text-blue-400 rotate-90' : 'text-gray-400 group-hover:text-gray-600 dark:group-hover:text-gray-300'}`}
                  />
                </div>
                <div>
                  <div
                    className={`font-semibold text-sm mb-1 transition-colors ${isActive ? 'text-blue-700 dark:text-blue-300' : 'text-gray-900 dark:text-white'}`}
                  >
                    {section.label}
                  </div>
                  <div
                    className={`text-xs transition-colors ${isActive ? 'text-blue-600/80 dark:text-blue-400/80' : 'text-gray-500 dark:text-gray-400'}`}
                  >
                    {section.description}
                  </div>
                </div>
              </button>
            );
          })}
        </div>
      </div>

      {/* Content Area */}
      <div className="animate-fade-in">{renderContent()}</div>
    </div>
  );
};
