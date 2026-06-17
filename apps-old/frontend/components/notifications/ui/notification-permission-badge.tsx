'use client';

import { Badge } from '@/components/ui/badge';
import { AlertTriangle, CheckCircle, X } from 'lucide-react';

interface NotificationPermissionBadgeProps {
  permission: NotificationPermission;
}

export function NotificationPermissionBadge({ permission }: NotificationPermissionBadgeProps) {
  switch (permission) {
    case 'granted':
      return (
        <Badge className="bg-green-100 text-green-800 dark:bg-green-900 dark:text-green-200">
          <CheckCircle className="h-3 w-3 mr-1" />
          Enabled
        </Badge>
      );
    case 'denied':
      return (
        <Badge variant="destructive">
          <X className="h-3 w-3 mr-1" />
          Blocked
        </Badge>
      );
    default:
      return (
        <Badge variant="secondary">
          <AlertTriangle className="h-3 w-3 mr-1" />
          Not Set
        </Badge>
      );
  }
}
