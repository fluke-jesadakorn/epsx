'use client';

import { Shield } from 'lucide-react';
import { useSearchParams } from 'next/navigation';
import React from 'react';
import { DashboardOverview } from './dashboard/DashboardOverview';
import { ActivityLogs } from './logs/ActivityLogs';
import { PermissionTemplates } from './templates/PermissionTemplates';

export const IAMDashboardContent: React.FC = () => {
  const searchParams = useSearchParams();
  const activeSection = searchParams.get('section') || 'overview';

  const renderContent = () => {
    switch (activeSection) {
      case 'overview':
        return <DashboardOverview />;
      case 'templates':
        return <PermissionTemplates />;
      case 'logs':
        return <ActivityLogs />;
      default:
        return <DashboardOverview />;
    }
  };

  const getSectionTitle = () => {
    switch (activeSection) {
      case 'overview':
        return 'Identity & Access Management Overview';
      case 'templates':
        return 'Permission Templates';
      case 'logs':
        return 'Activity Logs';
      default:
        return 'Identity & Access Management';
    }
  };

  const getSectionDescription = () => {
    switch (activeSection) {
      case 'overview':
        return 'Dashboard overview and system statistics';
      case 'templates':
        return 'Create and manage permission templates';
      case 'logs':
        return 'View system activity and audit logs';
      default:
        return 'Manage users, permissions, and access control across your organization';
    }
  };

  return (
    <div className="space-y-6">
      {/* Section Header */}
      <div className="flex items-center justify-between">
        <div>
          <h1 className="text-2xl font-bold text-gray-900 dark:text-white">
            {getSectionTitle()}
          </h1>
          <p className="text-gray-600 dark:text-gray-400 mt-1">
            {getSectionDescription()}
          </p>
        </div>
        <div className="flex items-center gap-2 text-sm text-gray-500 dark:text-gray-400">
          <Shield className="h-4 w-4" />
          <span>IAM Console</span>
        </div>
      </div>

      {/* Content Area */}
      <div className="animate-fade-in">{renderContent()}</div>
    </div>
  );
};
