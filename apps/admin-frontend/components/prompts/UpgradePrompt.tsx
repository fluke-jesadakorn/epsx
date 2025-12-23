'use client';

import {
  AlertTriangle,
  ArrowRight,
  MessageSquare,
  Settings,
  Shield,
  UserCheck,
  X
} from 'lucide-react';
import { useState } from 'react';

import { Badge } from '@/components/ui/badge';
import { Button } from '@/components/ui/button';
import { Card, CardContent } from '@/components/ui/card';

interface AdminUpgradePromptProps {
  // Permission info
  requiredPermission?: string;
  requiredRole?: 'admin' | 'super_admin' | 'system_admin';

  // Feature context
  featureName?: string;
  description?: string;

  // Display options
  variant?: 'banner' | 'card' | 'inline' | 'tooltip';
  size?: 'sm' | 'md' | 'lg';
  dismissible?: boolean;

  // Styling
  className?: string;

  // Actions
  onRequestAccess?: () => void;
  onContactSuperAdmin?: () => void;
  onDismiss?: () => void;
  onLearnMore?: () => void;
}

const ROLE_INFO = {
  admin: {
    color: 'from-blue-500 to-indigo-500',
    bgColor: 'bg-blue-50 dark:bg-blue-900/20',
    borderColor: 'border-blue-200 dark:border-blue-800',
    textColor: 'text-blue-800 dark:text-blue-200',
    icon: UserCheck,
    description: 'Basic admin access to user management and content'
  },
  super_admin: {
    color: 'from-purple-500 to-pink-500',
    bgColor: 'bg-purple-50 dark:bg-purple-900/20',
    borderColor: 'border-purple-200 dark:border-purple-800',
    textColor: 'text-purple-800 dark:text-purple-200',
    icon: Shield,
    description: 'Advanced admin access to system configuration and security'
  },
  system_admin: {
    color: 'from-red-500 to-orange-500',
    bgColor: 'bg-red-50 dark:bg-red-900/20',
    borderColor: 'border-red-200 dark:border-red-800',
    textColor: 'text-red-800 dark:text-red-200',
    icon: Settings,
    description: 'Full system access including infrastructure and deployment'
  }
};

/**
 *
 * @param root0
 * @param root0.requiredPermission
 * @param root0.requiredRole
 * @param root0.featureName
 * @param root0.description
 * @param root0.variant
 * @param root0.size
 * @param root0.dismissible
 * @param root0.className
 * @param root0.onRequestAccess
 * @param root0.onContactSuperAdmin
 * @param root0.onDismiss
 * @param root0.onLearnMore
 */
