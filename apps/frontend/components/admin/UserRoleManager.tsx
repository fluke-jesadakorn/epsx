"use client";

import React, { useState, useCallback } from "react";
import { ROLES, type Role } from "@/constants/roles";

interface UserRoleData {
  userId: string;
  currentRole: Role;
}

export default function UserRoleManager() {
  const [userId, setUserId] = useState("");
  const [selectedRole, setSelectedRole] = useState<Role>(ROLES.BASIC);
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [userRole, setUserRole] = useState<UserRoleData | null>(null);

  // Fetch user's current role
  const fetchUserRole = useCallback(async (uid: string) => {
    try {
      const response = await fetch(
        `/api/auth/roles?userId=${encodeURIComponent(uid)}`
      );
      const data = await response.json();

      if (!response.ok) {
        throw new Error(data.error || "Failed to fetch user role");
      }

      setUserRole({
        userId: uid,
        currentRole: data.role,
      });
      setSelectedRole(data.role);
      setError(null);
    } catch (err) {
      setError(
        err instanceof Error ? err.message : "Failed to fetch user role"
      );
      setUserRole(null);
    }
  }, []);

  // Update user's role
  const updateUserRole = async (e: React.FormEvent) => {
    e.preventDefault();
    setLoading(true);
    setError(null);

    try {
      const response = await fetch("/api/auth/roles", {
        method: "POST",
        headers: {
          "Content-Type": "application/json",
        },
        body: JSON.stringify({
          userId,
          role: selectedRole,
        }),
      });

      const data = await response.json();

      if (!response.ok) {
        throw new Error(data.error || "Failed to update role");
      }

      // Refresh the displayed role
      await fetchUserRole(userId);
    } catch (err) {
      setError(err instanceof Error ? err.message : "Failed to update role");
    } finally {
      setLoading(false);
    }
  };

  return (
    <div className="p-4 max-w-md mx-auto bg-white rounded-lg shadow">
      <h2 className="text-2xl font-bold mb-4">User Role Manager</h2>

      <form onSubmit={updateUserRole} className="space-y-4">
        <div>
          <label
            htmlFor="userId"
            className="block text-sm font-medium text-gray-700"
          >
            User ID
          </label>
          <input
            type="text"
            id="userId"
            value={userId}
            onChange={(e) => {
              setUserId(e.target.value);
              if (e.target.value) {
                fetchUserRole(e.target.value);
              } else {
                setUserRole(null);
              }
            }}
            className="mt-1 block w-full rounded-md border-gray-300 shadow-sm focus:border-indigo-500 focus:ring-indigo-500"
            required
          />
        </div>

        {userRole && (
          <div>
            <label
              htmlFor="role"
              className="block text-sm font-medium text-gray-700"
            >
              Role
            </label>
            <select
              id="role"
              value={selectedRole}
              onChange={(e) => setSelectedRole(e.target.value as Role)}
              className="mt-1 block w-full rounded-md border-gray-300 shadow-sm focus:border-indigo-500 focus:ring-indigo-500"
              required
            >
              {Object.values(ROLES).map((role) => (
                <option key={role} value={role}>
                  {role.charAt(0).toUpperCase() + role.slice(1)}
                </option>
              ))}
            </select>
          </div>
        )}

        {error && <div className="text-red-600 text-sm">{error}</div>}

        <div className="flex justify-between items-center">
          {userRole && (
            <div className="text-sm text-gray-600">
              Current Role: {userRole.currentRole}
            </div>
          )}
          <button
            type="submit"
            disabled={loading || !userId}
            className={`
                px-4 py-2 rounded-md text-white
                ${
                  loading || !userId
                    ? "bg-gray-400"
                    : "bg-indigo-600 hover:bg-indigo-700"
                }
              `}
          >
            {loading ? "Updating..." : "Update Role"}
          </button>
        </div>
      </form>
    </div>
  );
}
