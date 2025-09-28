'use client';

import { memo, useCallback } from 'react';
import { Card, CardContent } from '@/components/ui/card';
import { Badge } from '@/components/ui/badge';
import { Button } from '@/components/ui/button';
import { 
  DropdownMenu, 
  DropdownMenuContent, 
  DropdownMenuItem, 
  DropdownMenuSeparator, 
  DropdownMenuTrigger 
} from '@/components/ui/dropdown-menu';
import { 
  CheckCircle, 
  XCircle, 
  AlertTriangle, 
  MoreHorizontal, 
  Edit, 
  Ban, 
  Trash,
  Clock,
  User,
  Shield
} from 'lucide-react';
import { format, formatDistance, isPast } from 'date-fns';

interface TemporaryPermission {
  id: string;
  user_id: string;
  permission: string;
  resource: string;
  action: string;
  expires_at: string;
  status: 'active' | 'expired' | 'revoked';
  is_expired: boolean;
  reason?: string;
  created_at: string;
  revoked_at?: string;
  revoked_by?: string;
  revoked_reason?: string;
}

interface PermissionListItemProps {
  permission: TemporaryPermission;
  onEdit?: (permission: TemporaryPermission) => void;
  onRevoke?: (permissionId: string, reason: string) => void;
  onDelete?: (permissionId: string) => void;
  enableEmbeddedTimestamps?: boolean;
}

const STATUS_COLORS = {
  active: 'bg-green-100 text-green-800 border-green-200',
  expired: 'bg-gray-100 text-gray-800 border-gray-200',
  revoked: 'bg-red-100 text-red-800 border-red-200',
};

const STATUS_COLORS_ENHANCED = {
  active: 'bg-green-100 text-green-800 border border-green-200 dark:bg-green-900/20 dark:text-green-200',
  expired: 'bg-gray-100 text-gray-800 border border-gray-200 dark:bg-gray-900/20 dark:text-gray-200',
  revoked: 'bg-red-100 text-red-800 border border-red-200 dark:bg-red-900/20 dark:text-red-200',
  expiring_soon: 'bg-yellow-100 text-yellow-800 border border-yellow-200 dark:bg-yellow-900/20 dark:text-yellow-200',
};

const STATUS_ICONS = {
  active: CheckCircle,
  expired: XCircle,
  revoked: Ban,
};

