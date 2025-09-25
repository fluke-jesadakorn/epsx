/**
 * Permission Request Modal
 * Stub implementation for build compatibility
 */

'use client';

import { Dialog, DialogContent, DialogHeader, DialogTitle } from '@/components/ui/dialog';

interface PermissionRequestModalProps {
  isOpen: boolean;
  onClose: () => void;
  requiredPermission?: string;
  requiredRole?: string;
  itemLabel?: string;
  itemDescription?: string;
}

export function PermissionRequestModal({
  isOpen,
  onClose,
  requiredPermission,
  requiredRole,
  itemLabel,
  itemDescription
}: PermissionRequestModalProps) {
  return (
    <Dialog open={isOpen} onOpenChange={onClose}>
      <DialogContent>
        <DialogHeader>
          <DialogTitle>Permission Required</DialogTitle>
        </DialogHeader>
        <div className="p-4">
          <p>You need additional permissions to access this feature.</p>
          {requiredPermission && (
            <p className="mt-2 text-sm text-gray-600">
              Required permission: {requiredPermission}
            </p>
          )}
          {requiredRole && (
            <p className="mt-2 text-sm text-gray-600">
              Required role: {requiredRole}
            </p>
          )}
        </div>
      </DialogContent>
    </Dialog>
  );
}