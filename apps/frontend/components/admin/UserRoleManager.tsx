"use client";

import { useState } from "react";
import { UserRole } from "@/constants/roles";
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

export function UserRoleManager({ users: initialUsers }: UserRoleManagerProps) {
  const [users, setUsers] = useState<User[]>(initialUsers);
  const [selectedUser, setSelectedUser] = useState<string>("");
  const [selectedRole, setSelectedRole] = useState<UserRole>();
  const [searchEmail, setSearchEmail] = useState("");
  const [isAssigning, startAssignTransition] = useTransition();

  const handleAssignRole = async () => {
    if (!selectedUser || !selectedRole) {
      toast.error("Please select both user and role");
      return;
    }

    startAssignTransition(async () => {
      try {
        const response = await fetch(`${process.env.NEXT_PUBLIC_API_URL}/auth/roles/assign`, {
          method: 'POST',
          headers: {
            'Content-Type': 'application/json',
          },
          credentials: 'include',
          body: JSON.stringify({
            userId: selectedUser,
            role: selectedRole,
          }),
        });

        if (!response.ok) {
          throw new Error('Failed to assign role');
        }

        toast.success("Role assigned successfully");

        // Fetch updated users list
        const usersResponse = await fetch(`${process.env.NEXT_PUBLIC_API_URL}/auth/users`, {
          credentials: 'include',
          cache: 'no-store',
        });

        if (!usersResponse.ok) {
          throw new Error('Failed to fetch updated users');
        }

        const data = await usersResponse.json();
        setUsers(data.users.map((user: any) => ({
          userId: user.userId,
          email: user.email,
          role: user.role as UserRole
        })));

        // Reset selections
        setSelectedUser("");
        setSelectedRole(undefined);
      } catch (error) {
        console.error('Error assigning role:', error);
        toast.error("Failed to assign role");
      }
    });
  };

  const availableRoles = [UserRole.BASIC, UserRole.PREMIUM, UserRole.PUBLIC];
  
  const filteredUsers = users.filter((user) =>
    user.email?.toLowerCase().includes(searchEmail.toLowerCase())
  );

  const handleError = (error: any) => {
    console.error('Error:', error);
    if (error instanceof Response) {
      switch (error.status) {
        case 403:
          toast.error("You don't have permission to perform this action");
          break;
        case 404:
          toast.error("User not found");
          break;
        default:
          toast.error("An error occurred while assigning role");
      }
    } else {
      toast.error("Failed to communicate with server");
    }
  };

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
