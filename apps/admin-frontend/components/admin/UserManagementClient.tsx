'use client';

import { useState, useTransition } from 'react';
import { assignPermissionProfileAction, softDeleteUserAction } from '@/lib/actions/admin';
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '@/components/ui/card';
import { Button } from '@/components/ui/button';
import { Input } from '@/components/ui/input';
import { Label } from '@/components/ui/label';
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from '@/components/ui/select';
import { Dialog, DialogContent, DialogDescription, DialogFooter, DialogHeader, DialogTitle } from '@/components/ui/dialog';
import { Textarea } from '@/components/ui/textarea';
import { useToast } from '@/components/ui/use-toast';
import { Loader2, Trash2 } from 'lucide-react';

interface User {
  uid: string;
  email: string;
  displayName?: string;
  roles: string[];
  isAdmin: boolean;
}

interface UserManagementClientProps {
  initialData: {
    users: User[];
    total: number;
  };
}

export function UserManagementClient({ initialData }: UserManagementClientProps) {
  const [users] = useState(initialData.users);
  const [selectedUser, setSelectedUser] = useState<string>('');
  const [profileId, setProfileId] = useState<string>('');
  const [expiresAt, setExpiresAt] = useState<string>('');
  const [isPending, startTransition] = useTransition();
  const [deleteDialogOpen, setDeleteDialogOpen] = useState(false);
  const [userToDelete, setUserToDelete] = useState<string>('');
  const [deleteReason, setDeleteReason] = useState<string>('');
  const { toast } = useToast();

  const handleAssignProfile = () => {
    if (!selectedUser || !profileId) {
      toast({
        title: 'Missing Information',
        description: 'Please select both a user and permission profile',
        variant: 'destructive',
      });
      return;
    }

    startTransition(async () => {
      const formData = new FormData();
      formData.append('userId', selectedUser);
      formData.append('profileId', profileId);
      if (expiresAt) {
        formData.append('expiresAt', expiresAt);
      }

      const result = await assignPermissionProfileAction(formData);
      
      if (result.success) {
        toast({
          title: 'Success',
          description: 'Permission profile assigned successfully',
        });
        setSelectedUser('');
        setProfileId('');
        setExpiresAt('');
      } else {
        toast({
          title: 'Error',
          description: result.error || 'Failed to assign permission profile',
          variant: 'destructive',
        });
      }
    });
  };

  const handleDeleteUser = (userId: string) => {
    setUserToDelete(userId);
    setDeleteDialogOpen(true);
  };

  const confirmDeleteUser = () => {
    if (!userToDelete) return;

    startTransition(async () => {
      const formData = new FormData();
      formData.append('userId', userToDelete);
      formData.append('reason', deleteReason || 'Deleted via admin interface');

      const result = await softDeleteUserAction(formData);
      
      if (result.success) {
        toast({
          title: 'Success',
          description: 'User has been deleted successfully',
        });
        setDeleteDialogOpen(false);
        setUserToDelete('');
        setDeleteReason('');
      } else {
        toast({
          title: 'Error',
          description: result.error || 'Failed to delete user',
          variant: 'destructive',
        });
      }
    });
  };

  return (
    <div className="space-y-6">
      <Card>
        <CardHeader>
          <CardTitle>Users ({initialData.total})</CardTitle>
          <CardDescription>
            Manage user permissions and access
          </CardDescription>
        </CardHeader>
        <CardContent>
          <div className="space-y-4">
            {users.map((user) => (
              <div key={user.uid} className="flex items-center justify-between p-4 border rounded-lg">
                <div>
                  <p className="font-medium">{user.displayName || user.email}</p>
                  <p className="text-sm text-muted-foreground">{user.email}</p>
                  <div className="flex gap-2 mt-1">
                    {user.roles.map((role) => (
                      <span key={role} className="px-2 py-1 text-xs bg-blue-100 text-blue-800 rounded">
                        {role}
                      </span>
                    ))}
                  </div>
                </div>
                <div className="flex gap-2">
                  <Button
                    variant="outline"
                    size="sm"
                    onClick={() => setSelectedUser(user.uid)}
                  >
                    Manage
                  </Button>
                  <Button
                    variant="outline"
                    size="sm"
                    onClick={() => handleDeleteUser(user.uid)}
                    className="text-red-600 hover:text-red-700"
                  >
                    <Trash2 className="h-4 w-4" />
                  </Button>
                </div>
              </div>
            ))}
          </div>
        </CardContent>
      </Card>

      <Card>
        <CardHeader>
          <CardTitle>Assign Permission Profile</CardTitle>
          <CardDescription>
            Assign a permission profile to a user
          </CardDescription>
        </CardHeader>
        <CardContent className="space-y-4">
          <div className="space-y-2">
            <Label htmlFor="user-select">Select User</Label>
            <Select value={selectedUser} onValueChange={setSelectedUser}>
              <SelectTrigger>
                <SelectValue placeholder="Select a user..." />
              </SelectTrigger>
              <SelectContent>
                {users.map((user) => (
                  <SelectItem key={user.uid} value={user.uid}>
                    {user.displayName || user.email}
                  </SelectItem>
                ))}
              </SelectContent>
            </Select>
          </div>

          <div className="space-y-2">
            <Label htmlFor="profile-select">Permission Profile</Label>
            <Select value={profileId} onValueChange={setProfileId}>
              <SelectTrigger>
                <SelectValue placeholder="Select a profile..." />
              </SelectTrigger>
              <SelectContent>
                <SelectItem value="premium">Premium Access</SelectItem>
                <SelectItem value="basic">Basic Access</SelectItem>
                <SelectItem value="admin">Admin Access</SelectItem>
              </SelectContent>
            </Select>
          </div>

          <div className="space-y-2">
            <Label htmlFor="expires-at">Expires At (Optional)</Label>
            <Input
              id="expires-at"
              type="datetime-local"
              value={expiresAt}
              onChange={(e) => setExpiresAt(e.target.value)}
            />
          </div>

          <Button 
            onClick={handleAssignProfile} 
            disabled={isPending || !selectedUser || !profileId}
            className="w-full"
          >
            {isPending ? (
              <>
                <Loader2 className="mr-2 h-4 w-4 animate-spin" />
                Assigning...
              </>
            ) : (
              'Assign Permission Profile'
            )}
          </Button>
        </CardContent>
      </Card>

      <Dialog open={deleteDialogOpen} onOpenChange={setDeleteDialogOpen}>
        <DialogContent>
          <DialogHeader>
            <DialogTitle>Delete User</DialogTitle>
            <DialogDescription>
              Are you sure you want to delete this user? This action will soft delete the user account.
            </DialogDescription>
          </DialogHeader>
          
          <div className="space-y-4">
            <div className="space-y-2">
              <Label htmlFor="delete-reason">Reason for deletion (optional)</Label>
              <Textarea
                id="delete-reason"
                placeholder="Enter reason for deleting this user..."
                value={deleteReason}
                onChange={(e) => setDeleteReason(e.target.value)}
              />
            </div>
          </div>

          <DialogFooter>
            <Button
              variant="outline"
              onClick={() => {
                setDeleteDialogOpen(false);
                setUserToDelete('');
                setDeleteReason('');
              }}
              disabled={isPending}
            >
              Cancel
            </Button>
            <Button
              variant="destructive"
              onClick={confirmDeleteUser}
              disabled={isPending}
            >
              {isPending ? (
                <>
                  <Loader2 className="mr-2 h-4 w-4 animate-spin" />
                  Deleting...
                </>
              ) : (
                'Delete User'
              )}
            </Button>
          </DialogFooter>
        </DialogContent>
      </Dialog>
    </div>
  );
}