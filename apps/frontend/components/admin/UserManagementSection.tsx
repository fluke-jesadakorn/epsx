"use client";

import { useState, useCallback } from "react";
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card";
import { UserDetailsDialog } from "./UserDetailsDialog";
import { Input } from "@/components/ui/input";
import {
  Table,
  TableBody,
  TableCell,
  TableHead,
  TableHeader,
  TableRow,
} from "@/components/ui/table";
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from "@/components/ui/select";
import { Badge } from "@/components/ui/badge";
import { Button } from "@/components/ui/button";
import { updateUserRole } from "@/app/actions/admin-server";
import { toast } from "sonner";

interface User {
  userId: string;
  email?: string;
  role: string;
  tokenBalance: number;
  features: string[];
  permissions: string[];
}

interface UserManagementSectionProps {
  users: User[];
}

export default function UserManagementSection({ users: initialUsers }: UserManagementSectionProps) {
  const [users, setUsers] = useState<User[]>(initialUsers);
  const [searchEmail, setSearchEmail] = useState("");
  const [selectedUser, setSelectedUser] = useState<User | null>(null);
  const [dialogOpen, setDialogOpen] = useState(false);
  const [isUpdating, setIsUpdating] = useState(false);

  const availableRoles = ["admin", "premium", "basic", "public"];

  const filteredUsers = users.filter((user) =>
    user.email?.toLowerCase().includes(searchEmail.toLowerCase())
  );

  const handleRoleChange = async (userId: string, newRole: string) => {
    try {
      setIsUpdating(true);
      await updateUserRole(userId, newRole);
      
      // Update local state
      setUsers((currentUsers) =>
        currentUsers.map((user) =>
          user.userId === userId ? { ...user, role: newRole } : user
        )
      );
      
      toast.success("User role updated successfully");
    } catch (error) {
      toast.error("Failed to update user role");
      console.error("Error updating role:", error);
    } finally {
      setIsUpdating(false);
    }
  };

  const formatFeatures = (features: string[]) => {
    if (!features.length) return "None";
    return features.join(", ");
  };

  const formatPermissions = (permissions: string[]) => {
    if (!permissions.length) return "None";
    return permissions.join(", ");
  };

  return (
    <Card>
      <CardHeader>
        <CardTitle>User Management</CardTitle>
      </CardHeader>
      <CardContent>
        <div className="space-y-4">
          <Input 
            placeholder="Search by email"
            value={searchEmail}
            onChange={(e) => setSearchEmail(e.target.value)}
            className="max-w-sm mb-4"
          />

          <Table>
            <TableHeader>
              <TableRow>
                <TableHead className="w-[200px]">Email</TableHead>
                <TableHead>Role</TableHead>
                <TableHead>Token Balance</TableHead>
                <TableHead>Features</TableHead>
                <TableHead>Permissions</TableHead>
                <TableHead className="text-right">Actions</TableHead>
              </TableRow>
            </TableHeader>
            <TableBody>
              {filteredUsers.map((user) => (
                <TableRow key={user.userId}>
                  <TableCell className="font-medium">{user.email}</TableCell>
                  <TableCell>
                    <Select
                      value={user.role}
                      onValueChange={(value) => handleRoleChange(user.userId, value)}
                      disabled={isUpdating}
                    >
                      <SelectTrigger className="w-[130px]">
                        <SelectValue placeholder="Select role" />
                      </SelectTrigger>
                      <SelectContent>
                        {availableRoles.map((role) => (
                          <SelectItem key={role} value={role}>
                            {role.charAt(0).toUpperCase() + role.slice(1)}
                          </SelectItem>
                        ))}
                      </SelectContent>
                    </Select>
                  </TableCell>
                  <TableCell>
                    <Badge variant={user.tokenBalance > 0 ? "default" : "secondary"}>
                      {user.tokenBalance}
                    </Badge>
                  </TableCell>
                  <TableCell>{formatFeatures(user.features)}</TableCell>
                  <TableCell>{formatPermissions(user.permissions)}</TableCell>
                  <TableCell className="text-right">
                    <Button
                      variant="outline"
                      size="sm"
                      onClick={() => {
                        setSelectedUser(user);
                        setDialogOpen(true);
                      }}
                    >
                      View Details
                    </Button>
                  </TableCell>
                </TableRow>
              ))}
            </TableBody>
          </Table>

          {filteredUsers.length === 0 && (
            <p className="text-center text-muted-foreground py-4">
              No users found
            </p>
          )}
        </div>
      </CardContent>

      <UserDetailsDialog 
        user={selectedUser}
        open={dialogOpen}
        onOpenChange={setDialogOpen}
      />
    </Card>
  );
}