function PermissionListItem({ 
  permission, 
  onEdit, 
  onRevoke, 
  onDelete,
  enableEmbeddedTimestamps = true 
}: PermissionListItemProps) {
  
  const handleRevoke = useCallback(() => {
    if (onRevoke) {
      const reason = prompt('Please provide a reason for revoking this permission:');
      if (reason) {
        onRevoke(permission.id, reason);
      }
    }
  }, [onRevoke, permission.id]);

  const handleDelete = useCallback(() => {
    if (onDelete) {
      if (confirm('Are you sure you want to delete this permission? This action cannot be undone.')) {
        onDelete(permission.id);
      }
    }
  }, [onDelete, permission.id]);

  const handleEdit = useCallback(() => {
    if (onEdit) {
      onEdit(permission);
    }
  }, [onEdit, permission]);

  const expiryDate = new Date(permission.expires_at);
  const now = new Date();
  const timeDiff = expiryDate.getTime() - now.getTime();
  const isExpiringSoon = timeDiff > 0 && timeDiff < 24 * 60 * 60 * 1000; // 24 hours
  const isCritical = timeDiff > 0 && timeDiff < 60 * 60 * 1000; // 1 hour

  const StatusIcon = STATUS_ICONS[permission.status];

  let cardClassName = "p-4";
  let badgeClassName = STATUS_COLORS_ENHANCED[permission.status];

  if (isCritical && permission.status === 'active') {
    cardClassName = "p-4 bg-gradient-to-r from-red-50 to-rose-50 border-l-4 border-red-500 dark:from-red-950/20 dark:to-rose-950/20"
    badgeClassName = STATUS_COLORS_ENHANCED.expiring_soon
  } else if (isExpiringSoon && permission.status === 'active') {
    cardClassName = "p-4 bg-gradient-to-r from-yellow-50 to-amber-50 border-l-4 border-yellow-400 dark:from-yellow-950/20 dark:to-amber-950/20"
    badgeClassName = STATUS_COLORS_ENHANCED.expiring_soon
  } else if (permission.status === 'expired') {
    cardClassName = "p-4 bg-gradient-to-r from-gray-50 to-slate-50 border-l-4 border-gray-400 opacity-80 dark:from-gray-950/20 dark:to-slate-950/20"
  }

  return (
    <Card className={permission.is_expired ? "opacity-75" : ""}>
      <CardContent className={cardClassName}>
        <div className="flex items-start justify-between">
          <div className="flex-1">
            <div className="flex items-center gap-2 mb-3">
              <Badge className={badgeClassName}>
                <StatusIcon className="h-3 w-3 mr-1" />
                {permission.status}
              </Badge>
              
              {isCritical && permission.status === 'active' && (
                <Badge className="bg-red-100 text-red-800 border border-red-200 dark:bg-red-900/20 dark:text-red-200 opacity-75">
                  <AlertTriangle className="h-3 w-3 mr-1" />
                  Critical
                </Badge>
              )}
              
              {isExpiringSoon && !isCritical && permission.status === 'active' && (
                <Badge className="bg-yellow-100 text-yellow-800 border border-yellow-200 dark:bg-yellow-900/20 dark:text-yellow-200">
                  <Clock className="h-3 w-3 mr-1" />
                  Expiring Soon
                </Badge>
              )}
            </div>

            <div className="space-y-2">
              <div className="flex items-center gap-2">
                <Shield className="h-4 w-4 text-muted-foreground" />
                <span className="font-mono text-sm font-medium break-all">
                  {permission.permission}
                </span>
              </div>
              
              <div className="flex items-center gap-4 text-sm text-muted-foreground">
                <div className="flex items-center gap-1">
                  <span className="font-medium">Resource:</span>
                  <span className="font-mono">{permission.resource}</span>
                </div>
                <div className="flex items-center gap-1">
                  <span className="font-medium">Action:</span>
                  <span className="font-mono">{permission.action}</span>
                </div>
              </div>

              <div className="flex items-center gap-4 text-sm text-muted-foreground">
                <div className="flex items-center gap-1">
                  <Clock className="h-4 w-4" />
                  <span>
                    {permission.is_expired ? 'Expired' : 'Expires'}: 
                    <span className={permission.is_expired ? 'text-red-600 font-medium ml-1' : 'ml-1'}>
                      {format(expiryDate, 'MMM dd, yyyy HH:mm')}
                    </span>
                    {!permission.is_expired && (
                      <span className="ml-1 text-xs">
                        ({formatDistance(expiryDate, now, { addSuffix: true })})
                      </span>
                    )}
                  </span>
                </div>
              </div>

              {permission.reason && (
                <div className="text-sm text-muted-foreground">
                  <span className="font-medium">Reason:</span> {permission.reason}
                </div>
              )}

              {permission.status === 'revoked' && permission.revoked_reason && (
                <div className="text-sm text-red-600 dark:text-red-400">
                  <span className="font-medium">Revoked:</span> {permission.revoked_reason}
                  {permission.revoked_at && (
                    <span className="text-xs ml-1">
                      ({format(new Date(permission.revoked_at), 'MMM dd, yyyy HH:mm')})
                    </span>
                  )}
                </div>
              )}

              {/* TODO: Add embedded timestamp support when component is available */}
              {/* {enableEmbeddedTimestamps && (
                <AdminPermissionExpiryIndicator permission={permission.permission} />
              )} */}
            </div>
          </div>

          <div className="ml-4">
            <DropdownMenu>
              <DropdownMenuTrigger asChild>
                <Button variant="ghost" size="sm">
                  <MoreHorizontal className="h-4 w-4" />
                </Button>
              </DropdownMenuTrigger>
              <DropdownMenuContent align="end">
                {permission.status === 'active' && onEdit && (
                  <>
                    <DropdownMenuItem onClick={handleEdit}>
                      <Edit className="h-4 w-4 mr-2" />
                      Edit
                    </DropdownMenuItem>
                    <DropdownMenuSeparator />
                  </>
                )}
                
                {permission.status === 'active' && onRevoke && (
                  <DropdownMenuItem 
                    onClick={handleRevoke}
                    className="text-orange-600 focus:text-orange-600"
                  >
                    <Ban className="h-4 w-4 mr-2" />
                    Revoke
                  </DropdownMenuItem>
                )}
                
                {onDelete && (
                  <DropdownMenuItem 
                    onClick={handleDelete}
                    className="text-red-600 focus:text-red-600"
                  >
                    <Trash className="h-4 w-4 mr-2" />
                    Delete
                  </DropdownMenuItem>
                )}
              </DropdownMenuContent>
            </DropdownMenu>
          </div>
        </div>
      </CardContent>
    </Card>
  );
}

export default memo(PermissionListItem);