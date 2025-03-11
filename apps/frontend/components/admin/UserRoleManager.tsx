"use client";

import { useState } from "react";
import { useAuth } from "@/context/auth-context";
import { updateUserRole } from "@/app/actions/admin-server";
import { UserRole } from "@/types/auth/roles";
import {
  Card,
  CardContent,
  CardDescription,
  CardHeader,
  CardTitle,
} from "@/components/ui/card";
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from "@/components/ui/select";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import { toast } from "sonner";
import { useTransition } from "react";

interface User {
  userId: string;
  email?: string;
  role: UserRole;
}

interface UserRoleManagerProps {
  users: User[];
}

export default function UserRoleManager({
  users: initialUsers,
}: UserRoleManagerProps) {
  const users = initialUsers; // No need for state since we're not modifying the users array
  const [selectedUser, setSelectedUser] = useState<string>("");
  const [selectedRole, setSelectedRole] = useState<UserRole>();
  const [searchEmail, setSearchEmail] = useState("");
  const [isAssigning, startAssignTransition] = useTransition();
  const { checkStatus } = useAuth();

  const handleAssignRole = async () => {
    if (!selectedUser || !selectedRole) {
      toast.error("Please select both user and role");
      return;
    }

    startAssignTransition(async () => {
      try {
        const result = await updateUserRole(selectedUser, selectedRole);
        if (result.success) {
          toast.success("Role assigned successfully");
        }

        // Refresh auth context and user list
        await checkStatus();

        // Reset selections
        setSelectedUser("");
        setSelectedRole(undefined);
      } catch (error) {
        console.error("Error assigning role:", error);
        toast.error("Failed to assign role");
      }
    });
  };

  const availableRoles = [
    UserRole.ADMINISTRATOR,
    UserRole.TOKEN_HOLDER,
    UserRole.PREMIUM_USER,
    UserRole.REGISTERED_USER,
    UserRole.GUEST
  ] as const;

  const filteredUsers = users.filter((user) =>
    user.email?.toLowerCase().includes(searchEmail.toLowerCase())
  );

  return (
    <Card className="w-full max-w-2xl mx-auto">
      <CardHeader>
        <CardTitle>User Role Management</CardTitle>
        <CardDescription>Assign roles to users</CardDescription>
      </CardHeader>
      <CardContent>
        <div className="space-y-4">
          <div>
            <Input
              placeholder="Search by email"
              value={searchEmail}
              onChange={(e) => setSearchEmail(e.target.value)}
              className="mb-4"
            />
          </div>

          <div className="flex flex-col space-y-4">
            <Select value={selectedUser} onValueChange={setSelectedUser}>
              <SelectTrigger>
                <SelectValue placeholder="Select user" />
              </SelectTrigger>
              <SelectContent>
                {filteredUsers.map((user) => (
                  <SelectItem key={user.userId} value={user.userId}>
                    {user.email} ({user.role})
                  </SelectItem>
                ))}
              </SelectContent>
            </Select>

            <Select
              value={selectedRole}
              onValueChange={(value) => setSelectedRole(value as UserRole)}
            >
              <SelectTrigger>
                <SelectValue placeholder="Select role" />
              </SelectTrigger>
              <SelectContent>
                {availableRoles.map((role) => (
                  <SelectItem key={role} value={role}>
                    {role.charAt(0).toUpperCase() + role.slice(1).toLowerCase()}
                  </SelectItem>
                ))}
              </SelectContent>
            </Select>

            <Button
              onClick={handleAssignRole}
              disabled={isAssigning || !selectedUser || !selectedRole}
              className="bg-primary hover:bg-primary/90"
            >
              {isAssigning ? "Assigning..." : "Assign Role"}
            </Button>

            {users.length === 0 && (
              <p className="text-sm text-muted-foreground text-center">
                No users found
              </p>
            )}
          </div>
        </div>
      </CardContent>
    </Card>
  );
}