export function AdminUpgradePrompt({
  requiredPermission,
  requiredRole,
  featureName,
  description,
  variant = 'card',
  size = 'md',
  dismissible = true,
  className = '',
  onRequestAccess,
  onContactSuperAdmin,
  onDismiss,
  onLearnMore
}: AdminUpgradePromptProps) {
  const [isDismissed, setIsDismissed] = useState(false);

  if (isDismissed) { return null; }

  // Determine which info to use
  const roleInfo = requiredRole ? ROLE_INFO[requiredRole] : ROLE_INFO.admin;
  const IconComponent = roleInfo.icon;

  const handleDismiss = () => {
    setIsDismissed(true);
    onDismiss?.();
  };

  const getTitle = () => {
    if (featureName) { return `${featureName} Access Required`; }
    if (requiredPermission) { return `Permission Required: ${requiredPermission}`; }
    if (requiredRole) { return `${requiredRole.replace('_', ' ').toUpperCase()} Group Required`; }
    return 'Administrative Access Required';
  };

  const getDescription = () => {
    if (description) { return description; }
    if (requiredPermission) { return `This feature requires the "${requiredPermission}" permission.`; }
    if (requiredRole) { return roleInfo.description; }
    return 'You need additional administrative permissions to access this feature.';
  };

  // Banner variant - full width, prominent
  if (variant === 'banner') {
    return (
      <div className={`relative overflow-hidden ${roleInfo.bgColor} ${roleInfo.borderColor} border rounded-lg p-4 ${className}`}>
        <div className={`absolute inset-0 bg-gradient-to-r ${roleInfo.color} opacity-5`} />

        <div className="relative flex items-center justify-between">
          <div className="flex items-center space-x-3">
            <div className={`p-2 rounded-full bg-gradient-to-r ${roleInfo.color}`}>
              <IconComponent className="h-5 w-5 text-white" />
            </div>
            <div>
              <h3 className="font-semibold">{getTitle()}</h3>
              <p className="text-sm text-muted-foreground">{getDescription()}</p>
            </div>
          </div>

          <div className="flex items-center space-x-2">
            {onRequestAccess && (
              <Button onClick={onRequestAccess} size="sm">
                Request Access
                <ArrowRight className="h-4 w-4 ml-1" />
              </Button>
            )}
            {onContactSuperAdmin && (
              <Button onClick={onContactSuperAdmin} variant="outline" size="sm">
                Contact Admin
              </Button>
            )}
            {dismissible && (
              <Button onClick={handleDismiss} variant="ghost" size="sm">
                <X className="h-4 w-4" />
              </Button>
            )}
          </div>
        </div>
      </div>
    );
  }

  // Card variant - contained, detailed
  if (variant === 'card') {
    return (
      <Card className={`${roleInfo.borderColor} ${className} relative`}>
        <CardContent className="p-4">
          {dismissible && (
            <button
              onClick={handleDismiss}
              className="absolute top-2 right-2 p-1 hover:bg-gray-100 dark:hover:bg-gray-800 rounded"
            >
              <X className="h-4 w-4" />
            </button>
          )}

          <div className="flex items-start space-x-4">
            <div className={`p-3 rounded-lg bg-gradient-to-br ${roleInfo.color}`}>
              <IconComponent className="h-6 w-6 text-white" />
            </div>

            <div className="flex-1">
              <div className="flex items-center space-x-2 mb-2">
                {requiredRole && (
                  <Badge className={`${roleInfo.bgColor} ${roleInfo.textColor} border-0`}>
                    {requiredRole.replace('_', ' ').toUpperCase()}
                  </Badge>
                )}
                {requiredPermission && (
                  <Badge variant="outline" className="text-xs font-mono">
                    {requiredPermission}
                  </Badge>
                )}
              </div>

              <h3 className="font-semibold mb-1">{getTitle()}</h3>
              <p className="text-sm text-muted-foreground mb-4">{getDescription()}</p>

              <div className="space-y-2 mb-4">
                <div className="flex items-center text-sm text-muted-foreground">
                  <AlertTriangle className="h-4 w-4 mr-2 text-yellow-500" />
                  This feature is restricted for security reasons
                </div>
                {onRequestAccess && (
                  <div className="flex items-center text-sm text-muted-foreground">
                    <MessageSquare className="h-4 w-4 mr-2 text-blue-500" />
                    You can request access from a super administrator
                  </div>
                )}
              </div>

              <div className="flex gap-2">
                {onRequestAccess && (
                  <Button onClick={onRequestAccess} size="sm" className="flex-1">
                    Request Access
                  </Button>
                )}
                {onContactSuperAdmin && (
                  <Button onClick={onContactSuperAdmin} variant="outline" size="sm">
                    Contact Admin
                  </Button>
                )}
                {onLearnMore && (
                  <Button onClick={onLearnMore} variant="ghost" size="sm">
                    Learn More
                  </Button>
                )}
              </div>
            </div>
          </div>
        </CardContent>
      </Card>
    );
  }

  // Inline variant - minimal, integrated
  if (variant === 'inline') {
    return (
      <div className={`flex items-center justify-between p-3 ${roleInfo.bgColor} ${roleInfo.borderColor} border rounded-md ${className}`}>
        <div className="flex items-center space-x-2">
          <IconComponent className={`h-4 w-4 ${roleInfo.textColor}`} />
          <span className="text-sm font-medium">
            {featureName ? `${featureName} ` : 'Admin access '}required
            {requiredRole && (
              <Badge className={`${roleInfo.bgColor} ${roleInfo.textColor} border-0 text-xs ml-1`}>
                {requiredRole.replace('_', ' ').toUpperCase()}
              </Badge>
            )}
          </span>
        </div>

        <div className="flex items-center space-x-2">
          {onRequestAccess && (
            <Button onClick={onRequestAccess} size="sm" variant="outline">
              Request
            </Button>
          )}
          {dismissible && (
            <Button onClick={handleDismiss} variant="ghost" size="sm">
              <X className="h-3 w-3" />
            </Button>
          )}
        </div>
      </div>
    );
  }

  // Tooltip variant - minimal overlay
  if (variant === 'tooltip') {
    return (
      <div className={`absolute inset-0 flex items-center justify-center bg-black/60 backdrop-blur-sm rounded ${className}`}>
        <div className={`p-4 ${roleInfo.bgColor} rounded-lg border ${roleInfo.borderColor} shadow-lg max-w-xs text-center`}>
          <IconComponent className={`h-8 w-8 mx-auto mb-2 ${roleInfo.textColor}`} />
          <h3 className="font-semibold text-sm mb-1">
            Access Restricted
          </h3>
          <p className="text-xs text-muted-foreground mb-3">
            {requiredRole
              ? `${requiredRole.replace('_', ' ').toUpperCase()} group required`
              : 'Administrative permission required'
            }
          </p>
          <div className="flex gap-2">
            {onRequestAccess && (
              <Button onClick={onRequestAccess} size="sm" className="flex-1 text-xs">
                Request
              </Button>
            )}
            {dismissible && (
              <Button onClick={handleDismiss} variant="outline" size="sm">
                <X className="h-3 w-3" />
              </Button>
            )}
          </div>
        </div>
      </div>
    );
  }

  return null;
}

// Convenience components for common admin scenarios
/**
 *
 * @param props
 */
export const AdminAccessPrompt = (props: Omit<AdminUpgradePromptProps, 'requiredRole'>) => (
  <AdminUpgradePrompt requiredRole="admin" {...props} />
);

/**
 *
 * @param props
 */
export const SuperAdminAccessPrompt = (props: Omit<AdminUpgradePromptProps, 'requiredRole'>) => (
  <AdminUpgradePrompt requiredRole="super_admin" {...props} />
);

/**
 *
 * @param props
 */
export const SystemAdminAccessPrompt = (props: Omit<AdminUpgradePromptProps, 'requiredRole'>) => (
  <AdminUpgradePrompt requiredRole="system_admin" {...props} />
);

/**
 *
 * @param props
 */
export const PermissionPrompt = (props: AdminUpgradePromptProps) => (
  <AdminUpgradePrompt {...props} />
);