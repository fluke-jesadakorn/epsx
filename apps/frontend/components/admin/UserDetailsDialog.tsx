"use client";

import {
  Dialog,
  DialogContent,
  DialogDescription,
  DialogHeader,
  DialogTitle,
} from "@/components/ui/dialog";
import { ScrollArea } from "@/components/ui/scroll-area";
import { Badge } from "@/components/ui/badge";

interface User {
  userId: string;
  email?: string;
  role: string;
  tokenBalance: number;
  features: string[];
  permissions: string[];
}

interface UserDetailsDialogProps {
  user: User | null;
  open: boolean;
  onOpenChange: (open: boolean) => void;
}

export function UserDetailsDialog({ user, open, onOpenChange }: UserDetailsDialogProps) {
  if (!user) return null;

  return (
    <Dialog open={open} onOpenChange={onOpenChange}>
      <DialogContent className="max-w-2xl max-h-[80vh] overflow-hidden flex flex-col">
        <DialogHeader>
          <DialogTitle>User Details</DialogTitle>
          <DialogDescription>
            Detailed information for {user.email}
          </DialogDescription>
        </DialogHeader>
        
        <ScrollArea className="flex-grow">
          <div className="space-y-6 p-4">
            {/* Basic Information */}
            <div className="space-y-2">
              <h3 className="text-lg font-semibold">Basic Information</h3>
              <div className="grid grid-cols-2 gap-2 text-sm">
                <div className="text-muted-foreground">User ID</div>
                <div>{user.userId}</div>
                <div className="text-muted-foreground">Email</div>
                <div>{user.email}</div>
                <div className="text-muted-foreground">Role</div>
                <div>
                  <Badge className="capitalize">{user.role}</Badge>
                </div>
              </div>
            </div>

            {/* Token Balance */}
            <div className="space-y-2">
              <h3 className="text-lg font-semibold">Token Balance</h3>
              <div className="text-3xl font-bold">
                <Badge variant="secondary" className="text-xl px-4 py-1">
                  {user.tokenBalance}
                </Badge>
              </div>
            </div>

            {/* Features */}
            <div className="space-y-2">
              <h3 className="text-lg font-semibold">Features</h3>
              {user.features.length > 0 ? (
                <div className="flex flex-wrap gap-2">
                  {user.features.map((feature, index) => (
                    <Badge key={index} variant="outline">
                      {feature}
                    </Badge>
                  ))}
                </div>
              ) : (
                <p className="text-sm text-muted-foreground">No features assigned</p>
              )}
            </div>

            {/* Permissions */}
            <div className="space-y-2">
              <h3 className="text-lg font-semibold">Permissions</h3>
              {user.permissions.length > 0 ? (
                <div className="flex flex-wrap gap-2">
                  {user.permissions.map((permission, index) => (
                    <Badge key={index} variant="outline">
                      {permission}
                    </Badge>
                  ))}
                </div>
              ) : (
                <p className="text-sm text-muted-foreground">No permissions assigned</p>
              )}
            </div>
          </div>
        </ScrollArea>
      </DialogContent>
    </Dialog>
  );
}
